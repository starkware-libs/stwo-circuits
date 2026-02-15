use crate::circuit_air::CircuitClaim;
use crate::circuit_air::CircuitInteractionClaim;
use crate::circuit_air::CircuitInteractionElements;
use crate::circuit_prover::witness::components::eq;
use crate::circuit_prover::witness::components::qm31_ops;
use crate::circuit_prover::witness::preprocessed::PreProcessedTrace;
use crate::circuits::context::Context;
use itertools::Itertools;
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::prover::TreeBuilder;
use stwo::prover::backend::simd::SimdBackend;

pub struct TraceGenerator {
    pub qm31_ops_trace_generator: qm31_ops::TraceGenerator,
}

pub fn write_trace(
    context: &Context<QM31>,
    preprocessed_trace: &PreProcessedTrace,
    tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sM31MerkleChannel>,
    trace_generator: &TraceGenerator,
) -> (CircuitClaim, CircuitInteractionClaimGenerator) {
    let context_values = context.values();
    let (eq_log_size, eq_lookup_data) =
        eq::write_trace(context_values, preprocessed_trace, tree_builder);
    let (qm31_ops_log_size, qm31_ops_lookup_data) = qm31_ops::write_trace(
        context_values,
        preprocessed_trace,
        tree_builder,
        &trace_generator.qm31_ops_trace_generator,
    );

    (
        CircuitClaim {
            log_sizes: [eq_log_size, qm31_ops_log_size],
            outputs: context
                .circuit
                .output
                .iter()
                .map(|out| (out.in0.into(), context_values[out.in0].to_m31_array()))
                .collect_vec(),
        },
        CircuitInteractionClaimGenerator { eq_lookup_data, qm31_ops_lookup_data },
    )
}

pub struct CircuitInteractionClaimGenerator {
    pub eq_lookup_data: eq::LookupData,
    pub qm31_ops_lookup_data: qm31_ops::LookupData,
}

pub fn write_interaction_trace(
    circuit_claim: &CircuitClaim,
    circuit_interaction_claim_generator: CircuitInteractionClaimGenerator,
    tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sM31MerkleChannel>,
    interaction_elements: &CircuitInteractionElements,
) -> CircuitInteractionClaim {
    let CircuitClaim { log_sizes, outputs: _ } = circuit_claim;
    let mut component_log_size_iter = log_sizes.iter();

    let eq_claimed_sum = eq::write_interaction_trace(
        *component_log_size_iter.next().unwrap(),
        circuit_interaction_claim_generator.eq_lookup_data,
        tree_builder,
        &interaction_elements.common_lookup_elements,
    );
    let qm31_ops_claimed_sum = qm31_ops::write_interaction_trace(
        *component_log_size_iter.next().unwrap(),
        circuit_interaction_claim_generator.qm31_ops_lookup_data,
        tree_builder,
        &interaction_elements.common_lookup_elements,
    );
    CircuitInteractionClaim { claimed_sums: [eq_claimed_sum, qm31_ops_claimed_sum] }
}
