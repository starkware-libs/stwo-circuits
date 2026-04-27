//! End-to-end Cairo serde round-trip — analogous to
//! `circuit_serialize::test::test_serialize_deserialize`. Builds a real circuit prover
//! output, converts it into the Cairo verifier's input format via this crate's
//! `CairoSerialize` impls, deserializes the felt252 stream back via `CairoDeserialize`,
//! and asserts the result equals the original.

use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_prover::prover::{BaseColumnPool, SimdBackend, prove_circuit_assignment};
use circuit_verifier::circuit_components::ComponentList;
use circuits::context::Context;
use circuits::ivalue::qm31_from_u32s;
use circuits::ops::{guess, output};
use num_traits::{One, Zero};
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;
use stwo_cairo_serialize::{CairoDeserialize, CairoSerialize};

use crate::claim::{CairoCircuitClaim, CairoCircuitInteractionClaim};
use crate::proof::{CairoCircuitProof, prepare_circuit_proof_for_cairo_verifier};

fn qm31(a: u32, b: u32, c: u32, d: u32) -> QM31 {
    qm31_from_u32s(a, b, c, d)
}

/// Builds a tiny but real circuit assignment so we can run the prover and get a
/// well-formed `CircuitProof` for serde round-trip test. Same shape as the Fibonacci context used
/// in `circuit_prover::test`, just smaller.
fn build_minimal_context() -> Context<QM31> {
    const N: usize = 16;
    let mut ctx = Context::<QM31>::default();
    let (mut a, mut b) = (guess(&mut ctx, QM31::zero()), guess(&mut ctx, QM31::one()));
    for _ in 2..N {
        (a, b) = (b, circuits::eval!(&mut ctx, (a) + (b)));
    }
    output(&mut ctx, b);
    ctx
}

#[test]
fn test_serialize_deserialize_cairo_proof() {
    let mut ctx = build_minimal_context();
    ctx.finalize_guessed_vars();
    ctx.validate_circuit();
    let preprocessed_circuit = PreprocessedCircuit::preprocess_circuit(&mut ctx);
    let circuit_proof = prove_circuit_assignment(
        ctx.values(),
        &preprocessed_circuit,
        &BaseColumnPool::<SimdBackend>::new(),
        PcsConfig::default(),
    )
    .unwrap();

    // Round-trip via the same CairoSerialize / CairoDeserialize implementations the
    // Cairo verifier consumes. We can't compare `CairoCircuitProof` values directly
    // (FriProof / MerkleDecommitmentLifted don't implement `PartialEq`), so we compare
    // re-serialized felts — if the deserializer is the inverse of the serializer, the
    // two byte streams must match.
    let felts = prepare_circuit_proof_for_cairo_verifier(circuit_proof);
    let mut iter = felts.iter();
    let deserialized: CairoCircuitProof<Blake2sM31MerkleHasher> =
        CairoCircuitProof::deserialize(&mut iter);
    assert!(iter.next().is_none(), "trailing data after proof");
    let mut felts_after = Vec::new();
    CairoSerialize::serialize(&deserialized, &mut felts_after);
    assert_eq!(felts, felts_after);
}

#[test]
fn test_serialize_deserialize_claim_and_interaction_claim() {
    // 16 distinct values so any ordering bug shows up.
    let claim = CairoCircuitClaim {
        output_values: vec![qm31(1, 2, 3, 4), qm31(5, 6, 7, 8)],
        eq_log_size: 10,
        qm31_ops_log_size: 11,
        blake_gate_log_size: 12,
        blake_round_log_size: 13,
        blake_g_log_size: 14,
        blake_output_log_size: 15,
        triple_xor_32_log_size: 16,
        m_31_to_u_32_log_size: 17,
    };
    let interaction = CairoCircuitInteractionClaim {
        eq: qm31(1, 0, 0, 0),
        qm31_ops: qm31(2, 0, 0, 0),
        blake_gate: qm31(3, 0, 0, 0),
        blake_round: qm31(4, 0, 0, 0),
        blake_round_sigma: qm31(5, 0, 0, 0),
        blake_g: qm31(6, 0, 0, 0),
        blake_output: qm31(7, 0, 0, 0),
        triple_xor_32: qm31(8, 0, 0, 0),
        m_31_to_u_32: qm31(9, 0, 0, 0),
        verify_bitwise_xor_8: qm31(10, 0, 0, 0),
        verify_bitwise_xor_12: qm31(11, 0, 0, 0),
        verify_bitwise_xor_4: qm31(12, 0, 0, 0),
        verify_bitwise_xor_7: qm31(13, 0, 0, 0),
        verify_bitwise_xor_9: qm31(14, 0, 0, 0),
        range_check_15: qm31(15, 0, 0, 0),
        range_check_16: qm31(16, 0, 0, 0),
    };

    // Roundtrip claim
    let mut felts = Vec::new();
    CairoSerialize::serialize(&claim, &mut felts);
    let mut iter = felts.iter();
    let claim_back = CairoCircuitClaim::deserialize(&mut iter);
    assert!(iter.next().is_none());
    assert_eq!(claim, claim_back);

    // Roundtrip interaction_claim
    let mut felts = Vec::new();
    CairoSerialize::serialize(&interaction, &mut felts);
    let mut iter = felts.iter();
    let interaction_back = CairoCircuitInteractionClaim::deserialize(&mut iter);
    assert!(iter.next().is_none());
    assert_eq!(interaction, interaction_back);
}

#[test]
fn test_claim_field_order_matches_component_list() {
    // Sanity-check that `CairoCircuitClaim::from(&CircuitClaim)` selects the right
    // indices: drop different sentinels into each of the 16 log_sizes and confirm only
    // the variable-size ones survive (in the right slot).
    use circuit_verifier::circuit_claim::CircuitClaim;
    let mut log_sizes = [0u32; 16];
    log_sizes[ComponentList::Eq as usize] = 100;
    log_sizes[ComponentList::Qm31Ops as usize] = 101;
    log_sizes[ComponentList::BlakeGate as usize] = 102;
    log_sizes[ComponentList::BlakeRound as usize] = 103;
    log_sizes[ComponentList::BlakeG as usize] = 105;
    log_sizes[ComponentList::BlakeOutput as usize] = 106;
    log_sizes[ComponentList::TripleXor32 as usize] = 107;
    log_sizes[ComponentList::M31ToU32 as usize] = 108;

    let rust_claim = CircuitClaim { log_sizes, output_values: vec![qm31(9, 9, 9, 9)] };
    let cairo_claim = CairoCircuitClaim::from(&rust_claim);
    assert_eq!(cairo_claim.eq_log_size, 100);
    assert_eq!(cairo_claim.qm31_ops_log_size, 101);
    assert_eq!(cairo_claim.blake_gate_log_size, 102);
    assert_eq!(cairo_claim.blake_round_log_size, 103);
    assert_eq!(cairo_claim.blake_g_log_size, 105);
    assert_eq!(cairo_claim.blake_output_log_size, 106);
    assert_eq!(cairo_claim.triple_xor_32_log_size, 107);
    assert_eq!(cairo_claim.m_31_to_u_32_log_size, 108);
    assert_eq!(cairo_claim.output_values, vec![qm31(9, 9, 9, 9)]);
}
