use crate::prover::blake_iv_public_logup_sum;
use crate::prover::{CircuitProof, finalize_context, prove_circuit};
use crate::witness::components::{
    blake_g, blake_gate, blake_output, blake_round, blake_round_sigma, eq, qm31_ops,
    range_check_15, range_check_16, triple_xor_32, verify_bitwise_xor_4, verify_bitwise_xor_7,
    verify_bitwise_xor_8, verify_bitwise_xor_9, verify_bitwise_xor_12,
};
use crate::witness::preprocessed::PreProcessedTrace;
use crate::witness::trace::{TraceGenerator, write_interaction_trace};
use circuit_air::statement::{CircuitStatement, INTERACTION_POW_BITS};
use circuit_air::{CircuitClaim, CircuitInteractionElements};
use circuits::blake::blake;
use circuits::context::{TraceContext, Var};
use circuits::eval;
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::ops::{Guess, permute};
use circuits::{context::Context, ops::guess};
use circuits_stark_verifier::proof::{Claim, ProofConfig};
use circuits_stark_verifier::proof_from_stark_proof::{
    pack_component_log_sizes, pack_enable_bits, proof_from_stark_proof,
};

use expect_test::expect;
use num_traits::{One, Zero};
use std::sync::Arc;
use stwo::core::air::Component;
use stwo::core::channel::Blake2sM31Channel;
use stwo::core::channel::Channel;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::{CommitmentSchemeVerifier, PcsConfig, TreeVec};
use stwo::core::poly::circle::CanonicCoset;
use stwo::core::proof_of_work::GrindOps;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::poly::circle::PolyOps;
use stwo::prover::{CommitmentSchemeProver, TreeBuilder, prove_ex};
use stwo_constraint_framework::relation_tracker::{RelationSummary, add_to_relation_entries};

// Not a power of 2 so that we can test component padding.
const N: usize = 1030;

pub fn build_fibonacci_context() -> Context<QM31> {
    let mut context = Context::<QM31>::default();

    let (mut a, mut b) = (guess(&mut context, QM31::zero()), guess(&mut context, QM31::one()));
    for _ in 2..N {
        (a, b) = (b, eval!(&mut context, (a) + (b)));
    }

    expect![[r#"
        (809871181 + 0i) + (0 + 0i)u
    "#]]
    .assert_debug_eq(&context.get(b));

    context
}

pub fn build_permutation_context() -> Context<QM31> {
    let mut context = Context::<QM31>::default();

    let a = guess(&mut context, qm31_from_u32s(0, 2, 0, 2));
    let b = guess(&mut context, qm31_from_u32s(1, 1, 1, 1));

    let outputs = permute(&mut context, &[a, b], IValue::sort_by_u_coordinate);
    let _outputs = permute(&mut context, &outputs, IValue::sort_by_u_coordinate);

    context
}

pub fn build_blake_gate_context() -> Context<QM31> {
    let mut context = Context::<QM31>::default();
    context.enable_assert_eq_on_eval();

    let mut inputs: Vec<Var> = vec![];
    let n_inputs = 4;
    let n_bytes = n_inputs * 16;
    let n_blake_gates = 10;
    for i in 0..n_inputs {
        inputs.push(guess(
            &mut context,
            qm31_from_u32s(4 * i + 82, 4 * i + 83, 4 * i + 84, 4 * i + 85),
        ));
    }
    for _ in 0..n_blake_gates {
        let _output = blake(&mut context, &inputs, n_bytes as usize);
    }

    context
}

#[allow(dead_code)]
fn evals_to_cols(
    evals: &[stwo::prover::poly::circle::CircleEvaluation<
        SimdBackend,
        M31,
        stwo::prover::poly::BitReversedOrder,
    >],
) -> Vec<Vec<M31>> {
    evals.iter().map(|eval| eval.to_cpu().values.clone()).collect()
}

#[allow(dead_code)]
fn write_trace_with_cols(
    context_values: &[QM31],
    preprocessed_trace: Arc<PreProcessedTrace>,
    trace_generator: &TraceGenerator,
    tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sM31MerkleChannel>,
) -> (CircuitClaim, crate::witness::trace::CircuitInteractionClaimGenerator, Vec<Vec<M31>>) {
    let preprocessed_trace_ref = preprocessed_trace.as_ref();
    let mut original_cols: Vec<Vec<M31>> = vec![];

    // Write eq component.
    let (eq_trace, eq_log_size, eq_lookup_data) =
        eq::write_trace(context_values, preprocessed_trace_ref);
    let eq_evals = eq_trace.to_evals();
    original_cols.extend(evals_to_cols(&eq_evals));
    tree_builder.extend_evals(eq_evals);

    // Write qm31_ops component.
    let (qm31_ops_trace, qm31_ops_log_size, qm31_ops_lookup_data) = qm31_ops::write_trace(
        context_values,
        preprocessed_trace_ref,
        &trace_generator.qm31_ops_trace_generator,
    );
    let qm31_ops_evals = qm31_ops_trace.to_evals();
    original_cols.extend(evals_to_cols(&qm31_ops_evals));
    tree_builder.extend_evals(qm31_ops_evals);

    let verify_bitwise_xor_8_state =
        verify_bitwise_xor_8::ClaimGenerator::new(preprocessed_trace.clone());
    let verify_bitwise_xor_12_state =
        verify_bitwise_xor_12::ClaimGenerator::new(preprocessed_trace.clone());
    let verify_bitwise_xor_4_state =
        verify_bitwise_xor_4::ClaimGenerator::new(preprocessed_trace.clone());
    let verify_bitwise_xor_7_state =
        verify_bitwise_xor_7::ClaimGenerator::new(preprocessed_trace.clone());
    let verify_bitwise_xor_9_state =
        verify_bitwise_xor_9::ClaimGenerator::new(preprocessed_trace.clone());
    let range_check_16_state = range_check_16::ClaimGenerator::new(preprocessed_trace.clone());
    let range_check_15_state = range_check_15::ClaimGenerator::new(preprocessed_trace.clone());
    let mut triple_xor_32_state = triple_xor_32::ClaimGenerator::new();
    let blake_gate_claim_generator = blake_gate::ClaimGenerator::new(preprocessed_trace.clone());
    let mut blake_round_generator = blake_round::ClaimGenerator::default();
    let blake_round_sigma_generator =
        blake_round_sigma::ClaimGenerator::new(preprocessed_trace.clone());
    let mut blake_g_generator = blake_g::ClaimGenerator::new();

    // Write blake gate component.
    let (
        blake_gate_trace,
        blake_gate_interaction_claim_gen,
        blake_message_state,
        blake_output_component_input,
    ) = blake_gate_claim_generator.write_trace(
        context_values,
        preprocessed_trace_ref,
        &verify_bitwise_xor_8_state,
        &range_check_16_state,
        &range_check_15_state,
        &mut blake_round_generator,
        &mut triple_xor_32_state,
    );
    let blake_gate_evals = blake_gate_trace.to_evals();
    original_cols.extend(evals_to_cols(&blake_gate_evals));
    tree_builder.extend_evals(blake_gate_evals);

    // Write blake round component.
    let (blake_round_trace, blake_round_log_size, blake_round_interaction_claim_gen) =
        blake_round_generator.write_trace(
            &blake_round_sigma_generator,
            &blake_message_state,
            &mut blake_g_generator,
        );
    let blake_round_evals = blake_round_trace.to_evals();
    original_cols.extend(evals_to_cols(&blake_round_evals));
    tree_builder.extend_evals(blake_round_evals);

    // Write blake round sigma component.
    let (
        blake_round_sigma_trace,
        _blake_round_sigma_claim,
        blake_round_sigma_interaction_claim_gen,
    ) = blake_round_sigma_generator.write_trace();
    let blake_round_sigma_evals = blake_round_sigma_trace.to_evals();
    original_cols.extend(evals_to_cols(&blake_round_sigma_evals));
    tree_builder.extend_evals(blake_round_sigma_evals);

    // Write blake g component.
    let (blake_g_trace, blake_g_claim, blake_g_interaction_claim_gen) = blake_g_generator
        .write_trace(
            &verify_bitwise_xor_8_state,
            &verify_bitwise_xor_12_state,
            &verify_bitwise_xor_4_state,
            &verify_bitwise_xor_7_state,
            &verify_bitwise_xor_9_state,
        );
    let blake_g_evals = blake_g_trace.to_evals();
    original_cols.extend(evals_to_cols(&blake_g_evals));
    tree_builder.extend_evals(blake_g_evals);

    // Write blake output component.
    let blake_output_generator =
        blake_output::ClaimGenerator::new(blake_output_component_input, preprocessed_trace);
    let (blake_output_trace, blake_output_claim, blake_output_interaction_claim_gen) =
        blake_output_generator.write_trace();
    let blake_output_evals = blake_output_trace.to_evals();
    original_cols.extend(evals_to_cols(&blake_output_evals));
    tree_builder.extend_evals(blake_output_evals);

    // Write triple xor 32 component.
    let (triple_xor_32_trace, triple_xor_32_claim, triple_xor_32_interaction_claim_gen) =
        triple_xor_32_state.write_trace(&verify_bitwise_xor_8_state);
    let triple_xor_32_evals = triple_xor_32_trace.to_evals();
    original_cols.extend(evals_to_cols(&triple_xor_32_evals));
    tree_builder.extend_evals(triple_xor_32_evals);

    // Write verify bitwise xor 8 component.
    let (
        verify_bitwise_xor_8_trace,
        _verify_bitwise_xor_8_claim,
        verify_bitwise_xor_8_interaction_claim_gen,
    ) = verify_bitwise_xor_8_state.write_trace();
    let verify_bitwise_xor_8_evals = verify_bitwise_xor_8_trace.to_evals();
    original_cols.extend(evals_to_cols(&verify_bitwise_xor_8_evals));
    tree_builder.extend_evals(verify_bitwise_xor_8_evals);

    let (
        verify_bitwise_xor_12_trace,
        _verify_bitwise_xor_12_claim,
        verify_bitwise_xor_12_interaction_claim_gen,
    ) = verify_bitwise_xor_12_state.write_trace();
    original_cols.extend(evals_to_cols(&verify_bitwise_xor_12_trace));
    tree_builder.extend_evals(verify_bitwise_xor_12_trace);

    // Write verify bitwise xor 4 component.
    let (
        verify_bitwise_xor_4_trace,
        _verify_bitwise_xor_4_claim,
        verify_bitwise_xor_4_interaction_claim_gen,
    ) = verify_bitwise_xor_4_state.write_trace();
    let verify_bitwise_xor_4_evals = verify_bitwise_xor_4_trace.to_evals();
    original_cols.extend(evals_to_cols(&verify_bitwise_xor_4_evals));
    tree_builder.extend_evals(verify_bitwise_xor_4_evals);

    // Write verify bitwise xor 7 component.
    let (
        verify_bitwise_xor_7_trace,
        _verify_bitwise_xor_7_claim,
        verify_bitwise_xor_7_interaction_claim_gen,
    ) = verify_bitwise_xor_7_state.write_trace();
    let verify_bitwise_xor_7_evals = verify_bitwise_xor_7_trace.to_evals();
    original_cols.extend(evals_to_cols(&verify_bitwise_xor_7_evals));
    tree_builder.extend_evals(verify_bitwise_xor_7_evals);

    // Write verify bitwise xor 9 component.
    let (
        verify_bitwise_xor_9_trace,
        _verify_bitwise_xor_9_claim,
        verify_bitwise_xor_9_interaction_claim_gen,
    ) = verify_bitwise_xor_9_state.write_trace();
    let verify_bitwise_xor_9_evals = verify_bitwise_xor_9_trace.to_evals();
    original_cols.extend(evals_to_cols(&verify_bitwise_xor_9_evals));
    tree_builder.extend_evals(verify_bitwise_xor_9_evals);

    // Write range check 15 component.
    let (range_check_15_trace, _range_check_15_claim, range_check_15_interaction_claim_gen) =
        range_check_15_state.write_trace();
    let range_check_15_evals = range_check_15_trace.to_evals();
    original_cols.extend(evals_to_cols(&range_check_15_evals));
    tree_builder.extend_evals(range_check_15_evals);

    // Write range check 16 component.
    let (range_check_16_trace, _range_check_16_claim, range_check_16_interaction_claim_gen) =
        range_check_16_state.write_trace();
    let range_check_16_evals = range_check_16_trace.to_evals();
    original_cols.extend(evals_to_cols(&range_check_16_evals));
    tree_builder.extend_evals(range_check_16_evals);

    (
        CircuitClaim {
            log_sizes: [
                eq_log_size,
                qm31_ops_log_size,
                blake_gate_interaction_claim_gen.log_size,
                blake_round_log_size.log_size,
                circuit_air::components::blake_round_sigma::LOG_SIZE,
                blake_g_claim.log_size,
                blake_output_claim.log_size,
                triple_xor_32_claim.log_size,
                circuit_air::components::verify_bitwise_xor_8::LOG_SIZE,
                circuit_air::components::verify_bitwise_xor_12::LOG_SIZE,
                circuit_air::components::verify_bitwise_xor_4::LOG_SIZE,
                circuit_air::components::verify_bitwise_xor_7::LOG_SIZE,
                circuit_air::components::verify_bitwise_xor_9::LOG_SIZE,
                circuit_air::components::range_check_15::LOG_SIZE,
                circuit_air::components::range_check_16::LOG_SIZE,
            ],
        },
        crate::witness::trace::CircuitInteractionClaimGenerator {
            eq_lookup_data,
            qm31_ops_lookup_data,
            blake_gate: blake_gate_interaction_claim_gen,
            blake_round: blake_round_interaction_claim_gen,
            blake_round_sigma: blake_round_sigma_interaction_claim_gen,
            blake_g: blake_g_interaction_claim_gen,
            blake_output: blake_output_interaction_claim_gen,
            triple_xor_32: triple_xor_32_interaction_claim_gen,
            verify_bitwise_xor_8: verify_bitwise_xor_8_interaction_claim_gen,
            verify_bitwise_xor_12: verify_bitwise_xor_12_interaction_claim_gen,
            verify_bitwise_xor_4: verify_bitwise_xor_4_interaction_claim_gen,
            verify_bitwise_xor_7: verify_bitwise_xor_7_interaction_claim_gen,
            verify_bitwise_xor_9: verify_bitwise_xor_9_interaction_claim_gen,
            range_check_15: range_check_15_interaction_claim_gen,
            range_check_16: range_check_16_interaction_claim_gen,
        },
        original_cols,
    )
}

#[allow(dead_code)]
fn prove_circuit_with_relation_tracker(
    context: &mut Context<QM31>,
) -> (CircuitProof, RelationSummary) {
    finalize_context(context);
    let pcs_config = PcsConfig::default();
    let (preprocessed_trace, params) =
        PreProcessedTrace::generate_preprocessed_trace(&context.circuit);

    let trace_log_size = preprocessed_trace.log_sizes().into_iter().max().unwrap();
    let composition_log_degree_bound: u32 = 1;
    let twiddles = SimdBackend::precompute_twiddles(
        CanonicCoset::new(
            trace_log_size
                + std::cmp::max(
                    pcs_config.fri_config.log_blowup_factor,
                    composition_log_degree_bound,
                ),
        )
        .circle_domain()
        .half_coset,
    );
    let channel = &mut Blake2sM31Channel::default();
    // Mix channel salt. Note that we first reduce it modulo `M31::P`, then cast it as QM31.
    let channel_salt = 0_u32;
    channel.mix_felts(&[channel_salt.into()]);

    let mut commitment_scheme =
        CommitmentSchemeProver::<SimdBackend, Blake2sM31MerkleChannel>::new(pcs_config, &twiddles);

    // Preprocessed trace.
    let mut tree_builder = commitment_scheme.tree_builder();
    tree_builder.extend_evals(preprocessed_trace.get_trace::<SimdBackend>());
    tree_builder.commit(channel);

    // Base trace.
    let mut tree_builder = commitment_scheme.tree_builder();
    let preprocessed_trace_arc = Arc::new(preprocessed_trace);
    let trace_generator = TraceGenerator {
        qm31_ops_trace_generator: qm31_ops::TraceGenerator {
            first_permutation_row: params.first_permutation_row,
        },
    };
    let (claim, interaction_generator, original_cols) = write_trace_with_cols(
        context.values(),
        preprocessed_trace_arc.clone(),
        &trace_generator,
        &mut tree_builder,
    );
    claim.mix_into(channel);
    tree_builder.commit(channel);

    let interaction_pow_nonce = SimdBackend::grind(channel, INTERACTION_POW_BITS);
    channel.mix_u64(interaction_pow_nonce);
    // Draw interaction elements.
    let interaction_elements = CircuitInteractionElements::draw(channel);

    // Interaction trace.
    let mut tree_builder = commitment_scheme.tree_builder();
    let interaction_claim = write_interaction_trace(
        &claim,
        interaction_generator,
        &mut tree_builder,
        &interaction_elements,
    );
    interaction_claim.mix_into(channel);
    tree_builder.commit(channel);

    // Component provers.
    let component_builder = circuit_air::components::CircuitComponents::new(
        &claim,
        &interaction_elements,
        &interaction_claim,
        &preprocessed_trace_arc.ids(),
    );
    let components = component_builder.provers();

    // Relation tracker (all components).
    let preprocessed_cols_m31: Vec<Vec<M31>> = preprocessed_trace_arc
        .columns
        .iter()
        .map(|c| c.iter().copied().map(M31::from).collect())
        .collect();
    let preprocessed_refs: Vec<&Vec<M31>> = preprocessed_cols_m31.iter().collect();
    let original_refs: Vec<&Vec<M31>> = original_cols.iter().collect();
    let trace_tree = TreeVec::new(vec![preprocessed_refs, original_refs, vec![]]);
    let mut entries = vec![];
    entries.extend(add_to_relation_entries(&component_builder.eq, &trace_tree));
    entries.extend(add_to_relation_entries(&component_builder.qm31_ops, &trace_tree));
    entries.extend(add_to_relation_entries(&component_builder.blake_gate, &trace_tree));
    entries.extend(add_to_relation_entries(&component_builder.blake_round, &trace_tree));
    entries.extend(add_to_relation_entries(&component_builder.blake_round_sigma, &trace_tree));
    entries.extend(add_to_relation_entries(&component_builder.blake_g, &trace_tree));
    entries.extend(add_to_relation_entries(&component_builder.blake_output, &trace_tree));
    entries.extend(add_to_relation_entries(&component_builder.triple_xor_32, &trace_tree));
    entries.extend(add_to_relation_entries(&component_builder.verify_bitwise_xor_8, &trace_tree));
    entries.extend(add_to_relation_entries(&component_builder.verify_bitwise_xor_12, &trace_tree));
    entries.extend(add_to_relation_entries(&component_builder.verify_bitwise_xor_4, &trace_tree));
    entries.extend(add_to_relation_entries(&component_builder.verify_bitwise_xor_7, &trace_tree));
    entries.extend(add_to_relation_entries(&component_builder.verify_bitwise_xor_9, &trace_tree));
    entries.extend(add_to_relation_entries(&component_builder.range_check_15, &trace_tree));
    entries.extend(add_to_relation_entries(&component_builder.range_check_16, &trace_tree));
    let summary = RelationSummary::summarize_relations(&entries).cleaned();

    // Prove stark.
    let proof = prove_ex::<SimdBackend, _>(&components, channel, commitment_scheme, true);
    let circuit_proof = CircuitProof {
        components: component_builder.components(),
        claim,
        interaction_claim,
        pcs_config,
        stark_proof: proof,
        interaction_pow_nonce,
        channel_salt,
    };

    (circuit_proof, summary)
}
#[test]
fn test_prove_and_stark_verify_blake_gate_context() {
    let mut blake_gate_context = build_blake_gate_context();
    blake_gate_context.finalize_guessed_vars();
    blake_gate_context.validate_circuit();

    let (
        CircuitProof {
            components,
            claim,
            interaction_claim,
            pcs_config,
            stark_proof,
            interaction_pow_nonce,
            channel_salt,
        },
        preprocessed_trace_sizes,
    ) = prove_circuit(&mut blake_gate_context);
    assert!(stark_proof.is_ok(), "Got error: {}", stark_proof.err().unwrap());
    let proof = stark_proof.unwrap();

    let verifier_channel = &mut Blake2sM31Channel::default();
    verifier_channel.mix_felts(&[channel_salt.into()]);
    pcs_config.mix_into(verifier_channel);
    let commitment_scheme =
        &mut CommitmentSchemeVerifier::<Blake2sM31MerkleChannel>::new(pcs_config);

    // Retrieve the expected column sizes in each commitment interaction, from the AIR.
    let sizes = TreeVec::concat_cols(components.iter().map(|c| c.trace_log_degree_bounds()));

    commitment_scheme.commit(
        proof.proof.commitments[0],
        &preprocessed_trace_sizes,
        verifier_channel,
    );
    claim.mix_into(verifier_channel);
    commitment_scheme.commit(proof.proof.commitments[1], &sizes[1], verifier_channel);

    verifier_channel.verify_pow_nonce(INTERACTION_POW_BITS, interaction_pow_nonce);

    verifier_channel.mix_u64(interaction_pow_nonce);
    let CircuitInteractionElements { common_lookup_elements } =
        CircuitInteractionElements::draw(verifier_channel);

    interaction_claim.mix_into(verifier_channel);

    commitment_scheme.commit(proof.proof.commitments[2], &sizes[2], verifier_channel);
    stwo::core::verifier::verify_ex(
        &components.iter().map(|c| c.as_ref()).collect::<Vec<&dyn Component>>(),
        verifier_channel,
        commitment_scheme,
        proof.proof,
        true,
    )
    .unwrap();

    // Compute the expected logup term. In this case it's only the terms corresponding to blake's
    // IV.
    let yield_sum =
        blake_iv_public_logup_sum(blake_gate_context.circuit.blake.len(), &common_lookup_elements);

    let total_claim_sum: QM31 = interaction_claim.claimed_sums.iter().sum();

    assert_eq!(total_claim_sum, yield_sum);
}

#[test]
fn test_prove_and_stark_verify_permutation_context() {
    let mut permutation_context = build_permutation_context();
    permutation_context.finalize_guessed_vars();
    permutation_context.validate_circuit();

    let (
        CircuitProof {
            pcs_config,
            claim,
            interaction_pow_nonce,
            interaction_claim,
            components,
            stark_proof,
            channel_salt,
        },
        preprocessed_trace_sizes,
    ) = prove_circuit(&mut permutation_context);
    assert!(stark_proof.is_ok());
    let proof = stark_proof.unwrap();

    // Verify.
    let verifier_channel = &mut Blake2sM31Channel::default();
    verifier_channel.mix_felts(&[channel_salt.into()]);
    pcs_config.mix_into(verifier_channel);
    let commitment_scheme =
        &mut CommitmentSchemeVerifier::<Blake2sM31MerkleChannel>::new(pcs_config);

    // Retrieve the expected column sizes in each commitment interaction, from the AIR.
    let sizes = TreeVec::concat_cols(components.iter().map(|c| c.trace_log_degree_bounds()));

    commitment_scheme.commit(
        proof.proof.commitments[0],
        &preprocessed_trace_sizes,
        verifier_channel,
    );
    claim.mix_into(verifier_channel);
    commitment_scheme.commit(proof.proof.commitments[1], &sizes[1], verifier_channel);
    // TODO(Gali): Draw interaction element?
    verifier_channel.verify_pow_nonce(INTERACTION_POW_BITS, interaction_pow_nonce);
    verifier_channel.mix_u64(interaction_pow_nonce);
    interaction_claim.mix_into(verifier_channel);
    commitment_scheme.commit(proof.proof.commitments[2], &sizes[2], verifier_channel);
    stwo::core::verifier::verify_ex(
        &components.iter().map(|c| c.as_ref()).collect::<Vec<&dyn Component>>(),
        verifier_channel,
        commitment_scheme,
        proof.proof,
        true,
    )
    .unwrap();
}

#[test]
fn test_prove_and_stark_verify_fibonacci_context() {
    let mut fibonacci_context = build_fibonacci_context();
    fibonacci_context.finalize_guessed_vars();
    fibonacci_context.validate_circuit();

    let (
        CircuitProof {
            pcs_config,
            claim,
            interaction_pow_nonce,
            interaction_claim,
            components,
            stark_proof,
            channel_salt,
        },
        preprocessed_trace_sizes,
    ) = prove_circuit(&mut fibonacci_context);
    assert!(stark_proof.is_ok());
    let proof = stark_proof.unwrap();

    // Verify.
    let verifier_channel = &mut Blake2sM31Channel::default();
    verifier_channel.mix_felts(&[channel_salt.into()]);
    pcs_config.mix_into(verifier_channel);
    let commitment_scheme =
        &mut CommitmentSchemeVerifier::<Blake2sM31MerkleChannel>::new(pcs_config);

    // Retrieve the expected column sizes in each commitment interaction, from the AIR.
    let sizes = TreeVec::concat_cols(components.iter().map(|c| c.trace_log_degree_bounds()));

    commitment_scheme.commit(
        proof.proof.commitments[0],
        &preprocessed_trace_sizes,
        verifier_channel,
    );
    claim.mix_into(verifier_channel);
    commitment_scheme.commit(proof.proof.commitments[1], &sizes[1], verifier_channel);
    // TODO(Gali): Draw interaction element?
    verifier_channel.verify_pow_nonce(INTERACTION_POW_BITS, interaction_pow_nonce);
    verifier_channel.mix_u64(interaction_pow_nonce);
    interaction_claim.mix_into(verifier_channel);
    commitment_scheme.commit(proof.proof.commitments[2], &sizes[2], verifier_channel);
    stwo::core::verifier::verify_ex(
        &components.iter().map(|c| c.as_ref()).collect::<Vec<&dyn Component>>(),
        verifier_channel,
        commitment_scheme,
        proof.proof,
        true,
    )
    .unwrap();
}

#[test]
#[ignore = "Verifier does not yet support Blake AIR."]
fn test_prove_and_circuit_verify_fibonacci_context() {
    let mut fibonacci_context = build_fibonacci_context();
    fibonacci_context.finalize_guessed_vars();
    fibonacci_context.validate_circuit();

    let (
        CircuitProof {
            pcs_config,
            claim,
            interaction_pow_nonce,
            interaction_claim,
            components: _,
            stark_proof,
            channel_salt,
        },
        _preprocessed_trace_sizes,
    ) = prove_circuit(&mut fibonacci_context);
    assert!(stark_proof.is_ok());
    let proof = stark_proof.unwrap();

    // Verify.
    let statement = CircuitStatement::default();
    let claim = Claim {
        packed_enable_bits: pack_enable_bits(&[true, true]),
        packed_component_log_sizes: pack_component_log_sizes(&claim.log_sizes),
        claimed_sums: interaction_claim.claimed_sums.to_vec(),
    };
    let config = ProofConfig::from_statement(&statement, &pcs_config, INTERACTION_POW_BITS);

    let mut context = TraceContext::default();
    let proof = proof_from_stark_proof(&proof, &config, claim, interaction_pow_nonce, channel_salt);
    let proof_vars = proof.guess(&mut context);

    circuits_stark_verifier::verify::verify(&mut context, &proof_vars, &config, &statement);
    context.check_vars_used();
    context.finalize_guessed_vars();
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_finalize_context() {
    let mut context = build_fibonacci_context();
    finalize_context(&mut context);

    assert!(context.circuit.add.len().is_power_of_two());
    context.validate_circuit();
}
