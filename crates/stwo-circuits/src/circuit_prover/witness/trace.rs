use crate::circuit_air::CircuitClaim;
use crate::circuit_air::CircuitInteractionClaim;
use crate::circuit_air::CircuitInteractionElements;
use crate::circuit_prover::witness::components::eq;
use crate::circuit_prover::witness::components::qm31_ops;
use crate::circuit_prover::witness::preprocessed::PreProcessedTrace;
use crate::circuit_prover::witness::utils::TreeBuilder;
use stwo::core::fields::qm31::QM31;
use stwo::prover::backend::simd::SimdBackend;

pub fn write_trace(
    context_values: &[QM31],
    preprocessed_trace: &PreProcessedTrace,
    tree_builder: &mut impl TreeBuilder<SimdBackend>,
) -> (CircuitClaim, CircuitInteractionClaimGenerator) {
    let (qm31_ops_log_size, qm31_ops_lookup_data) =
        qm31_ops::write_trace(context_values, preprocessed_trace, tree_builder);
    let (eq_log_size, eq_lookup_data) =
        eq::write_trace(context_values, preprocessed_trace, tree_builder);

    (
        CircuitClaim { qm31_ops_log_size, eq_log_size },
        CircuitInteractionClaimGenerator { qm31_ops_lookup_data, eq_lookup_data },
    )
}

pub struct CircuitInteractionClaimGenerator {
    pub qm31_ops_lookup_data: qm31_ops::LookupData,
    pub eq_lookup_data: eq::LookupData,
    // ...
}

pub fn write_interaction_trace(
    circuit_claim: &CircuitClaim,
    circuit_interaction_claim_generator: CircuitInteractionClaimGenerator,
    tree_builder: &mut impl TreeBuilder<SimdBackend>,
    interaction_elements: &CircuitInteractionElements,
) -> CircuitInteractionClaim {
    let qm31_ops_claimed_sum = qm31_ops::write_interaction_trace(
        circuit_claim.qm31_ops_log_size,
        circuit_interaction_claim_generator.qm31_ops_lookup_data,
        tree_builder,
        &interaction_elements.gate,
    );
    let eq_claimed_sum = eq::write_interaction_trace(
        circuit_claim.eq_log_size,
        circuit_interaction_claim_generator.eq_lookup_data,
        tree_builder,
        &interaction_elements.gate,
    );
    CircuitInteractionClaim { qm31_ops_claimed_sum, eq_claimed_sum }
}
