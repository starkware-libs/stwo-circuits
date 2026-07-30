//! `verify_patricia_update` — the in-circuit Patricia update verifier (design doc §5, step 2).
//!
//! # Statement
//!
//! A satisfying assignment proves, for the returned `prev_root`/`new_root` (caller-bound) and the
//! `K` returned rows: **given that `prev_root` commits to a canonical trie** (genesis plus the
//! induction this very statement supports — every certified `new_root` is canonical), there is a
//! canonical trie `T_new` with `hash(T_new) = new_root` such that
//!
//! 1. per live row (`(prev_value, new_value) ≠ (0, 0)`), **given the caller's precondition that no
//!    two live rows share a key**: the key maps to `prev_value` in the prev trie and to `new_value`
//!    in `T_new`, `0` meaning **proven absent** — inserts and deletes included (see the closing
//!    argument below and the distinctness bullet under *Deliberately unproven*);
//! 2. off the row keys, `T_new` agrees with the prev trie everywhere — both folds consume the
//!    *same* shared sibling units (P6), so untouched state is byte-identical;
//! 3. a `(0, 0)` row proves only that its key is *unchanged* between the tries — possibly present
//!    with the same value — never that it is absent (P9).
//!
//! Presence flags are derived per side (`value ≠ 0`, P4), emptiness per root (P5); the four row
//! operations — overwrite, insert, delete, no-op — are witness shapes, not circuit cases.
//!
//! # Why false absence dies (the closing argument)
//!
//! Within one fold, two live units at the same position can never both be consumed: consumers
//! take children at *sibling* positions, so duplicate-position units force duplicate parent
//! chains that both need the single root entry — one dangles, the multiset fails. Now claim a
//! present key absent (`prev_value = 0`) and insert it: the real leaf hides behind some shared
//! sibling on the key's path; the new fold must consume both that sibling *and* the inserted
//! leaf's ancestor chain through the same positions — rejected. Deletes are symmetric: the prev
//! fold walks to the key, excluding any sibling on its path, so the new side's divergence is
//! genuine. A fake divergence is excluded by hash binding (with the *shifted*-collision caveat
//! for the additive edge length — design doc §6 Q3).
//!
//! # Deliberately unproven
//!
//! * `(0, 0)` rows prove "unchanged", never absence — no caller may lean on them as non-membership
//!   (open-gap ledger).
//! * A both-sides-absent row's key is unbound; the caller owes every key binding.
//! * **Live-key distinctness is the caller's precondition**, not a circuit guarantee. Two
//!   *same-side* duplicate live rows are rejected (two live units at one position in one fold), but
//!   the complementary pair — `(k, v, 0)` plus `(k, 0, w)` — is accepted and jointly behaves as the
//!   overwrite `v → w` while each row's absence claim is false. The absence clauses of the
//!   statement hold only for batches with distinct live keys (the dict squashing emits one row per
//!   key by construction; nonce batches are insert-only). A live row plus a `(0, 0)` row on one key
//!   are independent.
//!
//! # Witness table (condensed — what is new over two skeleton instances)
//!
//! | witness | width | determined by |
//! |---|---|---|
//! | shared sibling units | (u16 + 16·u16 + 8·u32) · S | consumed by **both** folds; hash pinned through the prev chain from `prev_root`; kind derived (P7) |
//! | per-row key (shared vars) | 8·u32 | the live side's leaf cancellation; unbound when absent on both sides |
//! | per-row `prev_value` / `new_value` | 8·u32 each | the respective fold's leaf cancellation; `0` pins the derived flag |
//! | everything inside each fold | — | step 1's witness table, per instance |

use circuits::blake::HashValue;
use circuits::context::{Context, Var};
use circuits::ivalue::IValue;
use circuits::wrappers::U32Wrapper;

use super::skeleton::SkeletonCapacity;
use super::skeleton_circuit::{SkeletonUnitVars, guess_word256, sibling_slot, skeleton_flow};
use super::update::UpdateWitness;
use crate::permutation::permute_units;

#[cfg(test)]
#[path = "update_circuit_test.rs"]
mod test;

/// One verified row: the shared key vars, both sides' values, and the derived presence flags
/// (`was_present ⟺ prev_value ≠ 0`, `is_present ⟺ new_value ≠ 0`).
pub struct UpdateRowVars {
    pub key: HashValue<Var>,
    pub prev_value: HashValue<Var>,
    pub new_value: HashValue<Var>,
    pub was_present: Var,
    pub is_present: Var,
}

/// Output of [`verify_patricia_update`]: the two roots for the caller's public-input bindings and
/// the verified rows for the caller's batch binding.
pub struct VerifiedPatriciaUpdate {
    pub prev_root: HashValue<Var>,
    pub new_root: HashValue<Var>,
    pub rows: Vec<UpdateRowVars>,
}

/// Builds the update-verifying circuit over `witness`, both sides padded to `capacity`. See the
/// module docs for the statement. The topology depends only on `(height, capacity)`.
///
/// Panics on a malformed witness — misaligned rows, diverging sibling lists, a capacity the
/// witness does not fit, or non-permutable multisets; every other defect is a failed constraint.
pub fn verify_patricia_update<Value: IValue>(
    ctx: &mut Context<Value>,
    witness: &UpdateWitness,
    capacity: &SkeletonCapacity,
) -> VerifiedPatriciaUpdate {
    assert!(witness.prev.height == witness.new.height, "the sides must share the trie height");
    assert!(witness.prev.siblings == witness.new.siblings, "the sibling list is shared (P6)");
    assert!(
        witness.prev.leaves.len() == witness.new.leaves.len()
            && witness
                .prev
                .leaves
                .iter()
                .zip(&witness.new.leaves)
                .all(|(prev, new)| prev.path == new.path),
        "the rows must be aligned across the sides"
    );
    let prev = witness.prev.padded(capacity);
    let new = witness.new.padded(capacity);

    // Shared guesses (P6): one key per row, one unit per sibling slot — both folds consume the
    // same vars, so cross-side divergence is inexpressible.
    let keys: Vec<[U32Wrapper<Var>; 8]> =
        prev.leaves.iter().map(|unit| guess_word256(ctx, &unit.path)).collect();
    let siblings: Vec<SkeletonUnitVars> =
        prev.siblings.iter().map(|unit| sibling_slot(ctx, unit)).collect();

    let prev_flow = skeleton_flow(ctx, &prev, &keys, &siblings);
    let new_flow = skeleton_flow(ctx, &new, &keys, &siblings);
    permute_units(ctx, &prev_flow.consumed, &prev_flow.produced);
    permute_units(ctx, &new_flow.consumed, &new_flow.produced);

    let rows = prev_flow
        .leaves
        .into_iter()
        .zip(new_flow.leaves)
        .map(|(prev_leaf, new_leaf)| UpdateRowVars {
            key: prev_leaf.key,
            prev_value: prev_leaf.value,
            new_value: new_leaf.value,
            was_present: prev_leaf.is_present,
            is_present: new_leaf.is_present,
        })
        .collect();
    VerifiedPatriciaUpdate { prev_root: prev_flow.root, new_root: new_flow.root, rows }
}
