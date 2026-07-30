//! RejectionCase harness for the skeleton witness: a named catalogue of modifications that a sound
//! skeleton circuit must reject. Each row of [`REJECTION_CASES`] is one row of the rejection table
//! in `docs/payments-circuit-soundness.md` — the constraint [`Family`] it failure modes and the
//! [`Check`] that rejects it today.
//!
//! # The two seams
//!
//! [`oracle`] — the out-of-circuit [`witness_invariants`] — decides "rejected" with attribution:
//! `skeleton_test` asserts each case fails on the exact [`Check`] it targets. The circuit is the
//! second seam: `skeleton_circuit_test::every_rejection_case_is_rejected_by_the_circuit` runs the
//! same catalogue through `verify_patricia_skeleton` end to end (attribution there is coarser —
//! rejection may surface as any failed constraint or as an unhostable witness).
//!
//! # Attribution
//!
//! A rejection case that fails for an unintended reason proves nothing, so a modification meant for
//! a local rule is applied *consistently* — [`retag`] rewrites the producer's copy and every
//! consumer's copy — leaving the production/consumption multiset balanced. RejectionCases that
//! target the multiset itself are the ones that deliberately touch a single side. Tests assert the
//! exact `Check`, never merely that something failed.

use super::{
    BinarySlot, Check, EdgeSlot, SkeletonKind, SkeletonUnit, SkeletonWitness, Violation,
    witness_invariants,
};
use crate::patricia::reference::EMPTY_HASH;

/// The constraint family a rejection case failure modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Family {
    /// Class tags and the domain separation that keeps classes from exchanging units.
    KindConfusion,
    /// `leaf.path = key`, `child.path = 2·parent.path + bit`, `bottom.path = parent.path·2^ℓ +
    /// edge.path`, and the height relations.
    PositionBinding,
    /// `hash_binary` / `hash_edge` over a slot's inputs.
    NodeHashing,
    /// Canonical form: `ℓ ∈ [1, height]`, aligned edge paths, no edge over edge, no empty node.
    Canonicity,
    /// The production/consumption multiset and the derived sibling count.
    MultisetBalance,
    /// Padded slots are forced to the inert tuple.
    LivePadding,
    /// The held-out root unit is the claimed root.
    RootBinding,
    /// The derived presence and emptiness flags: `is_present ⟺ hash ≠ 0` per leaf slot (P4) and
    /// `root = 0 ⟹ everything inert` (P5).
    PresenceBinding,
}

/// One row of the rejection table.
pub struct RejectionCase {
    pub label: &'static str,
    pub family: Family,
    /// The invariant expected to reject it. Asserted exactly, for attribution.
    pub detected_by: Check,
    /// Whether it modifies with a padded slot and so needs a padded witness.
    pub needs_padding: bool,
    pub apply: fn(&mut SkeletonWitness),
}

/// The catalogue. Slot-indexed rejection_cases target index `0`; because [`walk`](super::walk)
/// emits slots post-order, index `0` is never the root slot when there is more than one.
pub const REJECTION_CASES: &[RejectionCase] = &[
    RejectionCase {
        label: "edge unit relabelled as binary",
        family: Family::KindConfusion,
        detected_by: Check::KindTag,
        needs_padding: false,
        apply: relabel_edge_as_binary,
    },
    RejectionCase {
        label: "sibling subtree claims to be a leaf",
        family: Family::KindConfusion,
        detected_by: Check::KindTag,
        needs_padding: false,
        apply: sibling_claims_leaf,
    },
    RejectionCase {
        label: "leaf and sibling units swap classes",
        family: Family::KindConfusion,
        detected_by: Check::KindTag,
        needs_padding: false,
        apply: swap_leaf_with_sibling,
    },
    RejectionCase {
        label: "leaf path substituted away from its key",
        family: Family::PositionBinding,
        detected_by: Check::Position,
        needs_padding: false,
        apply: substitute_leaf_path,
    },
    RejectionCase {
        label: "binary unit height off by one",
        family: Family::PositionBinding,
        detected_by: Check::Position,
        needs_padding: false,
        apply: perturb_binary_height,
    },
    RejectionCase {
        label: "sibling hash modified",
        family: Family::NodeHashing,
        detected_by: Check::Hashing,
        needs_padding: false,
        apply: corrupt_sibling_hash,
    },
    RejectionCase {
        label: "present leaf value zeroed everywhere (its consumer holds an empty node)",
        family: Family::Canonicity,
        detected_by: Check::Canonicity,
        needs_padding: false,
        apply: zero_leaf_value,
    },
    // The two slot-only presence flips break the multiset (the flipped slot's contribution
    // dangles or goes missing), but the live-count identity — a corollary of the same balance —
    // breaks first in the oracle's declared order, so that is where they are attributed.
    RejectionCase {
        label: "present leaf value zeroed in its slot only (claimed absent)",
        family: Family::PresenceBinding,
        detected_by: Check::SiblingCount,
        needs_padding: false,
        apply: zero_leaf_value_slot_only,
    },
    RejectionCase {
        label: "absent leaf slot given a live value (claimed present)",
        family: Family::PresenceBinding,
        detected_by: Check::SiblingCount,
        needs_padding: false,
        apply: fake_present_leaf,
    },
    RejectionCase {
        label: "live skeleton claimed against the empty root",
        family: Family::PresenceBinding,
        detected_by: Check::Root,
        needs_padding: false,
        apply: claim_empty_root,
    },
    RejectionCase {
        label: "edge length 0",
        family: Family::Canonicity,
        detected_by: Check::Canonicity,
        needs_padding: false,
        apply: zero_edge_length,
    },
    RejectionCase {
        label: "edge length above its height",
        family: Family::Canonicity,
        detected_by: Check::Canonicity,
        needs_padding: false,
        apply: overlong_edge_length,
    },
    RejectionCase {
        label: "edge path bit at or above ℓ",
        family: Family::Canonicity,
        detected_by: Check::Canonicity,
        needs_padding: false,
        apply: edge_path_above_length,
    },
    RejectionCase {
        label: "edge bottom claims to be an edge",
        family: Family::Canonicity,
        detected_by: Check::Canonicity,
        needs_padding: false,
        apply: edge_bottom_claims_edge,
    },
    RejectionCase {
        label: "edge bottom claims opaque above height 0",
        family: Family::Canonicity,
        detected_by: Check::Canonicity,
        needs_padding: false,
        apply: edge_bottom_claims_tall_opaque,
    },
    RejectionCase {
        label: "leaf unit migrated into the sibling class",
        family: Family::KindConfusion,
        detected_by: Check::KindTag,
        needs_padding: false,
        apply: migrate_leaf_into_siblings,
    },
    RejectionCase {
        label: "sibling unit dropped",
        family: Family::MultisetBalance,
        detected_by: Check::SiblingCount,
        needs_padding: false,
        apply: drop_sibling_unit,
    },
    RejectionCase {
        label: "sibling unit duplicated",
        family: Family::MultisetBalance,
        detected_by: Check::SiblingCount,
        needs_padding: false,
        apply: duplicate_sibling_unit,
    },
    RejectionCase {
        label: "edge slot dropped",
        family: Family::MultisetBalance,
        detected_by: Check::Multiset,
        needs_padding: false,
        apply: drop_edge_slot,
    },
    RejectionCase {
        label: "edge slot duplicated",
        family: Family::MultisetBalance,
        detected_by: Check::Multiset,
        needs_padding: false,
        apply: duplicate_edge_slot,
    },
    RejectionCase {
        label: "padded binary slot fed a live child",
        family: Family::LivePadding,
        detected_by: Check::Padding,
        needs_padding: true,
        apply: half_padded_binary_slot,
    },
    RejectionCase {
        label: "claimed root hash substituted",
        family: Family::RootBinding,
        detected_by: Check::Root,
        needs_padding: false,
        apply: substitute_root_hash,
    },
];

/// Decides whether a witness is rejected. **This is the circuit seam** — see the module docs.
pub fn oracle(witness: &SkeletonWitness) -> Result<(), Violation> {
    witness_invariants(witness)
}

fn relabel_edge_as_binary(witness: &mut SkeletonWitness) {
    let out = witness.edges[0].out;
    retag(witness, &out, SkeletonUnit { kind: SkeletonKind::Binary, ..out });
}

fn sibling_claims_leaf(witness: &mut SkeletonWitness) {
    let unit = pick_sibling(witness, |unit| unit.height > 0);
    retag(witness, &unit, SkeletonUnit { kind: SkeletonKind::Leaf, ..unit });
}

fn swap_leaf_with_sibling(witness: &mut SkeletonWitness) {
    let index = witness
        .siblings
        .iter()
        .position(|unit| !unit.is_inert() && unit.kind != SkeletonKind::Leaf)
        .expect("no non-leaf sibling to swap with");
    std::mem::swap(&mut witness.leaves[0], &mut witness.siblings[index]);
}

/// Flips path bit 128 — far below the height bound, so only the parent's position relation breaks.
fn substitute_leaf_path(witness: &mut SkeletonWitness) {
    let leaf = witness.leaves[0];
    let mut path = leaf.path;
    path[4] ^= 1;
    retag(witness, &leaf, SkeletonUnit { path, ..leaf });
}

fn perturb_binary_height(witness: &mut SkeletonWitness) {
    let out = witness.binaries[0].out;
    retag(witness, &out, SkeletonUnit { height: out.height + 1, ..out });
}

fn corrupt_sibling_hash(witness: &mut SkeletonWitness) {
    let unit = pick_sibling(witness, |_| true);
    let mut hash = unit.hash;
    hash[0] ^= 1;
    retag(witness, &unit, SkeletonUnit { hash, ..unit });
}

/// Zeroes a present leaf's value in the slot *and* its consumer, so the multiset stays balanced:
/// what rejects it is the consumer holding an empty-hash unit (no empty node).
fn zero_leaf_value(witness: &mut SkeletonWitness) {
    let leaf = witness.leaves[0];
    assert!(leaf.hash != EMPTY_HASH, "leaf 0 is absent; the fixture must lead with present keys");
    retag(witness, &leaf, SkeletonUnit { hash: EMPTY_HASH, ..leaf });
}

/// Zeroes a present leaf's value in its slot only — a false absence claim (P4). The slot's
/// contribution turns inert while the parent still consumes the original unit, which now dangles.
fn zero_leaf_value_slot_only(witness: &mut SkeletonWitness) {
    let leaf = &mut witness.leaves[0];
    assert!(leaf.hash != EMPTY_HASH, "leaf 0 is absent; the fixture must lead with present keys");
    leaf.hash = EMPTY_HASH;
}

/// Gives an absent leaf slot a non-zero value — a false presence claim (P4). The slot's
/// contribution turns live and nothing in the trie flow consumes it.
fn fake_present_leaf(witness: &mut SkeletonWitness) {
    let slot = witness
        .leaves
        .iter_mut()
        .find(|unit| unit.hash == EMPTY_HASH)
        .expect("no absent leaf slot; use a fixture with absent keys");
    slot.hash = [0xdead_beef; 8];
}

/// Claims the empty root over a live skeleton (P5): `root = 0` must force everything inert.
fn claim_empty_root(witness: &mut SkeletonWitness) {
    assert!(witness.root != EMPTY_HASH, "the fixture is already empty");
    witness.root = EMPTY_HASH;
}

fn zero_edge_length(witness: &mut SkeletonWitness) {
    witness.edges[0].length = 0;
}

fn overlong_edge_length(witness: &mut SkeletonWitness) {
    witness.edges[0].length = witness.edges[0].out.height + 1;
}

fn edge_path_above_length(witness: &mut SkeletonWitness) {
    let slot = &mut witness.edges[0];
    let bit = slot.length;
    slot.edge_path[(bit / 32) as usize] |= 1 << (bit % 32);
}

/// A slot-only modification: consistent retagging would first trip the producer's derived kind,
/// attributing away from the edge rule this case targets. The slot's local canonicity check runs
/// before the multiset, so the attribution lands where aimed.
fn edge_bottom_claims_edge(witness: &mut SkeletonWitness) {
    let slot = tall_bottom_edge(witness);
    slot.bottom.kind = SkeletonKind::Edge;
}

/// The P7 rule: an Opaque unit may sit under an edge only at height 0. Slot-only, as above.
fn edge_bottom_claims_tall_opaque(witness: &mut SkeletonWitness) {
    let slot = tall_bottom_edge(witness);
    slot.bottom.kind = SkeletonKind::Opaque;
}

/// A `Leaf`-tagged unit moved into the sibling class — the same-kind class migration that was
/// undetectable before P7 gave siblings their own tag.
fn migrate_leaf_into_siblings(witness: &mut SkeletonWitness) {
    let leaf = witness.leaves[0];
    assert!(leaf.hash != EMPTY_HASH, "leaf 0 is absent; the fixture must lead with present keys");
    witness.siblings.push(leaf);
    witness.leaves[0] = SkeletonUnit { hash: EMPTY_HASH, ..leaf };
}

/// The first edge slot whose live bottom sits above height 0.
fn tall_bottom_edge(witness: &mut SkeletonWitness) -> &mut EdgeSlot {
    witness
        .edges
        .iter_mut()
        .find(|slot| !slot.out.is_inert() && slot.bottom.height > 0)
        .expect("no edge slot with a bottom above height 0")
}

fn drop_sibling_unit(witness: &mut SkeletonWitness) {
    witness.siblings.remove(0);
}

fn duplicate_sibling_unit(witness: &mut SkeletonWitness) {
    witness.siblings.push(witness.siblings[0]);
}

fn drop_edge_slot(witness: &mut SkeletonWitness) {
    assert!(witness.edges.len() >= 2, "slot 0 would be the root slot");
    witness.edges.remove(0);
}

fn duplicate_edge_slot(witness: &mut SkeletonWitness) {
    assert!(witness.edges.len() >= 2, "slot 0 would be the root slot");
    witness.edges.push(witness.edges[0]);
}

fn half_padded_binary_slot(witness: &mut SkeletonWitness) {
    let live = witness.leaves[0];
    assert!(!live.is_inert(), "no live leaf to smuggle in");
    let slot = witness.binaries.last_mut().expect("no binary slots");
    assert!(slot.out.is_inert(), "the last binary slot is live; pad the witness first");
    slot.left = live;
}

fn substitute_root_hash(witness: &mut SkeletonWitness) {
    witness.root[0] ^= 1;
}

/// The first live sibling matching `wanted`.
fn pick_sibling(witness: &SkeletonWitness, wanted: fn(&SkeletonUnit) -> bool) -> SkeletonUnit {
    *witness
        .siblings
        .iter()
        .find(|unit| !unit.is_inert() && wanted(unit))
        .expect("no live sibling of the required shape")
}

/// Rewrites every copy of `old` — its producer's class vector and every slot consuming it — to
/// `new`, so the multiset stays balanced and a local check is what rejects the witness. Panics if
/// `old` is not both produced and consumed (the root unit is produced only, so it is off limits).
fn retag(witness: &mut SkeletonWitness, old: &SkeletonUnit, new: SkeletonUnit) {
    let mut hits = 0;
    for unit in units_mut(witness) {
        if unit == old {
            *unit = new;
            hits += 1;
        }
    }
    assert!(hits >= 2, "unit {old:?} appears {hits} time(s); expected a producer and a consumer");
}

fn units_mut(witness: &mut SkeletonWitness) -> impl Iterator<Item = &mut SkeletonUnit> {
    let SkeletonWitness { leaves, siblings, binaries, edges, .. } = witness;
    let binary_units = binaries.iter_mut().flat_map(binary_slot_units);
    let edge_units = edges.iter_mut().flat_map(edge_slot_units);
    leaves.iter_mut().chain(siblings.iter_mut()).chain(binary_units).chain(edge_units)
}

fn binary_slot_units(slot: &mut BinarySlot) -> [&mut SkeletonUnit; 3] {
    [&mut slot.out, &mut slot.left, &mut slot.right]
}

fn edge_slot_units(slot: &mut EdgeSlot) -> [&mut SkeletonUnit; 2] {
    [&mut slot.out, &mut slot.bottom]
}
