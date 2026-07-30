//! `verify_patricia_skeleton` — the in-circuit Patricia skeleton verifier (design doc §5, step 1).
//!
//! # Statement
//!
//! A satisfying assignment proves: for the returned `root` and the `K` returned leaf slots, there
//! is a *rooted skeleton* — a set of binary/edge/sibling units folding bottom-up, by the
//! production/consumption multiset with the root entry as the single external consumption, to a
//! unit `{ height, path = 0, hash = root }` — in which every leaf slot with `is_present = 1` is a
//! height-0 leaf unit at `path = key` carrying `value`, consumed by the fold. Every node hash on
//! the way up is `hash2` truncated to 251 bits (binary) or `hash2 + ℓ` (edge), matching the
//! production convention pinned by `golden_test`, so by collision resistance the present leaves
//! are leaves of *the* trie committing to `root`. `root = 0` (P5) forces every slot inert and
//! every claim absent; presence is derived per slot as `is_present ⟺ value ≠ 0` (P4) — neither is
//! ever witnessed.
//!
//! # Deliberately unproven
//!
//! * **Absence is a claim, not a proof**: an `is_present = 0` slot contributes nothing to the
//!   multiset, so nothing forces the skeleton to walk to that key. Step 2's insert path is what
//!   turns the claim into evidence (soundness doc, open-gap ledger).
//! * **Absent keys are unbound**: their key words are range-checked `u32`s and nothing more; the
//!   caller owes the binding of every key, present or absent (in the payments stack, step 3's batch
//!   binding).
//! * **Sibling interiors are opaque**: a sibling carries the dedicated `Opaque` kind (P7) and so
//!   claims nothing structural; whether its hash is a real subtree at its position is what the
//!   prev-side hash chain pins in the update circuit (design doc §6 Q3, decided).
//!
//! # Derivation argument (sketch)
//!
//! The root entry pins `{height H, path 0, hash root}`; every other consumed unit cancels against
//! a produced unit. Produced units are pinned top-down: a slot's `out` is consumed by its parent
//! (or the root entry), fixing `out.height/path/kind/hash`; the slot's constraints then determine
//! its children's positions (`2·path + bit`, `path·2^ℓ + edge_path`) and heights as *functions* of
//! `out`, and Blake binding fixes the children's hashes. Any unit not consumed — or consumed but
//! never produced — unbalances the multiset. Height strictly decreases downward, so no cycle can
//! feed itself. Padding cannot be laundered: the inert unit is all-zero, every live-slot rule
//! rejects it as input, and an absent leaf's inert contribution is absorbed only by a padded
//! binary slot (see `skeleton.rs` module docs on capacity).
//!
//! # Witness table (condensed — every `guess` site and what determines it)
//!
//! | witness | width | determined by |
//! |---|---|---|
//! | consumed unit fields (binary children, edge bottoms) | u16 + 16·u16 + u16 + 8·u32 | cancellation against a produced unit, whose fields its slot derives |
//! | slot `out.path` | 16·u16 | the child relations below it plus cancellation above it |
//! | sibling unit (height, path, hash; kind is derived) | u16 + 16·u16 + 8·u32 | cancellation by its consumer; locally only a non-empty hash (opaque **by design**, see above) |
//! | leaf `key` / `value` | 8·u32 each | present: the multiset consumes `{0, key, Leaf, value}`; absent: **unbound** — the caller binds |
//! | edge `ℓ` bits, `edge_path` limbs | 8·bool, 16·u16 | Blake preimage binding through the pinned `out.hash`, plus `ℓ ≥ 1`, `edge_path < 2^ℓ` |
//! | root hash words | 8·u32 | the caller's public-input binding |
//! | root kind | u16 | cancellation + the tag rules (`0` iff empty; `{2,3,4}` at height > 0, `{1,4}` at height 0) |
//! | carries, borrows, lo/hi splits, `2^ℓ − 1 − ep` | bool / u16 | unique given the ranged relation they close |
//! | is-zero / gated-inverse witnesses | field | the standard deterministic gadgets (`is_zero_words`, `value·w = flag`) |
//!
//! # Topology
//!
//! The emitted circuit depends only on `(witness.height, capacity)` — every loop runs over
//! capacity-fixed slot lists, every value is guessed (never folded into constants), and the
//! branch on `height == 0` is a shape branch. `skeleton_circuit_test` pins witness-independence
//! and a cross-process fingerprint.

use circuits::blake::{HashValue, blake2s_u32s};
use circuits::context::{Context, Var};
use circuits::eval;
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::ops::{Guess, add, eq, guess, mul, sub};
use circuits::wrappers::U32Wrapper;

use super::skeleton::{
    BinarySlot, EdgeSlot, SkeletonCapacity, SkeletonKind, SkeletonUnit, SkeletonWitness,
};
use super::word_gadgets::{
    LimbedWord256, add_small_words, compose_bits, constrain_below_pow2, constrain_double_plus_bit,
    constrain_shift_add, guess_bits, guess_limbed, guess_u16, truncate_251,
};
use crate::gadgets::is_zero_words;
use crate::patricia::reference::Word256;
use crate::permutation::permute_units;

#[cfg(test)]
#[path = "skeleton_circuit_test.rs"]
mod test;

/// A skeleton unit's variables: the 18 multiset words plus the limbed path they were built from.
pub struct SkeletonUnitVars {
    pub height: Var,
    pub path: LimbedWord256,
    pub kind: Var,
    pub hash: [U32Wrapper<Var>; 8],
}

impl SkeletonUnitVars {
    /// The unit's 18 words in `(height, path, kind, hash)` order — the atom the multiset permutes.
    pub fn words(&self) -> Vec<U32Wrapper<Var>> {
        let mut words = vec![U32Wrapper::new_unsafe(self.height)];
        words.extend(self.path.words);
        words.push(U32Wrapper::new_unsafe(self.kind));
        words.extend(self.hash);
        words
    }
}

/// One batch leaf slot: `key` and `value` for the caller to bind, and the derived presence flag
/// (`is_present ⟺ value ≠ 0`, design doc P4). An absent slot's key is *not* bound by the skeleton
/// — see the module docs.
pub struct VerifiedLeaf {
    pub key: HashValue<Var>,
    pub value: HashValue<Var>,
    pub is_present: Var,
}

/// Output of [`verify_patricia_skeleton`]: the root hash to bind to the public input, the batch
/// leaf slots, and the opaque sibling units (consumed by the update circuit, step 2).
pub struct VerifiedSkeleton {
    pub root: HashValue<Var>,
    pub leaves: Vec<VerifiedLeaf>,
    pub siblings: Vec<SkeletonUnitVars>,
}

/// Builds the skeleton-verifying circuit over `witness`, padded to `capacity`. See the module
/// docs for the statement. The topology depends only on `(witness.height, capacity)`.
///
/// Panics if the witness does not fit the capacity, or — at witness time — if the produced and
/// consumed unit multisets do not match; every other defect surfaces as a failed constraint.
pub fn verify_patricia_skeleton<Value: IValue>(
    ctx: &mut Context<Value>,
    witness: &SkeletonWitness,
    capacity: &SkeletonCapacity,
) -> VerifiedSkeleton {
    let padded = witness.padded(capacity);
    let keys: Vec<[U32Wrapper<Var>; 8]> =
        padded.leaves.iter().map(|unit| guess_word256(ctx, &unit.path)).collect();
    let siblings: Vec<SkeletonUnitVars> =
        padded.siblings.iter().map(|unit| sibling_slot(ctx, unit)).collect();
    let flow = skeleton_flow(ctx, &padded, &keys, &siblings);
    permute_units(ctx, &flow.consumed, &flow.produced);
    VerifiedSkeleton { root: flow.root, leaves: flow.leaves, siblings }
}

/// One skeleton's slot flow over externally supplied leaf keys and sibling units — the core both
/// [`verify_patricia_skeleton`] and the update circuit instantiate (P6). The caller guesses the
/// keys and siblings (shared between the update's two sides), runs one `permute_units` per flow,
/// and binds the root.
pub(crate) struct SkeletonFlow {
    pub produced: Vec<Vec<U32Wrapper<Var>>>,
    pub consumed: Vec<Vec<U32Wrapper<Var>>>,
    pub root: HashValue<Var>,
    pub leaves: Vec<VerifiedLeaf>,
}

/// Emits every slot of `padded` — leaf values against the shared `keys`, binaries, edges, the
/// shared `siblings`' words, and the root entry — returning the flow for the caller's
/// `permute_units`. `padded.leaves` must be aligned 1:1 with `keys`.
pub(crate) fn skeleton_flow<Value: IValue>(
    ctx: &mut Context<Value>,
    padded: &SkeletonWitness,
    keys: &[[U32Wrapper<Var>; 8]],
    siblings: &[SkeletonUnitVars],
) -> SkeletonFlow {
    assert!(keys.len() == padded.leaves.len(), "keys are 1:1 with leaf slots");
    assert!(siblings.len() == padded.siblings.len(), "shared siblings must fill every slot");
    let mut produced: Vec<Vec<U32Wrapper<Var>>> = Vec::new();
    let mut consumed: Vec<Vec<U32Wrapper<Var>>> = Vec::new();

    let leaves: Vec<VerifiedLeaf> = padded
        .leaves
        .iter()
        .zip(keys)
        .map(|(unit, key)| leaf_slot(ctx, unit, key, &mut produced))
        .collect();
    for slot in &padded.binaries {
        binary_slot(ctx, slot, &mut produced, &mut consumed);
    }
    for slot in &padded.edges {
        edge_slot(ctx, slot, &mut produced, &mut consumed);
    }
    for sibling in siblings {
        produced.push(sibling.words());
    }
    let root = root_entry(ctx, padded, &mut consumed);
    SkeletonFlow { produced, consumed, root, leaves }
}

/// Guesses a unit's four fields, each range-constrained to its width.
fn guess_unit<Value: IValue>(ctx: &mut Context<Value>, unit: &SkeletonUnit) -> SkeletonUnitVars {
    SkeletonUnitVars {
        height: guess_u16(ctx, unit.height),
        path: guess_limbed(ctx, &unit.path),
        kind: guess_u16(ctx, unit.kind.tag()),
        hash: guess_word256(ctx, &unit.hash),
    }
}

/// A batch leaf slot (P4): takes the shared `key` vars, guesses `value`, derives presence from
/// the value, and contributes `{0, key, Leaf, value}` to the multiset when present and the inert
/// unit otherwise — the entry's words are `is_present`-gated, never branched on.
fn leaf_slot<Value: IValue>(
    ctx: &mut Context<Value>,
    unit: &SkeletonUnit,
    key: &[U32Wrapper<Var>; 8],
    produced: &mut Vec<Vec<U32Wrapper<Var>>>,
) -> VerifiedLeaf {
    let value = guess_word256(ctx, &unit.hash);
    let value_zero = is_zero_words(ctx, &value);
    let one = ctx.one();
    let is_present = eval!(ctx, (one) - (value_zero));

    let mut entry = vec![U32Wrapper::new_unsafe(ctx.zero())];
    for word in key {
        entry.push(U32Wrapper::new_unsafe(mul(ctx, *word.get(), is_present)));
    }
    entry.push(U32Wrapper::new_unsafe(is_present));
    entry.extend(value);
    produced.push(entry);

    VerifiedLeaf { key: HashValue(*key), value: HashValue(value), is_present }
}

/// A binary slot: consumes two guessed children, produces `{h+1, path, Binary, hash2(l, r)}`.
/// Liveness is derived from the children; the inert tuple satisfies every relation with the
/// `is_live`-gated increments.
fn binary_slot<Value: IValue>(
    ctx: &mut Context<Value>,
    slot: &BinarySlot,
    produced: &mut Vec<Vec<U32Wrapper<Var>>>,
    consumed: &mut Vec<Vec<U32Wrapper<Var>>>,
) {
    let left = guess_unit(ctx, &slot.left);
    let right = guess_unit(ctx, &slot.right);
    let out_path = guess_limbed(ctx, &slot.out.path);
    let live_value = u32::from(!(slot.left.is_inert() && slot.right.is_inert()));

    let left_words = left.words();
    let right_words = right.words();
    let left_inert = is_zero_words(ctx, &left_words);
    let right_inert = is_zero_words(ctx, &right_words);
    let one = ctx.one();
    let is_live = eval!(ctx, (one) - ((left_inert) * (right_inert)));
    // A live slot's children are both live (padding cannot enter the live flow) and non-empty
    // (canonicity: a binary node has two non-empty children).
    forbid_when(ctx, left_inert, is_live);
    forbid_when(ctx, right_inert, is_live);
    let left_empty = is_zero_words(ctx, &left.hash);
    let right_empty = is_zero_words(ctx, &right.hash);
    forbid_when(ctx, left_empty, is_live);
    forbid_when(ctx, right_empty, is_live);

    // Children sit side by side one level down: heights equal, paths 2·out.path (+ bit).
    eq(ctx, left.height, right.height);
    let out_height = add(ctx, left.height, is_live);
    let zero = ctx.zero();
    constrain_double_plus_bit(ctx, &out_path, &left.path, zero, &slot.out.path, 0);
    constrain_double_plus_bit(ctx, &out_path, &right.path, is_live, &slot.out.path, live_value);

    // hash2(left, right), truncated to 251 bits; the gate is always emitted, the result gated.
    let message: Vec<U32Wrapper<Var>> = left.hash.iter().chain(&right.hash).copied().collect();
    let digest = blake2s_u32s(ctx, message, 64);
    let truncated = truncate_251(ctx, &digest);
    let out_hash = gate_words(ctx, is_live, &truncated);
    let binary_tag = ctx.constant(qm31_from_u32s(SkeletonKind::Binary.tag(), 0, 0, 0));
    let out_kind = mul(ctx, is_live, binary_tag);

    consumed.push(left_words);
    consumed.push(right_words);
    produced.push(unit_words(out_height, &out_path.words, out_kind, &out_hash));
}

/// An edge slot: consumes its guessed `bottom`, produces `{h+ℓ, path, Edge, hash2(bottom, ep)+ℓ}`.
/// Owns the canonicity rules — `ℓ ≥ 1`, `edge_path < 2^ℓ`, no edge over edge — and the §4.2
/// variable shift.
fn edge_slot<Value: IValue>(
    ctx: &mut Context<Value>,
    slot: &EdgeSlot,
    produced: &mut Vec<Vec<U32Wrapper<Var>>>,
    consumed: &mut Vec<Vec<U32Wrapper<Var>>>,
) {
    let bottom = guess_unit(ctx, &slot.bottom);
    let out_path = guess_limbed(ctx, &slot.out.path);
    let edge_path = guess_limbed(ctx, &slot.edge_path);
    let l_bits: [Var; 8] = guess_bits(ctx, slot.length, 8).try_into().expect("eight length bits");
    let length = compose_bits(ctx, &l_bits);

    // Liveness covers the whole slot-local tuple, so no field can be populated on a padded slot.
    let bottom_words = bottom.words();
    let mut slot_words = bottom_words.clone();
    slot_words.extend(edge_path.words);
    slot_words.push(U32Wrapper::new_unsafe(length));
    let slot_inert = is_zero_words(ctx, &slot_words);
    let one = ctx.one();
    let is_live = eval!(ctx, (one) - (slot_inert));

    // Canonicity: ℓ ≥ 1, aligned compressed bits, maximally merged, non-empty bottom.
    require_nonzero_when(ctx, length, is_live);
    constrain_below_pow2(ctx, &edge_path, &l_bits, &slot.edge_path, slot.length);
    let edge_tag = ctx.constant(qm31_from_u32s(SkeletonKind::Edge.tag(), 0, 0, 0));
    let not_edge = sub(ctx, bottom.kind, edge_tag);
    require_nonzero_when(ctx, not_edge, is_live);
    // P7: an Opaque bottom is allowed only at height 0, so no structural claim about an
    // untouched subtree ever feeds the no-edge-over-edge rule above.
    let opaque_tag = ctx.constant(qm31_from_u32s(SkeletonKind::Opaque.tag(), 0, 0, 0));
    let opaque_offset = sub(ctx, bottom.kind, opaque_tag);
    let bottom_opaque = is_zero_words(ctx, &[U32Wrapper::new_unsafe(opaque_offset)]);
    forbid_when(ctx, bottom.height, bottom_opaque);
    let bottom_empty = is_zero_words(ctx, &bottom.hash);
    forbid_when(ctx, bottom_empty, is_live);

    // Positions: bottom sits ℓ levels down at `out.path·2^ℓ + edge_path` (design doc §4.2).
    let out_height = add(ctx, bottom.height, length);
    constrain_shift_add(
        ctx,
        &out_path,
        &l_bits,
        &edge_path,
        &bottom.path,
        &slot.out.path,
        slot.length,
    );

    // hash2(bottom, edge_path) truncated, then the length *added* (design doc §4.4).
    let message: Vec<U32Wrapper<Var>> =
        bottom.hash.iter().chain(&edge_path.words).copied().collect();
    let digest = blake2s_u32s(ctx, message, 64);
    let truncated = truncate_251(ctx, &digest);
    let summed = add_small_words(ctx, &truncated, length, slot.length);
    let out_hash = gate_words(ctx, is_live, &summed);
    let out_kind = mul(ctx, is_live, edge_tag);

    consumed.push(bottom_words);
    produced.push(unit_words(out_height, &out_path.words, out_kind, &out_hash));
}

/// Guesses and shape-checks one sibling unit; the caller feeds its words into every flow that
/// shares it. The kind is *derived* — `Opaque` when live, never guessed (P7) — so a sibling makes
/// no structural claim and can never cancel against a slot output. The only shape rule is a
/// non-empty hash; everything else about a sibling is deliberately opaque (module docs).
pub(crate) fn sibling_slot<Value: IValue>(
    ctx: &mut Context<Value>,
    unit: &SkeletonUnit,
) -> SkeletonUnitVars {
    let height = guess_u16(ctx, unit.height);
    let path = guess_limbed(ctx, &unit.path);
    let hash = guess_word256(ctx, &unit.hash);
    // Liveness from the claim-free fields, so deriving the kind from it is not circular.
    let mut claim_free = vec![U32Wrapper::new_unsafe(height)];
    claim_free.extend(path.words);
    claim_free.extend(hash);
    let inert = is_zero_words(ctx, &claim_free);
    let one = ctx.one();
    let is_live = eval!(ctx, (one) - (inert));
    let opaque_tag = ctx.constant(qm31_from_u32s(SkeletonKind::Opaque.tag(), 0, 0, 0));
    let kind = mul(ctx, is_live, opaque_tag);

    let empty = is_zero_words(ctx, &hash);
    forbid_when(ctx, empty, is_live);

    SkeletonUnitVars { height, path, kind, hash }
}

/// The root entry (P5): consumes `{H, 0, kind, root}` when the guessed root is non-zero and the
/// inert unit otherwise, with emptiness *derived* from the root the caller binds. Returns the
/// root for that binding.
fn root_entry<Value: IValue>(
    ctx: &mut Context<Value>,
    witness: &SkeletonWitness,
    consumed: &mut Vec<Vec<U32Wrapper<Var>>>,
) -> HashValue<Var> {
    let root = guess_word256(ctx, &witness.root);
    let is_empty = is_zero_words(ctx, &root);
    let one = ctx.one();
    let live = eval!(ctx, (one) - (is_empty));
    let kind = guess_u16(ctx, root_kind(witness));

    // Empty ⇒ the padding tag; live at height > 0 ⇒ binary, edge, or a wholly untouched trie
    // (an Opaque root, the no-keys shape); live at height 0 ⇒ the single leaf, touched or not.
    // The `height == 0` branch is a shape branch, not a witness branch.
    forbid_when(ctx, kind, is_empty);
    let allowed: &[SkeletonKind] = if witness.height == 0 {
        &[SkeletonKind::Leaf, SkeletonKind::Opaque]
    } else {
        &[SkeletonKind::Binary, SkeletonKind::Edge, SkeletonKind::Opaque]
    };
    let mut product = ctx.one();
    for tag in allowed {
        let tag = ctx.constant(qm31_from_u32s(tag.tag(), 0, 0, 0));
        let offset = sub(ctx, kind, tag);
        product = mul(ctx, product, offset);
    }
    forbid_when(ctx, product, live);

    let height_constant = ctx.constant(qm31_from_u32s(witness.height, 0, 0, 0));
    let height_word = mul(ctx, live, height_constant);
    let zero = ctx.zero();
    let mut entry = vec![U32Wrapper::new_unsafe(height_word)];
    entry.extend((0..8).map(|_| U32Wrapper::new_unsafe(zero)));
    entry.push(U32Wrapper::new_unsafe(kind));
    entry.extend(root);
    consumed.push(entry);

    HashValue(root)
}

/// The witness value of the root unit's kind tag: the produced unit at `(height, path 0)` with the
/// claimed hash. Zero (padding) when absent — a rejected witness then fails constraints instead of
/// panicking.
fn root_kind(witness: &SkeletonWitness) -> u32 {
    let is_root = |unit: &SkeletonUnit| {
        unit.height == witness.height && unit.path == [0; 8] && unit.hash == witness.root
    };
    let outs = witness
        .binaries
        .iter()
        .map(|slot| slot.out)
        .chain(witness.edges.iter().map(|slot| slot.out));
    outs.chain(witness.siblings.iter().copied())
        .chain(witness.leaves.iter().copied())
        .find(is_root)
        .map_or(0, |unit| unit.kind.tag())
}

/// Guesses eight `u32` words (each two ranged 16-bit halves).
pub(crate) fn guess_word256<Value: IValue>(
    ctx: &mut Context<Value>,
    value: &Word256,
) -> [U32Wrapper<Var>; 8] {
    std::array::from_fn(|i| U32Wrapper::new_unsafe(Value::pack_u32(value[i])).guess(ctx))
}

/// The 18 multiset words of a derived output unit.
fn unit_words(
    height: Var,
    path: &[U32Wrapper<Var>; 8],
    kind: Var,
    hash: &[U32Wrapper<Var>; 8],
) -> Vec<U32Wrapper<Var>> {
    let mut words = vec![U32Wrapper::new_unsafe(height)];
    words.extend(*path);
    words.push(U32Wrapper::new_unsafe(kind));
    words.extend(*hash);
    words
}

/// `flag ? words : 0`, word-wise — the always-emitted-then-gated pattern for derived hashes.
fn gate_words<Value: IValue>(
    ctx: &mut Context<Value>,
    flag: Var,
    words: &[U32Wrapper<Var>; 8],
) -> [U32Wrapper<Var>; 8] {
    std::array::from_fn(|i| U32Wrapper::new_unsafe(mul(ctx, *words[i].get(), flag)))
}

/// Forces `condition = 0` whenever `flag = 1` (`flag` boolean): `condition · flag = 0`.
fn forbid_when<Value: IValue>(ctx: &mut Context<Value>, condition: Var, flag: Var) {
    let product = mul(ctx, condition, flag);
    let zero = ctx.zero();
    eq(ctx, product, zero);
}

/// Requires `value ≠ 0` whenever `flag = 1` (boolean): guesses `w` with `value · w = flag`, the
/// gated inverse trick. With `flag = 0` any `value` passes (`w = 0`).
fn require_nonzero_when<Value: IValue>(ctx: &mut Context<Value>, value: Var, flag: Var) {
    let value_concrete = ctx.get(value);
    let flag_concrete = ctx.get(flag);
    let zero_value = Value::from_qm31(qm31_from_u32s(0, 0, 0, 0));
    let witness =
        if value_concrete == zero_value { zero_value } else { flag_concrete / value_concrete };
    let w = guess(ctx, witness);
    let product = mul(ctx, value, w);
    eq(ctx, product, flag);
}
