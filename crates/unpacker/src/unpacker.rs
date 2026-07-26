//! Merkle "unpacker" circuit — unbalanced commitment via a fixed, depth-independent topology.
//!
//! Proves that a Merkle `root` commits to a given Merkle tree (an arbitrary, possibly
//! unbalanced binary tree) whose entries are [`Node`]s pairing a `circuit_hash` (which circuit
//! produced the entry) with a `subtree_hash` (the Merkle hash of the subtree). It is the converse
//! of an authentication-path check ([`circuits_stark_verifier::merkle::verify_merkle_path`]):
//! instead of opening a single leaf against the root, it proves the *entire* set of leaves is
//! exactly what the root commits to.
//!
//! # Node hashing
//!
//! Each entry carries two hashes; an internal node's `subtree_hash` binds *both* fields of *both*
//! children, mirroring the multiverifier's `(circuit_hash ‖ output)` preimage layout:
//!
//! ```text
//! subtree_hash = blake2s( left.circuit_hash ‖ left.subtree_hash ‖ right.circuit_hash ‖ right.subtree_hash )
//! ```
//!
//! A node's own `circuit_hash` is supplied as witness (it is bound when its parent consumes it as a
//! child); the root's `circuit_hash` is bound by the caller. Because every child's `circuit_hash`
//! enters its parent's `subtree_hash` preimage, the returned root commits — transitively — to every
//! `circuit_hash` and `subtree_hash` in the tree.
//!
//! # Design (wiring-by-multiset)
//!
//! The circuit is built for a fixed **leaf-slot capacity** `L` (a power of two) and handles any
//! actual leaf count `n` with `1 ≤ n ≤ L` by zero-padding the leaf list with the all-zero node `Z`.
//! There are exactly `M = L - 1` internal-node slots regardless of the tree shape, so the gate
//! structure depends only on `L`, never on the tree's depth.
//!
//! The shape of the tree is supplied by the caller as a [`BinaryTree`]; it determines the witness
//! values, not the gate counts (always `L` leaf slots and `L - 1` node slots). Each node slot
//! supplies two child nodes (`left`, `right`) plus its own `circuit_hash` as witness and derives
//! its output with a **copy-up rule**:
//!
//! ```text
//! out = if is_zero(right) { left } else { Node { circuit_hash, subtree_hash: hash(left, right) } }
//! ```
//!
//! A single strict multiset identity (via [`permute_units`]) ties every produced node — as a
//! whole `(circuit_hash, subtree_hash)` pair — to its single consumption. The tree's one unconsumed
//! output — the resolved root — is held back from the produced side and **returned** instead,
//! leaving a balanced identity over everything else:
//!
//! ```text
//! inputs  (consumed) = node.left[..] ++ node.right[..]              (length 2M = 2(L-1))
//! outputs (produced) = leaf[..L] ++ node.out[..], minus root        (length L + M - 1 = 2(L-1))
//! ```
//!
//! Every node output except the root is consumed exactly once as a parent's child; each real leaf
//! cancels with its consumption; each padding `Z` is dropped by a node whose `right = Z` (so
//! `out = left`, keeping the present child flowing). Both sides reduce to ∅ iff the witness
//! describes a valid Merkle tree over exactly the `n` real leaves whose root is the returned node.
//! The caller binds the returned root (both fields) to the claimed commitment.

use circuits::blake::{HashValue, blake2s_u32s};
use circuits::context::{Context, Var};
use circuits::ivalue::IValue;
use circuits::ops::Guess;
use circuits::wrappers::U32Wrapper;
use stwo::core::fields::qm31::QM31;

use crate::gadgets::{
    guess_hash_value, guess_zero_hash, hash_value_of, is_zero_words, select_hash,
};
use crate::permutation::permute_units;
use crate::tree::BinaryTree;

#[cfg(test)]
#[path = "unpacker_test.rs"]
mod test;

/// A tree entry: `circuit_hash` (which circuit produced this position) paired with `subtree_hash`
/// (the Merkle hash of the subtree rooted here). Both are [`HashValue`]s (eight Blake2s output
/// words).
///
/// `T` is the per-element representation: [`Var`] for variables inside a [`Context`], or `QM31` for
/// concrete witness values.
#[derive(Clone)]
pub struct Node<T> {
    pub circuit_hash: HashValue<T>,
    pub subtree_hash: HashValue<T>,
}

impl<Value: IValue> Guess<Value> for Node<Value> {
    type Target = Node<Var>;
    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        Node {
            circuit_hash: self.circuit_hash.guess(context),
            subtree_hash: self.subtree_hash.guess(context),
        }
    }
}

impl Node<Var> {
    /// The node's sixteen words in `circuit_hash ‖ subtree_hash` order — the atomic unit permuted
    /// by [`permute_units`] and hashed by [`hash_node`].
    fn words(&self) -> Vec<U32Wrapper<Var>> {
        self.circuit_hash.iter().chain(self.subtree_hash.iter()).copied().collect()
    }
}

/// Output of [`verify_merkle_commitment`], holding the computed root [`Node`] together with the
/// padded leaf nodes.
pub struct VerifiedMerkleTree {
    pub root: Node<Var>,
    pub leaves: Vec<Node<Var>>,
}

/// Reads the concrete value of a [`Node<Var>`] back out of the context.
fn node_value_of<Value: IValue>(context: &Context<Value>, node: &Node<Var>) -> Node<Value> {
    Node {
        circuit_hash: hash_value_of(context, &node.circuit_hash),
        subtree_hash: hash_value_of(context, &node.subtree_hash),
    }
}

/// Guesses a fresh all-zero [`Node`] (both hashes zero) — see [`guess_zero_hash`] for why padding
/// is guessed rather than built from the zero constant.
///
/// Soundness is unaffected: a padding `Z` is unconstrained witness, but any non-zero value a prover
/// puts here is a produced node the multiset must consume, so it gets hashed into the tree and
/// changes the root — breaking the caller's binding to the claimed commitment.
fn guess_zero_node<Value: IValue>(context: &mut Context<Value>) -> Node<Var> {
    Node { circuit_hash: guess_zero_hash(context), subtree_hash: guess_zero_hash(context) }
}

/// Guesses a concrete [`Node<QM31>`] (both hashes) as witness. See [`guess_hash_value`].
fn guess_node<Value: IValue>(context: &mut Context<Value>, node: &Node<QM31>) -> Node<Var> {
    Node {
        circuit_hash: guess_hash_value(context, &node.circuit_hash),
        subtree_hash: guess_hash_value(context, &node.subtree_hash),
    }
}

/// Returns a `0/1` selector variable that is `1` iff `node` is the all-zero node `Z` (both hashes
/// all zero). See [`is_zero_words`] for the gadget; its determinism guarantees the copy-up rule
/// cannot be steered by a malicious prover.
fn is_zero_node<Value: IValue>(context: &mut Context<Value>, node: &Node<Var>) -> Var {
    // TODO(ilya): Condsider checking only the subtree_hash and keeping circuit_hash unconstrained.
    is_zero_words(context, &node.words())
}

/// Computes an internal node's `subtree_hash` by hashing the concatenation
/// `left.circuit_hash ‖ left.subtree_hash ‖ right.circuit_hash ‖ right.subtree_hash` (four
/// eight-word hashes = 128 bytes).
///
/// The words are already in `blake2s_u32s` message-word form (each is a previous Blake2s output),
/// so they are fed directly as the 32 message words without any unpacking.
fn hash_node<Value: IValue>(
    context: &mut Context<Value>,
    left: &Node<Var>,
    right: &Node<Var>,
) -> HashValue<Var> {
    let mut words: Vec<U32Wrapper<Var>> = Vec::with_capacity(32);
    words.extend(left.words());
    words.extend(right.words());
    blake2s_u32s(context, words, 128)
}

/// Derives one node's output from its two child nodes and its supplied `circuit_hash` using the
/// copy-up rule: `out = is_zero(right) ? left : Node { circuit_hash, subtree_hash: hash(left,
/// right) }`.
///
/// The [`hash_node`] gate is always emitted (the topology is fixed); the [`is_zero_node`] selector
/// picks the copied-up `left` (both fields) for padding edges and the genuine node otherwise.
fn handle_node<Value: IValue>(
    context: &mut Context<Value>,
    circuit_hash: &HashValue<Var>,
    left: &Node<Var>,
    right: &Node<Var>,
) -> Node<Var> {
    let hashed = hash_node(context, left, right);
    let right_is_zero = is_zero_node(context, right);
    // selector = 1 -> copy `left` (both fields); selector = 0 -> use the supplied `circuit_hash`
    // and the genuine hash.
    Node {
        circuit_hash: select_hash(context, right_is_zero, circuit_hash, &left.circuit_hash),
        subtree_hash: select_hash(context, right_is_zero, &hashed, &left.subtree_hash),
    }
}

/// Emits one node slot from two already-resolved child nodes and a supplied `circuit_hash`: guesses
/// fresh `left`/`right`/`circuit_hash` witnesses, derives `out` via the copy-up rule, and records
/// the slot in `lefts`/`rights`/`outs`. Children are tied to their producers only through the
/// multiset identity, never topologically.
fn push_node<Value: IValue>(
    context: &mut Context<Value>,
    circuit_hash: &HashValue<QM31>,
    left: &Node<Var>,
    right: &Node<Var>,
    lefts: &mut Vec<Node<Var>>,
    rights: &mut Vec<Node<Var>>,
    outs: &mut Vec<Node<Var>>,
) -> Node<Var> {
    let left = node_value_of(context, left).guess(context);
    let right = node_value_of(context, right).guess(context);
    let circuit_hash = guess_hash_value(context, circuit_hash);
    let out = handle_node(context, &circuit_hash, &left, &right);
    lefts.push(left);
    rights.push(right);
    outs.push(out.clone());
    out
}

/// Emits `tree`'s internal nodes depth-first into `lefts`/`rights`/`outs`, returning the resolved
/// root node (`n - 1` nodes for `n` leaves, any shape). Each internal node's supplied
/// `circuit_hash` is read from the tree.
///
/// Leaves are *not* guessed here — they were guessed up front into `leaf_vars` (in this walk's
/// left-to-right order) and consumed by `leaf_idx`. Guessing them inline would interleave leaf and
/// node allocations in a shape-dependent order; emitting only nodes here keeps the variable layout
/// (and thus the whole circuit) identical across all tree shapes of a given `capacity`.
fn build_tree<Value: IValue>(
    context: &mut Context<Value>,
    tree: &BinaryTree<Node<QM31>, HashValue<QM31>>,
    leaf_vars: &[Node<Var>],
    leaf_idx: &mut usize,
    lefts: &mut Vec<Node<Var>>,
    rights: &mut Vec<Node<Var>>,
    outs: &mut Vec<Node<Var>>,
) -> Node<Var> {
    match tree {
        BinaryTree::Leaf(_) => {
            let leaf = leaf_vars[*leaf_idx].clone();
            *leaf_idx += 1;
            leaf
        }
        BinaryTree::Node(circuit_hash, children) => {
            let left = build_tree(context, &children[0], leaf_vars, leaf_idx, lefts, rights, outs);
            let right = build_tree(context, &children[1], leaf_vars, leaf_idx, lefts, rights, outs);
            push_node(context, circuit_hash, &left, &right, lefts, rights, outs)
        }
    }
}

/// Builds a circuit (with a fixed `capacity`) that verifies `root` is the Merkle root over the
/// tree's leaf [`Node`]s, returning both as a [`VerifiedMerkleTree`]. The witness `tree` encodes
/// the leaf values, each internal node's `circuit_hash`, and the tree shape.
pub fn verify_merkle_commitment<Value: IValue>(
    context: &mut Context<Value>,
    tree: &BinaryTree<Node<QM31>, HashValue<QM31>>,
    capacity: usize,
) -> VerifiedMerkleTree {
    let leaf_values = tree.leaves();
    let n = leaf_values.len();
    assert!(n >= 1, "a Merkle tree must have at least one leaf");
    assert!(capacity.is_power_of_two(), "capacity must be a power of two, got {capacity}");
    assert!(n <= capacity, "got {n} leaves but capacity is only {capacity}");

    // Guess all `capacity` leaf nodes up front (real then padding), so `build_tree` can consume
    // pre-guessed leaves and emit only nodes — see its docs for why this ordering is required.
    let real_leaves: Vec<Node<Var>> =
        leaf_values.iter().map(|node| guess_node(context, node)).collect();
    let pads: Vec<Node<Var>> = (n..capacity).map(|_| guess_zero_node(context)).collect();
    let leaf_slots: Vec<Node<Var>> = real_leaves.iter().chain(pads.iter()).cloned().collect();

    let mut lefts: Vec<Node<Var>> = Vec::new();
    let mut rights: Vec<Node<Var>> = Vec::new();
    let mut outs: Vec<Node<Var>> = Vec::new();

    // Emit the real tree's `n - 1` internal nodes (depth-first), then drop each padding `Z` with a
    // copy-up node (`right = Z`, so `out = left`), chaining the real root up. Together these are
    // exactly `capacity - 1` uniform node slots, regardless of tree shape or `n`. Padding nodes
    // always copy up, so their supplied `circuit_hash` is unused — a guessed all-zero hash keeps
    // them structurally identical to real node slots.
    let zero_circuit_hash = HashValue::<QM31>::from([0u32; 8]);
    let mut leaf_idx = 0;
    let mut chain =
        build_tree(context, tree, &real_leaves, &mut leaf_idx, &mut lefts, &mut rights, &mut outs);
    for pad in &pads {
        chain =
            push_node(context, &zero_circuit_hash, &chain, pad, &mut lefts, &mut rights, &mut outs);
    }

    // Check that inputs (the consumed children) are, as a multiset, the leaves + outputs, with the
    // root held out: it is the one produced node with no consumption, so it is popped off and
    // returned for the caller to bind rather than cancelled internally. Each node is permuted as a
    // whole `(circuit_hash, subtree_hash)` pair (sixteen words), so both fields are tied together.
    let inputs: Vec<Vec<U32Wrapper<Var>>> =
        lefts.iter().chain(rights.iter()).map(Node::words).collect();
    let mut outputs: Vec<Node<Var>> = leaf_slots.iter().cloned().chain(outs).collect();
    let root = outputs.pop().expect("a Merkle tree must have at least one leaf");
    let output_units: Vec<Vec<U32Wrapper<Var>>> = outputs.iter().map(Node::words).collect();
    permute_units(context, &inputs, &output_units);
    VerifiedMerkleTree { root, leaves: leaf_slots }
}
