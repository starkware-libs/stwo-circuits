//! Merkle "unpacker" circuit — unbalanced commitment via a fixed, depth-independent topology.
//!
//! Proves that a Merkle `root` commits to a given Merkle tree (an arbitrary, possibly
//! unbalanced binary tree of leaf hashes) under the Merkle convention used elsewhere in the
//! codebase ([`circuits_stark_verifier::merkle`], matching the reference `merkle_root_blake2s`). It
//! is the converse of an authentication-path check
//! ([`circuits_stark_verifier::merkle::verify_merkle_path`]): instead of opening a single leaf
//! against the root, it proves the *entire* set of leaves is exactly what the root commits to.
//!
//! # Design (wiring-by-multiset)
//!
//! See `DESIGN.md`. The circuit is built for a fixed **leaf-slot capacity** `L` (a power of two)
//! and handles any actual leaf count `n` with `1 ≤ n ≤ L` by zero-padding the leaf list with the
//! all-zero hash `Z`. There are exactly `M = L - 1` internal-node slots regardless of the tree
//! shape, so the gate structure depends only on `L`, never on the tree's depth.
//!
//! The shape of the tree is supplied by the caller as a [`BinaryTree`]; it determines the witness
//! values, not the gate counts (always `L` leaf slots and `L - 1` node slots). Each node slot
//! supplies two child hashes (`left`, `right`) as witness and derives its output with a
//! **copy-up rule**:
//!
//! ```text
//! out = if is_zero(right) { left } else { hash_node(left, right) }
//! ```
//!
//! A single strict multiset identity (via [`permute_hash_values`]) ties every produced hash to its
//! single consumption. The tree's one unconsumed output — the resolved root — is held back from the
//! produced side and **returned** instead, leaving a balanced identity over everything else:
//!
//! ```text
//! inputs  (consumed) = node.left[..] ++ node.right[..]              (length 2M = 2(L-1))
//! outputs (produced) = leaf_hash[..L] ++ node.out[..], minus root   (length L + M - 1 = 2(L-1))
//! ```
//!
//! Every node output except the root is consumed exactly once as a parent's child; each real leaf
//! cancels with its consumption; each padding `Z` is dropped by a node whose `right = Z` (so
//! `out = left`, keeping the present child flowing). Both sides reduce to ∅ iff the witness
//! describes a valid Merkle tree over exactly the `n` real leaves whose root is the returned hash.
//! The caller binds that returned root to the claimed commitment. See `DESIGN.md` §Soundness for
//! the full argument.

use circuits::blake::HashValue;
use circuits::context::{Context, Var};
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::ops::{Guess, add, eq, guess, mul};
use circuits::wrappers::U32Wrapper;
use circuits_stark_verifier::merkle::hash_node;
use stwo::core::fields::qm31::QM31;

use crate::permutation::permute_hash_values;
use crate::tree::BinaryTree;

#[cfg(test)]
#[path = "unpacker_test.rs"]
mod test;

/// Output of [`verify_merkle_commitment`], holding the computed Merkle `root` together with the
/// padded `leaves` hashes.
pub struct VerifiedMerkleTree {
    pub root: HashValue<Var>,
    pub leaves: Vec<HashValue<Var>>,
}

/// Selects between two hash values word-by-word:
/// `select(selector, if_zero, if_one) = if_zero + selector * (if_one - if_zero)`.
///
/// Assumes `selector` is `0` or `1`.
fn select_hash<Value: IValue>(
    context: &mut Context<Value>,
    selector: Var,
    if_zero: &HashValue<Var>,
    if_one: &HashValue<Var>,
) -> HashValue<Var> {
    HashValue(std::array::from_fn(|i| {
        let diff = circuits::eval!(context, (*if_one[i].get()) - (*if_zero[i].get()));
        let result = circuits::eval!(context, (*if_zero[i].get()) + ((selector) * (diff)));
        U32Wrapper::new_unsafe(result)
    }))
}

/// Reads the concrete value of a [`HashValue<Var>`] back out of the context.
fn hash_value_of<Value: IValue>(
    context: &Context<Value>,
    hash: &HashValue<Var>,
) -> HashValue<Value> {
    HashValue(std::array::from_fn(|i| U32Wrapper::new_unsafe(context.get(*hash[i].get()))))
}

/// Guesses a fresh all-zero hash `Z` as witness — every word is a freshly guessed variable holding
/// zero (range-constrained to a valid `u32` like any other [`HashValue`] guess).
///
/// Crucially these are *distinct* variables, not the canonical zero constant (`Var { idx: 0 }`).
/// [`add`]/[`mul`] constant-fold away any operand that *is* the canonical zero, so building padding
/// from the zero constant would emit fewer gates than a real (non-zero) slot and make the gate
/// count scale with the real-leaf count `n`. Guessing zeros instead keeps padding slots
/// structurally identical to real slots, so the circuit depends only on `capacity`, never on `n` or
/// tree shape.
///
/// Soundness is unaffected: a padding `Z` is unconstrained witness, but any non-zero value a prover
/// puts here is a produced hash the multiset must consume, so it gets hashed into the tree and
/// changes the root — breaking the caller's binding to the claimed commitment (see `DESIGN.md`).
fn guess_zero_hash<Value: IValue>(context: &mut Context<Value>) -> HashValue<Var> {
    let zero: HashValue<Value> =
        HashValue(std::array::from_fn(|_| U32Wrapper::new_unsafe(Value::pack_u32(0))));
    zero.guess(context)
}

/// Guesses a concrete [`HashValue<QM31>`] as witness in the `Value` context, lifting each word's
/// `QM31` value through [`IValue::from_qm31`] (a no-op for a `QM31` context; discarded for a value-
/// less topology build). Each word is range-constrained like any other [`HashValue`] guess.
fn guess_hash_value<Value: IValue>(
    context: &mut Context<Value>,
    hash: &HashValue<QM31>,
) -> HashValue<Var> {
    let value: HashValue<Value> = HashValue(std::array::from_fn(|i| {
        U32Wrapper::new_unsafe(Value::from_qm31(*hash[i].get()))
    }));
    value.guess(context)
}

/// Returns a `0/1` selector variable that is `1` iff `hash` is the all-zero hash `Z`.
///
/// The eight words are summed into a single field element `acc`. Because every word of a guessed
/// [`HashValue`] is range-constrained to a valid `u32` packing `(low_u16, high_u16, 0, 0)` (see
/// [`HashValue::guess`]), each coordinate of the sum stays below `8 · 2^16 < M31::P`, so the sum
/// never wraps and `acc == 0` (in `QM31`) iff every word is zero, i.e. `hash == Z`.
///
/// `is_zero_hash` is then pinned to `[acc == 0]` by the standard is-zero gadget with witness
/// `inv_or_zero`:
/// * `acc * is_zero_hash == 0` forces `is_zero_hash` to be false (`0`) whenever `acc != 0`ץ
/// * `acc * inv_or_zero + is_zero_hash == 1` forces `is_zero_hash` to be true (`1`) whenever `acc
///   == 0`, and otherwise requires `inv_or_zero = 1/acc`.
///
/// Together these uniquely determine `is_zero_hash ∈ {0, 1}` as a deterministic function of `hash`,
/// so the copy-up rule cannot be steered by a malicious prover.
fn is_zero_hash<Value: IValue>(context: &mut Context<Value>, hash: &HashValue<Var>) -> Var {
    let zero = context.zero();
    let one = context.one();

    // acc = sum of the eight words.
    let acc = hash.iter().skip(1).fold(*hash[0].get(), |acc, w| add(context, acc, *w.get()));

    // Witness values for the is-zero gadget.
    let acc_val = context.get(acc);
    let is_zero = acc_val == Value::from_qm31(qm31_from_u32s(0, 0, 0, 0));
    let (is_zero_hash_val, inv_or_zero_val) = if is_zero {
        (Value::from_qm31(qm31_from_u32s(1, 0, 0, 0)), Value::from_qm31(qm31_from_u32s(0, 0, 0, 0)))
    } else {
        (
            Value::from_qm31(qm31_from_u32s(0, 0, 0, 0)),
            Value::from_qm31(qm31_from_u32s(1, 0, 0, 0)) / acc_val,
        )
    };
    let is_zero_hash = guess(context, is_zero_hash_val);
    let inv_or_zero = guess(context, inv_or_zero_val);

    // acc * is_zero_hash == 0
    let acc_is_zero_hash = mul(context, acc, is_zero_hash);
    eq(context, acc_is_zero_hash, zero);
    // acc * inv_or_zero + is_zero_hash == 1
    let acc_inv_or_zero = mul(context, acc, inv_or_zero);
    let acc_inv_or_zero_plus_is_zero_hash = add(context, acc_inv_or_zero, is_zero_hash);
    eq(context, acc_inv_or_zero_plus_is_zero_hash, one);

    is_zero_hash
}

/// Derives one node's output from its two child hashes using the copy-up rule:
/// `out = is_zero(right) ? left : hash_node(left, right)`.
///
/// The `hash_node` gate is always emitted (the topology is fixed); the [`is_zero_hash`] selector
/// picks the copied-up `left` for padding edges and the genuine hash otherwise.
fn handle_node<Value: IValue>(
    context: &mut Context<Value>,
    left: &HashValue<Var>,
    right: &HashValue<Var>,
) -> HashValue<Var> {
    let hashed = hash_node(context, left, right);
    let right_is_zero = is_zero_hash(context, right);
    // selector = 1 -> copy `left`; selector = 0 -> use the hash.
    select_hash(context, right_is_zero, &hashed, left)
}

/// Emits one node slot from two already-resolved child hashes: guesses fresh `left`/`right`
/// witnesses equal to the children, derives `out` via the copy-up rule, and records the slot in
/// `lefts`/`rights`/`outs`. Children are tied to their producers only through the multiset
/// identity, never topologically.
fn push_node<Value: IValue>(
    context: &mut Context<Value>,
    left: &HashValue<Var>,
    right: &HashValue<Var>,
    lefts: &mut Vec<HashValue<Var>>,
    rights: &mut Vec<HashValue<Var>>,
    outs: &mut Vec<HashValue<Var>>,
) -> HashValue<Var> {
    let left = hash_value_of(context, left).guess(context);
    let right = hash_value_of(context, right).guess(context);
    let out = handle_node(context, &left, &right);
    lefts.push(left);
    rights.push(right);
    outs.push(out.clone());
    out
}

/// Emits `tree`'s internal nodes depth-first into `lefts`/`rights`/`outs`, returning the resolved
/// root hash (`n - 1` nodes for `n` leaves, any shape).
///
/// Leaves are *not* guessed here — they were guessed up front into `leaf_vars` (in this walk's
/// left-to-right order) and consumed by `leaf_idx`. Guessing them inline would interleave leaf and
/// node allocations in a shape-dependent order; emitting only nodes here keeps the variable layout
/// (and thus the whole circuit) identical across all tree shapes of a given `capacity`.
fn build_tree<Value: IValue>(
    context: &mut Context<Value>,
    tree: &BinaryTree<HashValue<QM31>>,
    leaf_vars: &[HashValue<Var>],
    leaf_idx: &mut usize,
    lefts: &mut Vec<HashValue<Var>>,
    rights: &mut Vec<HashValue<Var>>,
    outs: &mut Vec<HashValue<Var>>,
) -> HashValue<Var> {
    match tree {
        BinaryTree::Leaf(_) => {
            let leaf = leaf_vars[*leaf_idx].clone();
            *leaf_idx += 1;
            leaf
        }
        BinaryTree::Node(children) => {
            let left = build_tree(context, &children[0], leaf_vars, leaf_idx, lefts, rights, outs);
            let right = build_tree(context, &children[1], leaf_vars, leaf_idx, lefts, rights, outs);
            push_node(context, &left, &right, lefts, rights, outs)
        }
    }
}

/// Builds a circuit (with a fixed `capacity`) that verifies `root` is the Merkle root over `leaves`,
/// returning both as a [`VerifiedMerkleTree`]. The witness `tree` encodes the leaf values and the
/// tree shape.
pub fn verify_merkle_commitment<Value: IValue>(
    context: &mut Context<Value>,
    tree: &BinaryTree<HashValue<QM31>>,
    capacity: usize,
) -> VerifiedMerkleTree {
    let leaf_values = tree.leaves();
    let n = leaf_values.len();
    assert!(n >= 1, "a Merkle tree must have at least one leaf");
    assert!(capacity.is_power_of_two(), "capacity must be a power of two, got {capacity}");
    assert!(n <= capacity, "got {n} leaves but capacity is only {capacity}");

    // Guess all `capacity` leaf hashes up front (real then padding), so `build_tree` can consume
    // pre-guessed leaves and emit only nodes — see its docs for why this ordering is required.
    let real_leaves: Vec<HashValue<Var>> =
        leaf_values.iter().map(|hash| guess_hash_value(context, hash)).collect();
    let pads: Vec<HashValue<Var>> = (n..capacity).map(|_| guess_zero_hash(context)).collect();
    let leaf_slots: Vec<HashValue<Var>> = real_leaves.iter().chain(pads.iter()).cloned().collect();

    let mut lefts: Vec<HashValue<Var>> = Vec::new();
    let mut rights: Vec<HashValue<Var>> = Vec::new();
    let mut outs: Vec<HashValue<Var>> = Vec::new();

    // Emit the real tree's `n - 1` internal nodes (depth-first), then drop each padding `Z` with a
    // copy-up node (`right = Z`, so `out = left`), chaining the real root up. Together these are
    // exactly `capacity - 1` uniform node slots, regardless of tree shape or `n`.
    let mut leaf_idx = 0;
    let mut chain =
        build_tree(context, tree, &real_leaves, &mut leaf_idx, &mut lefts, &mut rights, &mut outs);
    for pad in &pads {
        chain = push_node(context, &chain, pad, &mut lefts, &mut rights, &mut outs);
    }

    // Check that inputs (the consumed children) are, as a multiset, the leaves + outputs, with the
    // root held out: it is the one produced hash with no consumption, so it is popped off and
    // returned for the caller to bind rather than cancelled internally.
    let inputs: Vec<HashValue<Var>> = lefts.into_iter().chain(rights).collect();
    let mut outputs: Vec<HashValue<Var>> = leaf_slots.iter().cloned().chain(outs).collect();
    let root = outputs.pop().expect("a Merkle tree must have at least one leaf");
    permute_hash_values(context, &inputs, &outputs);
    VerifiedMerkleTree { root, leaves: leaf_slots }
}
