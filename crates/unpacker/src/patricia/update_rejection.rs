//! RejectionCase harness for the update witness, extending the skeleton catalogue to the
//! update-level surface: the shared sibling list, the row alignment, the two root bindings, and
//! the cross-side value flow. Per-side slot defects are already covered by the skeleton
//! catalogue — both seams run it against each side through [`update_invariants`].
//!
//! Attribution follows the same two-seam rule as `skeleton_rejection`: `update_test` asserts the
//! exact [`UpdateCheck`]; `update_circuit_test` runs every row through `verify_patricia_update`
//! end to end.

use super::{UpdateCheck, UpdateViolation, UpdateWitness, update_invariants};
use crate::patricia::skeleton::{Check, SkeletonUnit};

/// One row of the update rejection table.
pub struct UpdateRejectionCase {
    pub label: &'static str,
    /// The invariant expected to reject it. Asserted exactly, for attribution.
    pub detected_by: UpdateCheck,
    pub apply: fn(&mut UpdateWitness),
}

/// The catalogue. Applied to a mixed fixture (overwrites, inserts, deletes, a no-op) so every
/// shape is on hand.
pub const UPDATE_REJECTION_CASES: &[UpdateRejectionCase] = &[
    UpdateRejectionCase {
        label: "shared sibling hash modified everywhere",
        detected_by: UpdateCheck::Prev(Check::Hashing),
        apply: modify_shared_sibling,
    },
    UpdateRejectionCase {
        label: "sibling modified in one side's list only",
        detected_by: UpdateCheck::SharedSiblings,
        apply: modify_one_sided_sibling,
    },
    UpdateRejectionCase {
        label: "rows swapped on one side only",
        detected_by: UpdateCheck::Rows,
        apply: swap_rows_on_one_side,
    },
    UpdateRejectionCase {
        label: "unbacked write: new value zeroed in its slot only",
        detected_by: UpdateCheck::New(Check::SiblingCount),
        apply: zero_new_value_slot_only,
    },
    UpdateRejectionCase {
        label: "dropped write: new value substituted in its slot only",
        detected_by: UpdateCheck::New(Check::Multiset),
        apply: substitute_new_value_slot_only,
    },
    UpdateRejectionCase {
        label: "false absence: present prev value zeroed in its slot only",
        detected_by: UpdateCheck::Prev(Check::SiblingCount),
        apply: zero_prev_value_slot_only,
    },
    UpdateRejectionCase {
        label: "fake divergence: edge path bit flipped below ℓ",
        detected_by: UpdateCheck::New(Check::Position),
        apply: flip_edge_path_bit,
    },
    UpdateRejectionCase {
        label: "row rebound to a different key on both sides",
        detected_by: UpdateCheck::Prev(Check::Multiset),
        apply: rebind_row_key,
    },
    UpdateRejectionCase {
        label: "new root claimed empty over a live skeleton",
        detected_by: UpdateCheck::New(Check::Root),
        apply: claim_empty_new_root,
    },
    UpdateRejectionCase {
        label: "prev root substituted",
        detected_by: UpdateCheck::Prev(Check::Root),
        apply: substitute_prev_root,
    },
];

/// Decides whether an update witness is rejected — the out-of-circuit seam.
pub fn oracle(witness: &UpdateWitness) -> Result<(), UpdateViolation> {
    update_invariants(witness)
}

/// Rewrites every copy of a live shared sibling's hash — the shared list on both sides and every
/// slot consuming it — so only the recomputed hash chain can object.
fn modify_shared_sibling(witness: &mut UpdateWitness) {
    let old = *witness
        .prev
        .siblings
        .iter()
        .find(|unit| !unit.is_inert())
        .expect("no live shared sibling; use a fixture with untouched subtrees");
    let mut new = old;
    new.hash[0] ^= 1;
    for side in [&mut witness.prev, &mut witness.new] {
        for unit in side.siblings.iter_mut() {
            if *unit == old {
                *unit = new;
            }
        }
        for slot in side.binaries.iter_mut() {
            for unit in [&mut slot.left, &mut slot.right] {
                if *unit == old {
                    *unit = new;
                }
            }
        }
        for slot in side.edges.iter_mut() {
            if slot.bottom == old {
                slot.bottom = new;
            }
        }
    }
}

fn modify_one_sided_sibling(witness: &mut UpdateWitness) {
    let unit = witness
        .new
        .siblings
        .iter_mut()
        .find(|unit| !unit.is_inert())
        .expect("no live shared sibling");
    unit.hash[1] ^= 1;
}

fn swap_rows_on_one_side(witness: &mut UpdateWitness) {
    assert!(witness.new.leaves[0].path != witness.new.leaves[1].path);
    witness.new.leaves.swap(0, 1);
}

/// A row that is present on the new side, for the value-flow cases.
fn present_new_row(witness: &mut UpdateWitness) -> &mut SkeletonUnit {
    witness
        .new
        .leaves
        .iter_mut()
        .find(|unit| unit.hash != [0; 8])
        .expect("no row present on the new side")
}

fn zero_new_value_slot_only(witness: &mut UpdateWitness) {
    present_new_row(witness).hash = [0; 8];
}

fn substitute_new_value_slot_only(witness: &mut UpdateWitness) {
    present_new_row(witness).hash[0] ^= 1;
}

fn zero_prev_value_slot_only(witness: &mut UpdateWitness) {
    let slot = witness
        .prev
        .leaves
        .iter_mut()
        .find(|unit| unit.hash != [0; 8])
        .expect("no row present on the prev side");
    slot.hash = [0; 8];
}

/// Flips a compressed-path bit *below* ℓ of a live new-side edge, so alignment still holds and
/// only the position relation objects.
fn flip_edge_path_bit(witness: &mut UpdateWitness) {
    let slot = witness
        .new
        .edges
        .iter_mut()
        .find(|slot| !slot.out.is_inert() && slot.length >= 1)
        .expect("no live new-side edge");
    slot.edge_path[0] ^= 1;
}

/// Moves a present row's key on both sides consistently (alignment holds); the leaf units then
/// cancel against nothing at the substituted position.
fn rebind_row_key(witness: &mut UpdateWitness) {
    let index = witness
        .prev
        .leaves
        .iter()
        .position(|unit| unit.hash != [0; 8])
        .expect("no row present on the prev side");
    for side in [&mut witness.prev, &mut witness.new] {
        side.leaves[index].path[3] ^= 1 << 7;
    }
}

fn claim_empty_new_root(witness: &mut UpdateWitness) {
    assert!(witness.new.root != [0; 8]);
    witness.new.root = [0; 8];
}

fn substitute_prev_root(witness: &mut UpdateWitness) {
    witness.prev.root[0] ^= 1;
}
