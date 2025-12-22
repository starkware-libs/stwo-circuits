use crate::circuit_air::components::CircuitClaim;
use crate::circuit_air::components::CircuitInteractionClaim;
use crate::circuit_air::components::CircuitInteractionElements;
use crate::circuit_prover::witness::components::qm31_ops;
use crate::circuit_prover::witness::preprocessed::ActiveComponents;
use crate::circuit_prover::witness::preprocessed::PreProcessedTrace;
use crate::circuit_prover::witness::utils::TreeBuilder;
use stwo::core::fields::qm31::QM31;
use stwo::prover::backend::simd::SimdBackend;

pub fn write_trace(
    context_values: &[QM31],
    preprocessed_trace: &PreProcessedTrace,
    tree_builder: &mut impl TreeBuilder<SimdBackend>,
) -> (CircuitClaim, CircuitInteractionClaimGenerator) {
    let components = ActiveComponents::from_preprocessed_trace(preprocessed_trace);

    let (qm31_ops_log_size, qm31_ops_lookup_data) = if components.qm31_ops {
        let (log_size, lookup_data) =
            qm31_ops::write_trace(context_values, preprocessed_trace, tree_builder);
        (Some(log_size), Some(lookup_data))
    } else {
        (None, None)
    };

    (CircuitClaim { qm31_ops_log_size }, CircuitInteractionClaimGenerator { qm31_ops_lookup_data })
}

pub struct CircuitInteractionClaimGenerator {
    pub qm31_ops_lookup_data: Option<qm31_ops::LookupData>,
    // ...
}

pub fn write_interaction_trace(
    circuit_claim: &CircuitClaim,
    circuit_interaction_claim_generator: CircuitInteractionClaimGenerator,
    tree_builder: &mut impl TreeBuilder<SimdBackend>,
    interaction_elements: &CircuitInteractionElements,
) -> CircuitInteractionClaim {
    let qm31_ops_claimed_sum =
        circuit_interaction_claim_generator.qm31_ops_lookup_data.map(|lookup_data| {
            qm31_ops::write_interaction_trace(
                circuit_claim.qm31_ops_log_size.expect("qm31_ops_log_size must be Some"),
                lookup_data,
                tree_builder,
                &interaction_elements.gate,
            )
        });

    CircuitInteractionClaim { qm31_ops_claimed_sum }
}
