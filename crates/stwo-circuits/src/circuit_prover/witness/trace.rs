use std::sync::Arc;

use crate::circuit_air::CircuitClaim;
use crate::circuit_air::CircuitInteractionClaim;
use crate::circuit_air::CircuitInteractionElements;
use crate::circuit_prover::witness::components::blake_g;
use crate::circuit_prover::witness::components::blake_gate;
use crate::circuit_prover::witness::components::blake_message;
use crate::circuit_prover::witness::components::blake_round;
use crate::circuit_prover::witness::components::blake_round_sigma;
use crate::circuit_prover::witness::components::eq;
use crate::circuit_prover::witness::components::qm31_ops;
use crate::circuit_prover::witness::components::range_check_15;
use crate::circuit_prover::witness::components::range_check_16;
use crate::circuit_prover::witness::components::triple_xor_32;
use crate::circuit_prover::witness::components::verify_bitwise_xor_8;
use crate::circuit_prover::witness::preprocessed::PreProcessedTrace;
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::prover::TreeBuilder;
use stwo::prover::backend::simd::SimdBackend;

pub struct TraceGenerator {
    pub qm31_ops_trace_generator: qm31_ops::TraceGenerator,
}

pub fn write_trace(
    context_values: &[QM31],
    preprocessed_trace: Arc<PreProcessedTrace>,
    tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sM31MerkleChannel>,
    trace_generator: &TraceGenerator,
) -> (CircuitClaim, CircuitInteractionClaimGenerator) {
    let preprocessed_trace_ref = preprocessed_trace.as_ref();
    let (eq_trace, eq_log_size, eq_lookup_data) =
        eq::write_trace(context_values, preprocessed_trace_ref);
    tree_builder.extend_evals(eq_trace.to_evals());

    let (qm31_ops_trace, qm31_ops_log_size, qm31_ops_lookup_data) = qm31_ops::write_trace(
        context_values,
        preprocessed_trace_ref,
        &trace_generator.qm31_ops_trace_generator,
    );
    tree_builder.extend_evals(qm31_ops_trace.to_evals());

    let blake_gate_claim_generator = blake_gate::ClaimGenerator::new(preprocessed_trace.clone());
    let verify_bitwise_xor_8_state =
        verify_bitwise_xor_8::ClaimGenerator::new(preprocessed_trace.clone());
    let range_check_16_state = range_check_16::ClaimGenerator::new(preprocessed_trace.clone());
    let range_check_15_state = range_check_15::ClaimGenerator::new(preprocessed_trace.clone());
    let mut triple_xor_32_state = triple_xor_32::ClaimGenerator::new();
    let mut blake_round_state = blake_round::ClaimGenerator::default();
    let (blake_gate_trace, blake_gate_log_size, blake_gate_lookup_data, blake_message_state) =
        blake_gate_claim_generator.write_trace(
            context_values,
            preprocessed_trace_ref,
            &verify_bitwise_xor_8_state,
            &range_check_16_state,
            &range_check_15_state,
            &mut blake_round_state,
            &mut triple_xor_32_state,
        );
    tree_builder.extend_evals(blake_gate_trace.to_evals());

    // create blake round sigma state and blake_G
    let blake_round_sigma_state =
        blake_round_sigma::ClaimGenerator::new(preprocessed_trace.clone());
    let mut blake_g_state = blake_g::ClaimGenerator::new();
    // now blake_round write trace
    blake_round_state.write_trace(
        &blake_round_sigma_state,
        &blake_message_state,
        &mut blake_g_state,
    );

    (
        CircuitClaim { log_sizes: [eq_log_size, qm31_ops_log_size, blake_gate_log_size] },
        CircuitInteractionClaimGenerator {
            eq_lookup_data,
            qm31_ops_lookup_data,
            blake_gate_lookup_data,
        },
    )
}

pub struct CircuitInteractionClaimGenerator {
    pub eq_lookup_data: eq::LookupData,
    pub qm31_ops_lookup_data: qm31_ops::LookupData,
    pub blake_gate_lookup_data: blake_gate::LookupData,
}

pub fn write_interaction_trace(
    circuit_claim: &CircuitClaim,
    circuit_interaction_claim_generator: CircuitInteractionClaimGenerator,
    tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sM31MerkleChannel>,
    interaction_elements: &CircuitInteractionElements,
) -> CircuitInteractionClaim {
    let CircuitClaim { log_sizes } = circuit_claim;
    let mut component_log_size_iter = log_sizes.iter();

    let (eq_trace, eq_claimed_sum) = eq::write_interaction_trace(
        *component_log_size_iter.next().unwrap(),
        circuit_interaction_claim_generator.eq_lookup_data,
        &interaction_elements.common_lookup_elements,
    );
    tree_builder.extend_evals(eq_trace);
    let (qm31_ops_trace, qm31_ops_claimed_sum) = qm31_ops::write_interaction_trace(
        *component_log_size_iter.next().unwrap(),
        circuit_interaction_claim_generator.qm31_ops_lookup_data,
        &interaction_elements.common_lookup_elements,
    );
    tree_builder.extend_evals(qm31_ops_trace);

    let blake_gate_log_size = *component_log_size_iter.next().unwrap();
    let blake_gate_n_rows = 1 << blake_gate_log_size;
    let blake_gate_interaction_generator = blake_gate::InteractionClaimGenerator {
        n_rows: blake_gate_n_rows,
        log_size: blake_gate_log_size,
        lookup_data: circuit_interaction_claim_generator.blake_gate_lookup_data,
    };
    let (blake_gate_trace, blake_gate_interaction_claim) = blake_gate_interaction_generator
        .write_interaction_trace(&interaction_elements.common_lookup_elements);
    tree_builder.extend_evals(blake_gate_trace);

    CircuitInteractionClaim {
        claimed_sums: [
            eq_claimed_sum,
            qm31_ops_claimed_sum,
            blake_gate_interaction_claim.claimed_sum,
        ],
    }
}
