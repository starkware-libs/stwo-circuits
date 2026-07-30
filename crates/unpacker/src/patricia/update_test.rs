use std::collections::BTreeMap;

use rstest::rstest;

use super::rejection::UPDATE_REJECTION_CASES;
use super::{
    UpdateRow, UpdateWitness, extract_update_skeletons, rows_of, update_covering, update_invariants,
};
use crate::patricia::reference::{Word256, apply_updates, build_trie, root_hash};
use crate::patricia::skeleton::{SkeletonKind, SkeletonUnit};
use crate::patricia::slot_count_test::{HEIGHT, key_of, trie_of};

const N_ENTRIES: u64 = 300;

fn entries() -> BTreeMap<Word256, Word256> {
    trie_of(N_ENTRIES)
}

fn value(i: u64) -> Word256 {
    std::array::from_fn(|k| 0x2000_0000 + (i as u32) * 8 + k as u32)
}

/// A mixed batch: overwrites of present keys, inserts of absent keys, deletes of present keys,
/// and `(0, 0)` no-ops on keys absent from the trie.
fn mixed_rows(overwrites: u64, inserts: u64, deletes: u64, noops: u64) -> Vec<UpdateRow> {
    let entries = entries();
    let mut rows = Vec::new();
    for i in 0..overwrites {
        let key = key_of(i);
        rows.push(UpdateRow { key, prev_value: entries[&key], new_value: value(i) });
    }
    for i in 0..inserts {
        let key = key_of(N_ENTRIES + i);
        rows.push(UpdateRow { key, prev_value: [0; 8], new_value: value(1000 + i) });
    }
    for i in 0..deletes {
        let key = key_of(overwrites + i);
        rows.push(UpdateRow { key, prev_value: entries[&key], new_value: [0; 8] });
    }
    for i in 0..noops {
        let key = key_of(N_ENTRIES + inserts + i);
        rows.push(UpdateRow { key, prev_value: [0; 8], new_value: [0; 8] });
    }
    rows
}

fn extract(rows: &[UpdateRow]) -> UpdateWitness {
    extract_update_skeletons(&entries(), HEIGHT, rows).0
}

#[rstest]
#[case(4, 0, 0, 0)]
#[case(0, 4, 0, 0)]
#[case(0, 0, 4, 0)]
#[case(0, 0, 0, 3)]
#[case(3, 2, 2, 1)]
#[case(1, 0, 0, 0)]
fn extraction_satisfies_the_invariants(
    #[case] overwrites: u64,
    #[case] inserts: u64,
    #[case] deletes: u64,
    #[case] noops: u64,
) {
    let rows = mixed_rows(overwrites, inserts, deletes, noops);
    let witness = extract(&rows);
    update_invariants(&witness).expect("extracted update witness must be valid");
    assert!(rows_of(&witness) == rows);
    // Padding to the common budget preserves everything.
    let capacity = update_covering(&witness);
    let padded =
        UpdateWitness { prev: witness.prev.padded(&capacity), new: witness.new.padded(&capacity) };
    update_invariants(&padded).expect("padded update witness must be valid");
}

#[test]
fn the_new_root_matches_the_reference() {
    let rows = mixed_rows(3, 2, 2, 1);
    let (witness, new_entries) = extract_update_skeletons(&entries(), HEIGHT, &rows);
    let updates: Vec<(Word256, Word256)> =
        rows.iter().map(|row| (row.key, row.new_value)).collect();
    let expected = apply_updates(&entries(), &updates);
    assert!(new_entries == expected);
    assert!(witness.new.root == root_hash(&build_trie(&expected, HEIGHT)));
    assert!(witness.prev.root == root_hash(&build_trie(&entries(), HEIGHT)));
}

/// The nonce-trie genesis shape: inserts into the empty trie. The prev side is entirely inert
/// under P5 and there are no untouched subtrees to share.
#[test]
fn insert_into_the_empty_trie() {
    let rows: Vec<UpdateRow> = (0..3)
        .map(|i| UpdateRow { key: key_of(i), prev_value: [0; 8], new_value: value(i) })
        .collect();
    let (witness, _) = extract_update_skeletons(&BTreeMap::new(), HEIGHT, &rows);
    assert!(witness.prev.root == [0; 8]);
    assert!(witness.prev.binaries.is_empty() && witness.prev.edges.is_empty());
    assert!(witness.siblings_shared().is_empty());
    update_invariants(&witness).unwrap();
}

/// The mirror image: deleting every key empties the trie.
#[test]
fn delete_everything_empties_the_trie() {
    let small: BTreeMap<Word256, Word256> = (0..4).map(|i| (key_of(i), value(i))).collect();
    let rows: Vec<UpdateRow> = small
        .iter()
        .map(|(key, value)| UpdateRow { key: *key, prev_value: *value, new_value: [0; 8] })
        .collect();
    let (witness, new_entries) = extract_update_skeletons(&small, HEIGHT, &rows);
    assert!(new_entries.is_empty());
    assert!(witness.new.root == [0; 8]);
    assert!(witness.siblings_shared().is_empty());
    update_invariants(&witness).unwrap();
}

/// Reconciliation in the delete-merge shape: after a delete, the new side's merged edge reaches
/// through what the prev side saw as a sibling, so extraction opens the prev side until both
/// frontiers agree — pinned here by the shared-list equality plus both sides' invariants.
#[test]
fn delete_merge_reconciles_the_frontiers() {
    // Two keys differing high up, so deleting one merges a long edge over the other's subtree.
    let small: BTreeMap<Word256, Word256> = (0..6).map(|i| (key_of(i), value(i))).collect();
    for target in 0..6 {
        let rows = vec![UpdateRow {
            key: key_of(target),
            prev_value: small[&key_of(target)],
            new_value: [0; 8],
        }];
        let (witness, _) = extract_update_skeletons(&small, HEIGHT, &rows);
        update_invariants(&witness)
            .unwrap_or_else(|violation| panic!("delete of key {target}: {violation:?}"));
    }
}

/// A `(0, 0)` row over a key that is *present* (and unchanged) in both tries is accepted — it
/// proves "unchanged", never absence (P9). This pins the open-gap ledger row.
#[test]
fn a_no_op_row_can_hide_a_present_unchanged_key() {
    let rows = mixed_rows(2, 0, 0, 0);
    let mut witness = extract(&rows);
    let hidden = key_of(50); // present in the trie, untouched by the batch
    let slot = SkeletonUnit { height: 0, path: hidden, kind: SkeletonKind::Leaf, hash: [0; 8] };
    witness.prev.leaves.push(slot);
    witness.new.leaves.push(slot);
    update_invariants(&witness).expect("the (0,0) claim over a present key is accepted (P9)");
}

/// Every update rejection case is rejected by the exact [`UpdateCheck`] it targets.
#[test]
fn every_update_rejection_case_is_rejected_by_the_check_it_targets() {
    let base = extract(&mixed_rows(3, 2, 2, 1));
    super::rejection::oracle(&base).expect("the fixture must be valid");
    for case in UPDATE_REJECTION_CASES {
        let mut witness = base.clone();
        (case.apply)(&mut witness);
        let violation = super::rejection::oracle(&witness)
            .expect_err(&format!("update rejection case '{}' was accepted", case.label));
        assert!(
            violation.check == case.detected_by,
            "case '{}' targets {:?} but was rejected by {:?}: {}",
            case.label,
            case.detected_by,
            violation.check,
            violation.detail
        );
    }
}

impl UpdateWitness {
    /// The shared sibling list (equal on both sides by construction).
    fn siblings_shared(&self) -> &[SkeletonUnit] {
        assert!(self.prev.siblings == self.new.siblings);
        &self.prev.siblings
    }
}
