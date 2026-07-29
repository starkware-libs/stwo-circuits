use std::collections::{BTreeMap, BTreeSet};

use rstest::rstest;

use super::mutation::{Family, MUTATIONS, oracle};
use super::{
    BinarySlot, Check, EdgeSlot, INERT_UNIT, SkeletonCapacity, SkeletonKind, SkeletonUnit,
    SkeletonWitness, bottom_path, child_path, extract_skeleton, key_prefix, kind_of,
    witness_invariants,
};
use crate::patricia::reference::{PatriciaTree, Word256, build_trie, hash_binary, hash_edge};
use crate::patricia::slot_count_test::{HEIGHT, SlotCounts, count_skeleton, key_of, trie_of};

/// Entries of the shared fixture trie. Big enough that a batch of a few keys meets untouched
/// sibling subtrees at many depths (design doc §4.1).
const N_ENTRIES: u64 = 2_000;

/// A [`Word256`] with the given low word.
fn w(x: u32) -> Word256 {
    [x, 0, 0, 0, 0, 0, 0, 0]
}

/// A distinct non-zero value for key index `i`.
fn value(i: u32) -> Word256 {
    std::array::from_fn(|k| 0x1000_0000 + i * 8 + k as u32)
}

fn fixture_trie() -> PatriciaTree {
    build_trie(&trie_of(N_ENTRIES), HEIGHT).unwrap()
}

/// `k` keys present in [`fixture_trie`].
fn present_keys(k: usize) -> Vec<Word256> {
    (0..k as u64).map(key_of).collect()
}

/// `k` keys absent from [`fixture_trie`] (indices past its entry count).
fn absent_keys(k: usize) -> Vec<Word256> {
    (N_ENTRIES..N_ENTRIES + k as u64).map(key_of).collect()
}

fn mixed_keys(present: usize, absent: usize) -> Vec<Word256> {
    present_keys(present).into_iter().chain(absent_keys(absent)).collect()
}

/// The witness every mutation is applied to: present keys for leaves and binaries, absent keys so
/// that some edge slot has an opaque sibling bottom, over a trie deep enough to have siblings at
/// many heights.
fn mutation_fixture() -> SkeletonWitness {
    extract_skeleton(&fixture_trie(), HEIGHT, &mixed_keys(6, 3))
}

/// A budget with slack in every class, so every class gets padded slots.
fn generous_capacity(witness: &SkeletonWitness) -> SkeletonCapacity {
    let tight = SkeletonCapacity::covering(witness);
    SkeletonCapacity {
        n_leaves: tight.n_leaves + 4,
        n_binary: tight.n_binary + 16,
        n_edge: tight.n_edge + 5,
    }
}

/// Slot lookup by position, for [`recompute_root`].
struct Positions<'a> {
    binaries: BTreeMap<(u32, Word256), &'a BinarySlot>,
    edges: BTreeMap<(u32, Word256), &'a EdgeSlot>,
    terminals: BTreeMap<(u32, Word256), Word256>,
}

impl Positions<'_> {
    fn hash_at(&self, height: u32, path: Word256) -> Word256 {
        if self.binaries.contains_key(&(height, path)) {
            let left = self.hash_at(height - 1, child_path(&path, 0));
            let right = self.hash_at(height - 1, child_path(&path, 1));
            return hash_binary(&left, &right);
        }
        if let Some(slot) = self.edges.get(&(height, path)) {
            let below = bottom_path(&path, slot.length, &slot.edge_path);
            let bottom = self.hash_at(height - slot.length, below);
            return hash_edge(&bottom, &slot.edge_path, slot.length);
        }
        *self.terminals.get(&(height, path)).expect("no unit at this position")
    }
}

/// Independent bottom-up fold: re-derives the root from the slot structure keyed by *position*,
/// recomputing every hash instead of trusting any slot's stored `out.hash`.
fn recompute_root(witness: &SkeletonWitness) -> Word256 {
    let positions = Positions {
        binaries: witness
            .binaries
            .iter()
            .filter(|slot| !slot.out.is_inert())
            .map(|slot| ((slot.out.height, slot.out.path), slot))
            .collect(),
        edges: witness
            .edges
            .iter()
            .filter(|slot| !slot.out.is_inert())
            .map(|slot| ((slot.out.height, slot.out.path), slot))
            .collect(),
        terminals: witness
            .leaves
            .iter()
            .chain(&witness.siblings)
            .filter(|unit| !unit.is_inert())
            .map(|unit| ((unit.height, unit.path), unit.hash))
            .collect(),
    };
    positions.hash_at(witness.height, [0; 8])
}

#[test]
fn key_prefix_drops_the_low_bits() {
    let key = w(0b1011_0110);
    assert!(key_prefix(&key, 0) == key);
    assert!(key_prefix(&key, 4) == w(0b1011));
    assert!(key_prefix(&key, 8) == w(0));
    let mut high = [0u32; 8];
    high[1] = 0b11;
    assert!(key_prefix(&high, 32) == w(0b11));
    assert!(key_prefix(&high, 33) == w(0b1));
}

#[test]
fn position_relations_match_the_design_doc() {
    let path = w(0b101);
    assert!(child_path(&path, 0) == w(0b1010));
    assert!(child_path(&path, 1) == w(0b1011));
    assert!(bottom_path(&path, 3, &w(0b110)) == w(0b101_110));
    // A shift across the 32-bit limb boundary.
    let mut expected = [0u32; 8];
    expected[0] = 1 << 31;
    expected[1] = 0b10;
    assert!(bottom_path(&w(0b101), 31, &w(0)) == expected);
}

#[test]
fn padding_tag_is_reserved() {
    assert!(SkeletonKind::Padding.tag() == 0);
    assert!(INERT_UNIT.is_inert() && INERT_UNIT.kind.tag() == 0);
    let live = [SkeletonKind::Leaf, SkeletonKind::Binary, SkeletonKind::Edge];
    assert!(live.iter().all(|kind| kind.tag() != 0));
}

/// The extractor's classification agrees with `slot_count_test`'s independent walk. Present keys
/// only — see [`count_skeleton`]'s docs.
#[rstest]
#[case(1)]
#[case(2)]
#[case(7)]
#[case(16)]
#[case(64)]
fn unit_counts_match_count_skeleton(#[case] k: usize) {
    let tree = fixture_trie();
    let keys = present_keys(k);
    let witness = extract_skeleton(&tree, HEIGHT, &keys);
    let expected = count_skeleton(&tree, HEIGHT, &keys);
    let actual = SlotCounts {
        binary: witness.binaries.len(),
        edge: witness.edges.len(),
        siblings: witness.siblings.len(),
    };
    assert!(actual == expected, "K={k}: {actual:?} != {expected:?}");
    assert!(witness.leaves.len() == k);
}

#[rstest]
#[case(1, 0)]
#[case(7, 0)]
#[case(0, 4)]
#[case(6, 3)]
#[case(16, 16)]
fn sibling_identity_holds_on_the_extracted_witness(#[case] present: usize, #[case] absent: usize) {
    let witness = extract_skeleton(&fixture_trie(), HEIGHT, &mixed_keys(present, absent));
    assert!(
        witness.siblings.len() == witness.binaries.len() + 1 - witness.leaves.len(),
        "identity broken: {} siblings, {} binaries, {} leaves",
        witness.siblings.len(),
        witness.binaries.len(),
        witness.leaves.len()
    );
}

#[test]
fn leaf_units_carry_their_keys() {
    let keys = present_keys(16);
    let witness = extract_skeleton(&fixture_trie(), HEIGHT, &keys);
    let paths: BTreeSet<Word256> = witness.leaves.iter().map(|unit| unit.path).collect();
    assert!(paths == keys.iter().copied().collect::<BTreeSet<Word256>>());
    assert!(witness.leaves.iter().all(|unit| unit.height == 0));
    assert!(witness.leaves.iter().all(|unit| unit.kind == SkeletonKind::Leaf));
}

#[rstest]
#[case(0, 0)]
#[case(1, 0)]
#[case(7, 0)]
#[case(0, 3)]
#[case(6, 3)]
fn refolding_the_units_reproduces_the_root(#[case] present: usize, #[case] absent: usize) {
    let tree = fixture_trie();
    let witness = extract_skeleton(&tree, HEIGHT, &mixed_keys(present, absent));
    assert!(witness.root == tree.hash());
    assert!(recompute_root(&witness) == tree.hash());
}

#[rstest]
#[case(0, 0)]
#[case(1, 0)]
#[case(7, 0)]
#[case(0, 3)]
#[case(6, 3)]
#[case(64, 8)]
fn extracted_witnesses_satisfy_the_invariants(#[case] present: usize, #[case] absent: usize) {
    let witness = extract_skeleton(&fixture_trie(), HEIGHT, &mixed_keys(present, absent));
    witness_invariants(&witness).expect("extracted witness must be valid");
}

#[test]
fn empty_trie_has_no_skeleton() {
    // The empty map has no node, so there is no root unit for the multiset to hold out; a caller
    // must special-case the all-zero root rather than extract a skeleton.
    assert!(build_trie(&BTreeMap::new(), HEIGHT).is_none());
}

#[test]
fn no_keys_collapses_the_trie_into_one_sibling() {
    let tree = fixture_trie();
    let witness = extract_skeleton(&tree, HEIGHT, &[]);
    assert!(witness.leaves.is_empty() && witness.binaries.is_empty() && witness.edges.is_empty());
    let root =
        SkeletonUnit { height: HEIGHT, path: [0; 8], kind: kind_of(&tree), hash: tree.hash() };
    assert!(witness.siblings == vec![root]);
    witness_invariants(&witness).unwrap();
}

#[test]
fn single_leaf_trie_is_one_full_height_edge() {
    let key = w(0b1011);
    let tree = build_trie(&[(key, value(0))].into_iter().collect(), 8).unwrap();
    let witness = extract_skeleton(&tree, 8, &[key]);
    let leaf = SkeletonUnit { height: 0, path: key, kind: SkeletonKind::Leaf, hash: value(0) };
    assert!(witness.leaves == vec![leaf]);
    assert!(witness.binaries.is_empty() && witness.siblings.is_empty());
    assert!(witness.edges.len() == 1);
    assert!(witness.edges[0].length == 8 && witness.edges[0].edge_path == key);
    witness_invariants(&witness).unwrap();
}

/// Two keys differing only in bit 0 sit under one edge over a binary of leaves — the tightest
/// possible divergence.
#[test]
fn keys_differing_in_the_last_bit() {
    let tree = build_trie(&[(w(0b110), value(0)), (w(0b111), value(1))].into(), 3).unwrap();
    let witness = extract_skeleton(&tree, 3, &[w(0b110), w(0b111)]);
    assert!(witness.leaves.len() == 2 && witness.binaries.len() == 1 && witness.edges.len() == 1);
    assert!(witness.siblings.is_empty());
    assert!(witness.binaries[0].out.height == 1);
    witness_invariants(&witness).unwrap();
}

/// An absent key diverges inside the shared edge, so the batch touches the edge but not its bottom
/// — the non-membership evidence an insert needs.
#[test]
fn absent_key_ends_the_walk_at_the_diverging_edge() {
    let tree = build_trie(&[(w(0b110), value(0)), (w(0b111), value(1))].into(), 3).unwrap();
    let witness = extract_skeleton(&tree, 3, &[w(0b010)]);
    assert!(witness.leaves.is_empty() && witness.binaries.is_empty());
    assert!(witness.edges.len() == 1 && witness.siblings.len() == 1);
    assert!(witness.siblings[0] == witness.edges[0].bottom);
    assert!(witness.siblings[0].kind == SkeletonKind::Binary);
    witness_invariants(&witness).unwrap();
}

/// Pure inserts, the nonce-trie case: no leaf unit at all, and at least one edge left with an
/// opaque bottom.
#[test]
fn all_keys_absent_yields_no_leaves() {
    let witness = extract_skeleton(&fixture_trie(), HEIGHT, &absent_keys(8));
    assert!(witness.leaves.is_empty());
    assert!(!witness.edges.is_empty() && !witness.binaries.is_empty());
    let opaque =
        witness.edges.iter().filter(|slot| witness.siblings.contains(&slot.bottom)).count();
    assert!(opaque > 0, "an absent key must leave an edge with an opaque bottom");
    witness_invariants(&witness).unwrap();
}

#[test]
fn duplicate_keys_yield_the_same_witness() {
    let tree = fixture_trie();
    let keys = present_keys(4);
    let doubled: Vec<Word256> = keys.iter().chain(&keys).copied().collect();
    assert!(extract_skeleton(&tree, HEIGHT, &doubled) == extract_skeleton(&tree, HEIGHT, &keys));
}

#[test]
#[should_panic(expected = "does not fit height")]
fn key_above_the_height_panics() {
    let tree = build_trie(&[(w(0b110), value(0))].into(), 3).unwrap();
    extract_skeleton(&tree, 3, &[w(0b1000)]);
}

#[test]
fn capacity_derives_the_sibling_count() {
    let witness = mutation_fixture();
    assert!(SkeletonCapacity::covering(&witness).n_siblings() == witness.siblings.len());
    assert!(SkeletonCapacity { n_leaves: 4, n_binary: 9, n_edge: 2 }.n_siblings() == 6);
}

#[test]
#[should_panic(expected = "infeasible capacity")]
fn capacity_with_fewer_binaries_than_leaves_panics() {
    SkeletonCapacity { n_leaves: 8, n_binary: 3, n_edge: 4 }.n_siblings();
}

/// Padding is inert *and* self-balancing: every invariant — the sibling identity and the multiset
/// included — survives padding, because siblings are derived as `n_binary - n_leaves + 1`.
#[test]
fn padding_preserves_every_invariant() {
    let witness = mutation_fixture();
    let capacity = generous_capacity(&witness);
    let padded = witness.padded(&capacity);
    assert!(padded.leaves.len() == capacity.n_leaves);
    assert!(padded.binaries.len() == capacity.n_binary);
    assert!(padded.edges.len() == capacity.n_edge);
    assert!(padded.siblings.len() == capacity.n_siblings());
    assert!(padded.leaves.iter().filter(|unit| unit.is_inert()).count() == 4);
    witness_invariants(&padded).expect("padded witness must be valid");
    assert!(recompute_root(&padded) == witness.root);
}

#[test]
#[should_panic(expected = "sibling slots but the witness needs")]
fn padding_to_too_few_sibling_slots_panics() {
    let witness = mutation_fixture();
    let tight = SkeletonCapacity::covering(&witness);
    witness.padded(&SkeletonCapacity { n_leaves: tight.n_leaves + 8, ..tight });
}

/// Order within a class is meaningless — that is what the multiset buys. Reversing every class must
/// still validate, so no test here can be passing for an ordering reason.
#[test]
fn class_order_is_irrelevant() {
    let base = mutation_fixture();
    let mut witness = base.padded(&generous_capacity(&base));
    witness.leaves.reverse();
    witness.siblings.reverse();
    witness.binaries.reverse();
    witness.edges.reverse();
    witness_invariants(&witness).expect("reordering a class must not matter");
}

#[test]
fn mutation_labels_are_unique() {
    let labels: BTreeSet<&str> = MUTATIONS.iter().map(|mutation| mutation.label).collect();
    assert!(labels.len() == MUTATIONS.len());
}

/// Every mutation is rejected, and rejected by the invariant it targets — a rejection for an
/// unintended reason proves nothing.
#[test]
fn every_mutation_is_rejected_by_the_check_it_targets() {
    let base = mutation_fixture();
    let padded = base.padded(&generous_capacity(&base));
    oracle(&base).expect("the fixture must be valid");
    oracle(&padded).expect("the padded fixture must be valid");
    for mutation in MUTATIONS {
        let mut witness = if mutation.needs_padding { padded.clone() } else { base.clone() };
        (mutation.apply)(&mut witness);
        let violation =
            oracle(&witness).expect_err(&format!("mutation '{}' was accepted", mutation.label));
        assert!(
            violation.check == mutation.detected_by,
            "mutation '{}' targets {:?} but was rejected by {violation}",
            mutation.label,
            mutation.detected_by
        );
    }
}

/// Padding must not mask a tampering: the unpadded mutations still land on a padded witness.
#[test]
fn mutations_are_still_rejected_after_padding() {
    let base = mutation_fixture();
    let padded = base.padded(&generous_capacity(&base));
    for mutation in MUTATIONS.iter().filter(|mutation| !mutation.needs_padding) {
        let mut witness = padded.clone();
        (mutation.apply)(&mut witness);
        let violation = oracle(&witness)
            .expect_err(&format!("padded mutation '{}' was accepted", mutation.label));
        assert!(violation.check == mutation.detected_by, "{}: {violation}", mutation.label);
    }
}

/// Every invariant the oracle can report has a mutation exercising it, so no check is untested.
#[test]
fn every_check_has_a_mutation() {
    let covered: BTreeSet<Check> = MUTATIONS.iter().map(|mutation| mutation.detected_by).collect();
    let all = [
        Check::Padding,
        Check::KindTag,
        Check::Canonicity,
        Check::Position,
        Check::Hashing,
        Check::SiblingCount,
        Check::Multiset,
        Check::Root,
    ];
    assert!(covered == all.into_iter().collect::<BTreeSet<Check>>());
}

/// Every constraint family has a mutation attacking it — the mutation-matrix rows.
#[test]
fn every_family_has_a_mutation() {
    let covered: BTreeSet<Family> = MUTATIONS.iter().map(|mutation| mutation.family).collect();
    let all = [
        Family::KindConfusion,
        Family::PositionBinding,
        Family::NodeHashing,
        Family::Canonicity,
        Family::MultisetBalance,
        Family::LivePadding,
        Family::RootBinding,
    ];
    assert!(covered == all.into_iter().collect::<BTreeSet<Family>>());
}

/// A leaf unit moved into the sibling class *without* changing its tag is invisible to these
/// invariants: cardinalities and the multiset are untouched, and a height-0 `Leaf` unit is a
/// structurally valid sibling. What forbids it is the caller binding each leaf slot to a batch key,
/// which lives outside the skeleton — recorded here so the gap stays explicit.
#[test]
fn same_kind_class_migration_is_not_detected_here() {
    let mut witness = mutation_fixture();
    let leaf = witness.leaves.remove(0);
    witness.siblings.push(leaf);
    assert!(witness_invariants(&witness).is_ok());
}
