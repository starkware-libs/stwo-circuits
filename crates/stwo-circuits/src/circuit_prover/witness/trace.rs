use crate::circuit_air::CircuitClaim;
use crate::circuit_air::CircuitInteractionClaim;
use crate::circuit_air::CircuitInteractionElements;
use crate::circuit_prover::witness::components::eq;
use crate::circuit_prover::witness::components::qm31_ops;
use crate::circuit_prover::witness::preprocessed::PreProcessedTrace;
use stwo::core::fields::qm31::QM31;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::prover::TreeBuilder;
use stwo::prover::backend::simd::SimdBackend;

pub fn write_trace(
    context_values: &[QM31],
    preprocessed_trace: &PreProcessedTrace,
    tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sM31MerkleChannel>,
) -> (CircuitClaim, CircuitInteractionClaimGenerator) {
    let (qm31_ops_log_size, qm31_ops_lookup_data) =
        qm31_ops::write_trace(context_values, preprocessed_trace, tree_builder);
    let (eq_log_size, eq_lookup_data) =
        eq::write_trace(context_values, preprocessed_trace, tree_builder);

    (
        CircuitClaim { log_sizes: [qm31_ops_log_size, eq_log_size] },
        CircuitInteractionClaimGenerator { qm31_ops_lookup_data, eq_lookup_data },
    )
}

pub struct CircuitInteractionClaimGenerator {
    pub qm31_ops_lookup_data: qm31_ops::LookupData,
    pub eq_lookup_data: eq::LookupData,
}

pub fn write_interaction_trace(
    circuit_claim: &CircuitClaim,
    circuit_interaction_claim_generator: CircuitInteractionClaimGenerator,
    tree_builder: &mut TreeBuilder<'_, '_, SimdBackend, Blake2sM31MerkleChannel>,
    interaction_elements: &CircuitInteractionElements,
) -> CircuitInteractionClaim {
    let CircuitClaim { log_sizes } = circuit_claim;
    let mut component_log_size_iter = log_sizes.iter();

    let qm31_ops_claimed_sum = qm31_ops::write_interaction_trace(
        *component_log_size_iter.next().unwrap(),
        circuit_interaction_claim_generator.qm31_ops_lookup_data,
        tree_builder,
        &interaction_elements.gate,
    );
    let eq_claimed_sum = eq::write_interaction_trace(
        *component_log_size_iter.next().unwrap(),
        circuit_interaction_claim_generator.eq_lookup_data,
        tree_builder,
        &interaction_elements.gate,
    );
    CircuitInteractionClaim { claimed_sums: [qm31_ops_claimed_sum, eq_claimed_sum] }
}
