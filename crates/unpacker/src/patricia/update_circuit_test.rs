use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};

use circuits::circuit::Circuit;
use circuits::context::Context;
use circuits::ivalue::{IValue, NoValue};
use circuits::ops::eq;
use rstest::rstest;
use stwo::core::fields::qm31::QM31;

use super::{VerifiedPatriciaUpdate, verify_patricia_update};
use crate::fingerprint::circuit_fingerprint;
use crate::patricia::reference::Word256;
use crate::patricia::skeleton::{SkeletonCapacity, SkeletonKind, SkeletonUnit};
use crate::patricia::update::rejection::UPDATE_REJECTION_CASES;
use crate::patricia::update::{
    UpdateRow, UpdateWitness, extract_update_skeletons, update_covering,
};

const HEIGHT: u32 = 251;
const N_ENTRIES: u64 = 48;

/// Deterministic pseudorandom-ish 251-bit keys (no process randomness in tests).
fn key_of(i: u64) -> Word256 {
    let scrambled = i.wrapping_mul(0x9E37_79B9_7F4A_7C15).rotate_left(23) ^ (i << 40);
    let mut key = [0u32; 8];
    key[0] = scrambled as u32;
    key[2] = (scrambled >> 32) as u32;
    key[5] = (i as u32).wrapping_mul(0x0F0F_0F0F);
    key[7] = ((scrambled >> 17) as u32 ^ (i as u32)) & ((1 << 27) - 1);
    key
}

fn value(i: u64) -> Word256 {
    std::array::from_fn(|k| 0x3000_0000 + (i as u32) * 8 + k as u32)
}

fn entries() -> BTreeMap<Word256, Word256> {
    (0..N_ENTRIES).map(|i| (key_of(i), value(i))).collect()
}

/// Overwrites, then inserts, then deletes, then `(0, 0)` no-ops — all over disjoint keys.
fn mixed_rows(overwrites: u64, inserts: u64, deletes: u64, noops: u64) -> Vec<UpdateRow> {
    let entries = entries();
    let overwrite = (0..overwrites).map(|i| UpdateRow {
        key: key_of(i),
        prev_value: entries[&key_of(i)],
        new_value: value(500 + i),
    });
    let insert = (0..inserts).map(|i| UpdateRow {
        key: key_of(N_ENTRIES + i),
        prev_value: [0; 8],
        new_value: value(600 + i),
    });
    let delete = (0..deletes).map(|i| UpdateRow {
        key: key_of(overwrites + i),
        prev_value: entries[&key_of(overwrites + i)],
        new_value: [0; 8],
    });
    let noop = (0..noops).map(|i| UpdateRow {
        key: key_of(N_ENTRIES + inserts + i),
        prev_value: [0; 8],
        new_value: [0; 8],
    });
    overwrite.chain(insert).chain(delete).chain(noop).collect()
}

fn fixture_witness(rows: &[UpdateRow]) -> UpdateWitness {
    extract_update_skeletons(&entries(), HEIGHT, rows).0
}

/// Builds the circuit and binds both returned roots to the witness's claimed roots.
fn build<Value: IValue>(
    witness: &UpdateWitness,
    capacity: &SkeletonCapacity,
) -> (Context<Value>, VerifiedPatriciaUpdate) {
    let mut ctx = Context::default();
    let update = verify_patricia_update(&mut ctx, witness, capacity);
    let bindings = [(&update.prev_root, witness.prev.root), (&update.new_root, witness.new.root)];
    for (root, claimed) in bindings {
        for (word, value) in root.iter().zip(claimed) {
            let packed = ctx.constant(QM31::pack_u32(value));
            eq(&mut ctx, *word.get(), packed);
        }
    }
    (ctx, update)
}

/// Full-pipeline acceptance; a build-time panic (an unhostable witness) counts as rejection.
fn accepts(witness: &UpdateWitness, capacity: &SkeletonCapacity) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        let (ctx, _) = build::<QM31>(witness, capacity);
        ctx.finalize(false).is_circuit_valid()
    }))
    .unwrap_or(false)
}

/// The circuit without the callers' root bindings (they introduce witness constants by design).
fn circuit_of<Value: IValue>(witness: &UpdateWitness, capacity: &SkeletonCapacity) -> Circuit {
    let mut ctx = Context::<Value>::default();
    verify_patricia_update(&mut ctx, witness, capacity);
    ctx.finalize(false).context.circuit
}

fn generous(witness: &UpdateWitness) -> SkeletonCapacity {
    let tight = update_covering(witness);
    SkeletonCapacity { n_binary: tight.n_binary + 4, n_edge: tight.n_edge + 2, ..tight }
}

#[rstest]
#[case(4, 0, 0, 0)]
#[case(0, 4, 0, 0)]
#[case(0, 0, 4, 0)]
#[case(0, 0, 0, 2)]
#[case(2, 2, 2, 1)]
#[case(1, 0, 0, 0)]
fn honest_updates_are_accepted(
    #[case] overwrites: u64,
    #[case] inserts: u64,
    #[case] deletes: u64,
    #[case] noops: u64,
) {
    let witness = fixture_witness(&mixed_rows(overwrites, inserts, deletes, noops));
    assert!(accepts(&witness, &update_covering(&witness)), "tight capacity rejected");
    assert!(accepts(&witness, &generous(&witness)), "generous capacity rejected");
}

/// The nonce-trie genesis: inserts into the empty trie (P5 on the prev side).
#[test]
fn insert_into_the_empty_trie_is_accepted() {
    let rows: Vec<UpdateRow> = (0..3)
        .map(|i| UpdateRow { key: key_of(i), prev_value: [0; 8], new_value: value(i) })
        .collect();
    let (witness, _) = extract_update_skeletons(&BTreeMap::new(), HEIGHT, &rows);
    assert!(accepts(&witness, &update_covering(&witness)));
}

/// Deleting every key empties the trie (P5 on the new side).
#[test]
fn delete_to_the_empty_trie_is_accepted() {
    let small: BTreeMap<Word256, Word256> = (0..3).map(|i| (key_of(i), value(i))).collect();
    let rows: Vec<UpdateRow> = small
        .iter()
        .map(|(key, val)| UpdateRow { key: *key, prev_value: *val, new_value: [0; 8] })
        .collect();
    let (witness, _) = extract_update_skeletons(&small, HEIGHT, &rows);
    assert!(witness.new.root == [0; 8]);
    assert!(accepts(&witness, &update_covering(&witness)));
}

/// Every update rejection case is rejected end to end. Attribution is asserted against the
/// update oracle in `update_test`; here the seam is the whole circuit.
#[test]
fn every_update_rejection_case_is_rejected_by_the_circuit() {
    let base = fixture_witness(&mixed_rows(2, 2, 2, 1));
    let capacity = generous(&base);
    assert!(accepts(&base, &capacity), "the fixture itself must be accepted");
    for case in UPDATE_REJECTION_CASES {
        let mut witness = base.clone();
        (case.apply)(&mut witness);
        assert!(
            !accepts(&witness, &capacity),
            "update rejection case '{}' was accepted by the circuit",
            case.label
        );
    }
}

/// The reshaped false-absence witness: the prev side hides a present key behind the whole-trie
/// sibling while the new side inserts it. The frontiers cannot be shared, so the fixed topology
/// cannot even host the claim — the closing argument's constructive face.
#[test]
fn false_absence_with_an_insert_cannot_be_hosted() {
    use crate::patricia::reference::build_trie;
    use crate::patricia::skeleton::extract_skeleton;

    let prev_entries = entries();
    let target = key_of(0); // present in prev, claimed absent
    let mut new_entries = prev_entries.clone();
    new_entries.insert(target, value(999));

    let prev_tree = build_trie(&prev_entries, HEIGHT).unwrap();
    let new_tree = build_trie(&new_entries, HEIGHT).unwrap();
    // The prev side is extracted over *no* keys (the whole trie stays opaque) and given an
    // absent leaf slot for the target; the new side genuinely walks the inserted key.
    let mut prev = extract_skeleton(Some(&prev_tree), HEIGHT, &[]);
    prev.leaves =
        vec![SkeletonUnit { height: 0, path: target, kind: SkeletonKind::Leaf, hash: [0; 8] }];
    let new = extract_skeleton(Some(&new_tree), HEIGHT, &[target]);
    let witness = UpdateWitness { prev, new };

    let capacity = SkeletonCapacity {
        n_leaves: 1,
        n_binary: witness.new.binaries.len() + 4,
        n_edge: witness.new.edges.len() + 2,
    };
    assert!(!accepts(&witness, &capacity));
}

/// A `(0, 0)` row over a present, unchanged key is accepted — P9's documented semantics, pinned.
#[test]
fn a_no_op_row_hiding_a_present_key_is_accepted() {
    let mut witness = fixture_witness(&mixed_rows(2, 0, 0, 0));
    let hidden = key_of(40); // present, untouched
    let slot = SkeletonUnit { height: 0, path: hidden, kind: SkeletonKind::Leaf, hash: [0; 8] };
    witness.prev.leaves.push(slot);
    witness.new.leaves.push(slot);
    let capacity = update_covering(&witness);
    assert!(accepts(&witness, &capacity), "the (0,0) claim proves 'unchanged', and is accepted");
}

/// The complementary-pair finding, pinned: rows `(k, v, 0)` and `(k, 0, w)` are ACCEPTED —
/// each fold sees one live entry at the position, and the pair jointly behaves as the
/// overwrite `v → w` while each row's absence claim is false. The absence clauses of the
/// statement therefore hold only under the caller's live-key-distinctness precondition
/// (module docs; found by the Lean formalization).
#[test]
fn complementary_rows_on_one_key_act_as_an_overwrite() {
    let key = key_of(0);
    let single = fixture_witness(&[UpdateRow { key, prev_value: value(0), new_value: value(700) }]);
    let mut witness = single.clone();
    let slot = |hash| SkeletonUnit { height: 0, path: key, kind: SkeletonKind::Leaf, hash };
    // Row A = (key, v, 0), row B = (key, 0, w): the overwrite's walk, split across two rows.
    witness.prev.leaves = vec![slot(value(0)), slot([0; 8])];
    witness.new.leaves = vec![slot([0; 8]), slot(value(700))];
    let capacity = update_covering(&witness);
    assert!(
        accepts(&witness, &capacity),
        "the complementary pair is accepted; absence claims need distinct live keys"
    );
}

/// The direction the duplicate-position lemma genuinely covers: same-side duplicate live rows
/// put two live units at one position in one fold, and one of them dangles.
#[test]
fn same_side_duplicate_live_keys_are_rejected() {
    let key = key_of(0);
    let single = fixture_witness(&[UpdateRow { key, prev_value: value(0), new_value: value(700) }]);
    let mut witness = single.clone();
    let slot = |hash| SkeletonUnit { height: 0, path: key, kind: SkeletonKind::Leaf, hash };
    witness.prev.leaves = vec![slot(value(0)), slot(value(0))];
    witness.new.leaves = vec![slot(value(700)), slot(value(700))];
    let capacity = update_covering(&witness);
    assert!(!accepts(&witness, &capacity));
}

#[test]
fn structure_is_witness_independent() {
    let witness = fixture_witness(&mixed_rows(1, 1, 1, 1));
    let capacity = generous(&witness);
    assert_eq!(circuit_of::<QM31>(&witness, &capacity), circuit_of::<NoValue>(&witness, &capacity));
}

/// One `(K, capacity)` hosts every operation mix with an identical circuit — the four row
/// operations are witness shapes, not circuit cases.
#[test]
fn circuit_is_fixed_across_operation_mixes() {
    let capacity = SkeletonCapacity { n_leaves: 3, n_binary: 24, n_edge: 18 };
    let reference = circuit_of::<QM31>(&fixture_witness(&mixed_rows(3, 0, 0, 0)), &capacity);
    for rows in [
        mixed_rows(0, 3, 0, 0),
        mixed_rows(0, 0, 3, 0),
        mixed_rows(1, 1, 1, 0),
        mixed_rows(1, 1, 0, 1),
    ] {
        assert_eq!(circuit_of::<QM31>(&fixture_witness(&rows), &capacity), reference);
    }
    let genesis_rows: Vec<UpdateRow> = (0..3)
        .map(|i| UpdateRow { key: key_of(i), prev_value: [0; 8], new_value: value(i) })
        .collect();
    let (genesis, _) = extract_update_skeletons(&BTreeMap::new(), HEIGHT, &genesis_rows);
    assert_eq!(circuit_of::<QM31>(&genesis, &capacity), reference);
}

/// Pins the update topology across processes; see `skeleton_circuit_test::fingerprint_is_pinned`.
#[test]
fn fingerprint_is_pinned() {
    const UPDATE_2_14_10_FINGERPRINT: &str =
        "df240a684522f70937514cae49a8fed65598821a419312ac81a04388f941c3f9";
    let capacity = SkeletonCapacity { n_leaves: 2, n_binary: 14, n_edge: 10 };
    let witness = fixture_witness(&mixed_rows(1, 1, 0, 0));
    let fingerprint = circuit_fingerprint(&circuit_of::<QM31>(&witness, &capacity)).to_string();
    assert_eq!(fingerprint, UPDATE_2_14_10_FINGERPRINT);
}
