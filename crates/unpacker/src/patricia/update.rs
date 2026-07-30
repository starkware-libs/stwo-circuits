//! Witness extraction and invariants for the Patricia *update* circuit (design doc §5, step 2).
//!
//! An update witness is two skeletons — prev and new — over one shared sibling list (P6) and one
//! batch of rows `(key, prev_value, new_value)`, leaf slots 1:1 with the rows on both sides. The
//! circuit-side mirror is
//! [`verify_patricia_update`](super::update_circuit::verify_patricia_update).
//!
//! # Row semantics (P4/P9)
//!
//! Values of `0` mean absent, so the four row shapes are the four operations: overwrite
//! (`v → v′`), insert (`0 → v′`), delete (`v → 0`), and the no-op `(0, 0)` — which proves only
//! "unchanged between the tries", never absence (P9; see the open-gap ledger). Live rows (all but
//! `(0, 0)`) are *walked* on both sides; `(0, 0)` rows get absent leaf slots on both sides and no
//! walk.
//!
//! # The shared frontier
//!
//! Both skeletons must present the same sibling multiset, but naive per-side extraction does not:
//! an insert's edge split or a delete's edge merge leaves one side's natural frontier coarser than
//! the other's. [`extract_update_skeletons`] reconciles by *opening* the coarse side — replacing a
//! sibling with the slot(s) of the node at its position and pushing the frontier one node deeper —
//! until the two position sets coincide. Off the live keys' paths the two tries hold *identical*
//! subtrees (canonical uniqueness), so the reconciled units agree exactly; the loop terminates
//! because every opening strictly descends. P7's rule (no opaque edge bottom above height 0) is
//! applied by the same opening machinery.

use std::collections::BTreeMap;

use super::reference::{EMPTY_HASH, PatriciaTree, Word256, build_trie, root_hash, word256_bit};
use super::skeleton::{
    BinarySlot, Check, EdgeSlot, SkeletonKind, SkeletonUnit, SkeletonWitness, extract_skeleton,
    witness_invariants,
};

#[cfg(test)]
#[path = "update_test.rs"]
mod test;

#[cfg(test)]
#[path = "update_rejection.rs"]
pub mod rejection;

/// One batch write: `0` values mean absent (P4), so `(0, v)` inserts, `(v, 0)` deletes, and
/// `(0, 0)` is the unchanged no-op (P9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpdateRow {
    pub key: Word256,
    pub prev_value: Word256,
    pub new_value: Word256,
}

/// The update circuit's witness: two skeletons over one shared sibling list. `prev.siblings` and
/// `new.siblings` are the *same* list (asserted, and guessed once in-circuit per P6); the leaf
/// slots are 1:1 with the batch rows, in row order, on both sides.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateWitness {
    pub prev: SkeletonWitness,
    pub new: SkeletonWitness,
}

/// Where an update-witness defect lives: one side's skeleton invariants, the row alignment
/// between the sides, or the shared sibling list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateCheck {
    Prev(Check),
    New(Check),
    Rows,
    SharedSiblings,
}

/// A rejected update witness, tagged for attribution like the skeleton's
/// [`Violation`](super::skeleton::Violation).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateViolation {
    pub check: UpdateCheck,
    pub detail: String,
}

/// Extracts the update witness for applying `rows` to the trie over `prev_entries`. Returns the
/// witness together with the resulting entry map (whose trie commits to `witness.new.root`).
///
/// Panics if row keys repeat, if a row's `prev_value` disagrees with `prev_entries`, or if a key
/// has bits at or above `height` — witness generation is for honest inputs; rejected witnesses
/// are built by modifying honest ones.
pub fn extract_update_skeletons(
    prev_entries: &BTreeMap<Word256, Word256>,
    height: u32,
    rows: &[UpdateRow],
) -> (UpdateWitness, BTreeMap<Word256, Word256>) {
    for row in rows {
        let expected = prev_entries.get(&row.key).copied().unwrap_or(EMPTY_HASH);
        assert!(row.prev_value == expected, "row {row:?} disagrees with the prev trie");
    }
    let mut new_entries = prev_entries.clone();
    for row in rows {
        if row.new_value == EMPTY_HASH {
            new_entries.remove(&row.key);
        } else {
            new_entries.insert(row.key, row.new_value);
        }
    }
    let prev_tree = build_trie(prev_entries, height);
    let new_tree = build_trie(&new_entries, height);

    // Walk each side over the live keys only; (0,0) rows get absent slots on both sides.
    let live_keys: Vec<Word256> = rows
        .iter()
        .filter(|row| (row.prev_value, row.new_value) != (EMPTY_HASH, EMPTY_HASH))
        .map(|row| row.key)
        .collect();
    let mut prev = extract_skeleton(prev_tree.as_ref(), height, &live_keys);
    let mut new = extract_skeleton(new_tree.as_ref(), height, &live_keys);
    reconcile_frontiers(&mut prev, prev_tree.as_ref(), &mut new, new_tree.as_ref());

    // Rebuild the leaf slots 1:1 with the rows, in row order.
    let slot_of = |witness: &SkeletonWitness, key: &Word256, value: &Word256| {
        let walked = witness.leaves.iter().find(|unit| unit.path == *key).copied();
        walked.unwrap_or(SkeletonUnit {
            height: 0,
            path: *key,
            kind: SkeletonKind::Leaf,
            hash: *value,
        })
    };
    let prev_leaves = rows.iter().map(|row| slot_of(&prev, &row.key, &row.prev_value)).collect();
    let new_leaves = rows.iter().map(|row| slot_of(&new, &row.key, &row.new_value)).collect();
    prev.leaves = prev_leaves;
    new.leaves = new_leaves;

    assert!(prev.siblings == new.siblings, "reconciliation must equalize the frontiers");
    assert!(prev.root == root_hash(&prev_tree) && new.root == root_hash(&new_tree));
    (UpdateWitness { prev, new }, new_entries)
}

/// Everything a satisfying update assignment implies, checked out of circuit: both sides'
/// skeleton invariants, the 1:1 row alignment, and the shared sibling list. The order is part of
/// the contract, mirroring [`witness_invariants`].
pub fn update_invariants(witness: &UpdateWitness) -> Result<(), UpdateViolation> {
    if witness.prev.height != witness.new.height
        || witness.prev.leaves.len() != witness.new.leaves.len()
        || witness
            .prev
            .leaves
            .iter()
            .zip(&witness.new.leaves)
            .any(|(prev, new)| prev.path != new.path)
    {
        let detail = "the two sides' rows are not aligned".to_string();
        return Err(UpdateViolation { check: UpdateCheck::Rows, detail });
    }
    if witness.prev.siblings != witness.new.siblings {
        let detail = "the sibling lists differ between the sides".to_string();
        return Err(UpdateViolation { check: UpdateCheck::SharedSiblings, detail });
    }
    witness_invariants(&witness.prev).map_err(|violation| UpdateViolation {
        check: UpdateCheck::Prev(violation.check),
        detail: violation.detail,
    })?;
    witness_invariants(&witness.new).map_err(|violation| UpdateViolation {
        check: UpdateCheck::New(violation.check),
        detail: violation.detail,
    })
}

/// The tight common budget of an update witness: each side's covering budget, maximized — the
/// shared sibling count is exactly the derived `n_binary − K + 1` of the larger side.
pub fn update_covering(witness: &UpdateWitness) -> super::skeleton::SkeletonCapacity {
    let prev = super::skeleton::SkeletonCapacity::covering(&witness.prev);
    let new = super::skeleton::SkeletonCapacity::covering(&witness.new);
    super::skeleton::SkeletonCapacity {
        n_leaves: prev.n_leaves,
        n_binary: prev.n_binary.max(new.n_binary),
        n_edge: prev.n_edge.max(new.n_edge),
    }
}

/// The batch rows encoded in an update witness's aligned leaf slots.
pub fn rows_of(witness: &UpdateWitness) -> Vec<UpdateRow> {
    witness
        .prev
        .leaves
        .iter()
        .zip(&witness.new.leaves)
        .map(|(prev, new)| UpdateRow { key: prev.path, prev_value: prev.hash, new_value: new.hash })
        .collect()
}

/// Opens siblings on whichever side is coarser until the two sibling position sets coincide. See
/// the module docs; terminates because every opening strictly descends into finite trees.
fn reconcile_frontiers(
    prev: &mut SkeletonWitness,
    prev_tree: Option<&PatriciaTree>,
    new: &mut SkeletonWitness,
    new_tree: Option<&PatriciaTree>,
) {
    loop {
        let prev_positions: BTreeMap<(u32, Word256), SkeletonUnit> =
            prev.siblings.iter().map(|unit| ((unit.height, unit.path), *unit)).collect();
        let new_positions: BTreeMap<(u32, Word256), SkeletonUnit> =
            new.siblings.iter().map(|unit| ((unit.height, unit.path), *unit)).collect();
        // A sibling position present on exactly one side.
        let one_sided = prev_positions
            .keys()
            .find(|position| !new_positions.contains_key(*position))
            .map(|position| (*position, true))
            .or_else(|| {
                new_positions
                    .keys()
                    .find(|position| !prev_positions.contains_key(*position))
                    .map(|position| (*position, false))
            });
        let Some(((height, path), on_prev)) = one_sided else {
            // Openings land in per-side order; the shared list must be one list, so canonicalize.
            prev.siblings.sort();
            new.siblings.sort();
            return;
        };
        // The other side either has a sibling at a strict ancestor (it is coarser there — open
        // it), or its frontier near this position is already deeper (this side is the coarse one).
        let (mine, other, other_tree) = if on_prev {
            (&mut *prev, &mut *new, new_tree)
        } else {
            (&mut *new, &mut *prev, prev_tree)
        };
        let ancestor = other
            .siblings
            .iter()
            .position(|unit| {
                unit.height > height && key_prefix_of(&path, unit.height - height) == unit.path
            })
            .map(|index| (index, other, other_tree));
        let (index, side, tree) = ancestor.unwrap_or_else(|| {
            let my_tree = if on_prev { prev_tree } else { new_tree };
            let index = mine
                .siblings
                .iter()
                .position(|unit| unit.height == height && unit.path == path)
                .expect("the one-sided position came from this list");
            (index, mine, my_tree)
        });
        open_sibling(side, index, tree.expect("a sibling implies a non-empty trie"));
    }
}

/// Replaces `witness.siblings[index]` with the slots of the node at its position, pushing the
/// frontier one node deeper. Applies P7 to opened edges (their untouched bottoms above height 0
/// open one binary level). The opened unit's kind changes (Opaque → the node's slot kind), so
/// every slot that consumed the sibling is rewritten to the produced unit.
fn open_sibling(witness: &mut SkeletonWitness, index: usize, tree: &PatriciaTree) {
    let unit = witness.siblings.remove(index);
    let node = node_at(tree, witness.height, unit.height, &unit.path);
    let replacement = match node {
        PatriciaTree::Leaf { .. } => panic!("a height-0 sibling is never opened: {unit:?}"),
        PatriciaTree::Binary { .. } => open_binary(witness, node, unit.height, &unit.path),
        PatriciaTree::Edge { length, path: edge_path, bottom } => {
            let below_height = unit.height - length;
            let below_path = shifted_add(&unit.path, *length, edge_path);
            let bottom_unit = if below_height > 0 {
                open_binary(witness, bottom, below_height, &below_path)
            } else {
                let sibling = SkeletonUnit {
                    height: 0,
                    path: below_path,
                    kind: SkeletonKind::Opaque,
                    hash: bottom.hash(),
                };
                witness.siblings.push(sibling);
                sibling
            };
            let out = SkeletonUnit { kind: SkeletonKind::Edge, ..unit };
            witness.edges.push(EdgeSlot {
                out,
                bottom: bottom_unit,
                length: *length,
                edge_path: *edge_path,
            });
            out
        }
    };
    for slot in witness.binaries.iter_mut() {
        for child in [&mut slot.left, &mut slot.right] {
            if *child == unit {
                *child = replacement;
            }
        }
    }
    for slot in witness.edges.iter_mut() {
        if slot.bottom == unit {
            slot.bottom = replacement;
        }
    }
}

/// Emits a binary slot for `node` at `(height, path)` with both children as opaque siblings,
/// returning the produced unit.
fn open_binary(
    witness: &mut SkeletonWitness,
    node: &PatriciaTree,
    height: u32,
    path: &Word256,
) -> SkeletonUnit {
    let PatriciaTree::Binary { left, right } = node else {
        panic!("expected a binary at height {height} (canonical edge bottoms above 0 are binary)")
    };
    let child = |tree: &PatriciaTree, bit: u32| SkeletonUnit {
        height: height - 1,
        path: child_path_of(path, bit),
        kind: SkeletonKind::Opaque,
        hash: tree.hash(),
    };
    let left = child(left, 0);
    let right = child(right, 1);
    witness.siblings.push(left);
    witness.siblings.push(right);
    let out = SkeletonUnit { height, path: *path, kind: SkeletonKind::Binary, hash: node.hash() };
    witness.binaries.push(BinarySlot { out, left, right });
    out
}

/// The node of `tree` (height `tree_height`) at position `(height, path)`. Panics if the position
/// is not a node — reconciliation only opens positions that are.
fn node_at<'a>(
    tree: &'a PatriciaTree,
    tree_height: u32,
    height: u32,
    path: &Word256,
) -> &'a PatriciaTree {
    let mut node = tree;
    let mut node_height = tree_height;
    while node_height > height {
        match node {
            PatriciaTree::Leaf { .. } => panic!("no node at height {height}"),
            PatriciaTree::Binary { left, right } => {
                let bit = word256_bit(path, node_height - 1 - height);
                node = if bit == 0 { left } else { right };
                node_height -= 1;
            }
            PatriciaTree::Edge { length, bottom, .. } => {
                assert!(node_height - length >= height, "position is inside an edge run");
                node = bottom;
                node_height -= length;
            }
        }
    }
    node
}

/// `path >> shift` — the ancestor position `shift` levels up.
fn key_prefix_of(path: &Word256, shift: u32) -> Word256 {
    let mut out = [0u32; 8];
    for i in shift..256 {
        let bit = word256_bit(path, i);
        out[((i - shift) / 32) as usize] |= bit << ((i - shift) % 32);
    }
    out
}

/// `path · 2 + bit` — a child position.
fn child_path_of(path: &Word256, bit: u32) -> Word256 {
    let mut out = shifted_add(path, 1, &[0; 8]);
    out[0] |= bit;
    out
}

/// `path · 2^shift + addend` (bit-disjoint by construction here).
fn shifted_add(path: &Word256, shift: u32, addend: &Word256) -> Word256 {
    let mut out = [0u32; 8];
    for i in 0..256 - shift {
        let bit = word256_bit(path, i);
        out[((i + shift) / 32) as usize] |= bit << ((i + shift) % 32);
    }
    for (word, extra) in out.iter_mut().zip(addend) {
        *word |= extra;
    }
    out
}
