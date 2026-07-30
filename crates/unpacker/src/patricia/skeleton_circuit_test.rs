use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};

use circuits::circuit::Circuit;
use circuits::context::Context;
use circuits::ivalue::{IValue, NoValue};
use circuits::ops::eq;
use rstest::rstest;
use stwo::core::fields::qm31::QM31;

use super::{VerifiedSkeleton, verify_patricia_skeleton};
use crate::fingerprint::circuit_fingerprint;
use crate::patricia::reference::{PatriciaTree, Word256, build_trie};
use crate::patricia::skeleton::rejection::REJECTION_CASES;
use crate::patricia::skeleton::{SkeletonCapacity, SkeletonWitness, extract_skeleton};

/// Trie height used throughout — the production height.
const HEIGHT: u32 = 251;

/// A small fixture trie: big enough for absent keys to diverge mid-trie, small enough that a
/// couple of dozen Blake gates cover the skeleton.
const N_ENTRIES: u64 = 48;

fn key_of(i: u64) -> Word256 {
    // Pseudorandom-ish 251-bit keys, deterministic (no process randomness in tests).
    let scrambled = i.wrapping_mul(0x9E37_79B9_7F4A_7C15).rotate_left(17) ^ (i << 32);
    let mut key = [0u32; 8];
    key[0] = scrambled as u32;
    key[1] = (scrambled >> 32) as u32;
    key[6] = (i as u32).wrapping_mul(0x0101_0101);
    key[7] = ((scrambled as u32) ^ (i as u32)) & ((1 << 27) - 1);
    key
}

fn value_of(i: u64) -> Word256 {
    std::array::from_fn(|k| 0x1000_0000 + (i as u32) * 8 + k as u32)
}

fn fixture_trie() -> PatriciaTree {
    let entries: BTreeMap<Word256, Word256> =
        (0..N_ENTRIES).map(|i| (key_of(i), value_of(i))).collect();
    build_trie(&entries, HEIGHT).expect("non-empty")
}

/// `present` keys in the trie followed by `absent` keys past its entry count.
fn mixed_keys(present: usize, absent: usize) -> Vec<Word256> {
    let present = (0..present as u64).map(key_of);
    let absent = (N_ENTRIES..N_ENTRIES + absent as u64).map(key_of);
    present.chain(absent).collect()
}

fn fixture_witness(present: usize, absent: usize) -> SkeletonWitness {
    extract_skeleton(Some(&fixture_trie()), HEIGHT, &mixed_keys(present, absent))
}

/// Builds the circuit and binds the returned root to the witness's claimed root — the caller-side
/// binding the statement assumes.
fn build<Value: IValue>(
    witness: &SkeletonWitness,
    capacity: &SkeletonCapacity,
) -> (Context<Value>, VerifiedSkeleton) {
    let mut ctx = Context::default();
    let skeleton = verify_patricia_skeleton(&mut ctx, witness, capacity);
    for (word, claimed) in skeleton.root.iter().zip(witness.root) {
        let packed = ctx.constant(QM31::pack_u32(claimed));
        eq(&mut ctx, *word.get(), packed);
    }
    (ctx, skeleton)
}

/// Whether the circuit accepts `witness` — the full pipeline including finalization, so range
/// constraints participate. A build-time panic (an unpaddable or non-permutable witness) counts
/// as rejection: a fixed-topology circuit cannot even host such a witness.
fn accepts(witness: &SkeletonWitness, capacity: &SkeletonCapacity) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        let (ctx, _) = build::<QM31>(witness, capacity);
        ctx.finalize(false).is_circuit_valid()
    }))
    .unwrap_or(false)
}

/// The skeleton circuit alone, *without* the caller's root binding: the binding introduces the
/// claimed root as constants, which are witness data by design — the fixed topology claim is
/// about everything up to that point.
fn circuit_of<Value: IValue>(witness: &SkeletonWitness, capacity: &SkeletonCapacity) -> Circuit {
    let mut ctx = Context::<Value>::default();
    verify_patricia_skeleton(&mut ctx, witness, capacity);
    ctx.finalize(false).context.circuit
}

/// A budget with padded slots in every paddable class.
fn generous(witness: &SkeletonWitness) -> SkeletonCapacity {
    let tight = SkeletonCapacity::covering(witness);
    SkeletonCapacity { n_binary: tight.n_binary + 4, n_edge: tight.n_edge + 2, ..tight }
}

#[rstest]
#[case(1, 0)]
#[case(6, 3)]
#[case(0, 4)]
#[case(0, 0)]
fn extracted_witnesses_are_accepted(#[case] present: usize, #[case] absent: usize) {
    let witness = fixture_witness(present, absent);
    assert!(accepts(&witness, &SkeletonCapacity::covering(&witness)), "tight capacity rejected");
    assert!(accepts(&witness, &generous(&witness)), "generous capacity rejected");
}

#[test]
fn empty_trie_is_accepted() {
    let witness = extract_skeleton(None, HEIGHT, &mixed_keys(0, 3));
    assert!(accepts(&witness, &SkeletonCapacity::covering(&witness)));
}

#[test]
fn single_leaf_trie_is_one_full_height_edge() {
    let key = [0b1011, 0, 0, 0, 0, 0, 0, 0];
    let tree = build_trie(&[(key, value_of(0))].into_iter().collect(), 8).unwrap();
    let witness = extract_skeleton(Some(&tree), 8, &[key]);
    assert!(accepts(&witness, &SkeletonCapacity::covering(&witness)));
}

/// The `height == 0` shape: the trie is a single leaf at path 0 and the root entry expects the
/// `Leaf` tag.
#[test]
fn height_zero_trie_is_accepted() {
    let tree = build_trie(&[([0; 8], value_of(1))].into_iter().collect(), 0).unwrap();
    let witness = extract_skeleton(Some(&tree), 0, &[[0; 8]]);
    assert!(accepts(&witness, &SkeletonCapacity::covering(&witness)));
}

/// Every rejection-case row is rejected by the circuit. Attribution (which constraint family
/// rejects) is asserted against the out-of-circuit oracle in `skeleton_test`; here the seam is the
/// whole circuit, and rejection may also surface as a build-time panic — see [`accepts`].
#[test]
fn every_rejection_case_is_rejected_by_the_circuit() {
    let base = fixture_witness(6, 3);
    let capacity = generous(&base);
    assert!(accepts(&base, &capacity), "the fixture itself must be accepted");
    let padded = base.padded(&capacity);
    for case in REJECTION_CASES {
        let mut witness = if case.needs_padding { padded.clone() } else { base.clone() };
        (case.apply)(&mut witness);
        assert!(
            !accepts(&witness, &capacity),
            "rejection case '{}' was accepted by the circuit",
            case.label
        );
    }
}

/// P5, the other direction: a non-zero root claimed over an empty skeleton dangles — the root
/// entry consumes a unit nothing produced.
#[test]
fn nonzero_root_over_an_empty_skeleton_is_rejected() {
    let mut witness = extract_skeleton(None, HEIGHT, &mixed_keys(0, 2));
    witness.root = [7, 0, 0, 0, 0, 0, 0, 0];
    assert!(!accepts(&witness, &SkeletonCapacity::covering(&witness)));
}

/// The emitted circuit is identical with and without witness values.
#[test]
fn structure_is_witness_independent() {
    let witness = fixture_witness(3, 2);
    let capacity = generous(&witness);
    assert_eq!(circuit_of::<QM31>(&witness, &capacity), circuit_of::<NoValue>(&witness, &capacity));
}

/// The emitted circuit depends only on `(height, capacity)`, not on the trie or the batch.
#[test]
fn circuit_is_fixed_across_witnesses() {
    let capacity = SkeletonCapacity { n_leaves: 4, n_binary: 14, n_edge: 10 };
    let reference = circuit_of::<QM31>(&fixture_witness(4, 0), &capacity);
    assert_eq!(circuit_of::<QM31>(&fixture_witness(2, 2), &capacity), reference);
    assert_eq!(circuit_of::<QM31>(&fixture_witness(1, 3), &capacity), reference);
    let other_trie = {
        let entries: BTreeMap<Word256, Word256> =
            (100..140).map(|i| (key_of(i), value_of(i))).collect();
        build_trie(&entries, HEIGHT).expect("non-empty")
    };
    let other_keys: Vec<Word256> = (100..104).map(key_of).collect();
    let other = extract_skeleton(Some(&other_trie), HEIGHT, &other_keys);
    assert_eq!(circuit_of::<QM31>(&other, &capacity), reference);
    let empty = extract_skeleton(None, HEIGHT, &mixed_keys(0, 4));
    assert_eq!(
        circuit_of::<QM31>(&empty, &SkeletonCapacity { n_leaves: 4, n_binary: 14, n_edge: 10 }),
        reference
    );
}

/// Pins the topology across processes — same-process equality cannot see per-process
/// nondeterminism (e.g. `HashMap` iteration order). See `unpacker_test::fingerprint_is_pinned`.
#[test]
fn fingerprint_is_pinned() {
    const SKELETON_2_6_4_FINGERPRINT: &str =
        "ff156691c2304f42021465d5959e90292aa0092815ebd70a16c04dea8decf70e";
    let capacity = SkeletonCapacity { n_leaves: 2, n_binary: 6, n_edge: 4 };
    let witness = fixture_witness(2, 0);
    let fingerprint = circuit_fingerprint(&circuit_of::<QM31>(&witness, &capacity)).to_string();
    assert_eq!(fingerprint, SKELETON_2_6_4_FINGERPRINT);
}
