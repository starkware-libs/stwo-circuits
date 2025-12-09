use crate::circuit_air::components::CircuitClaim;
use crate::circuit_air::components::CircuitInteractionClaim;
use crate::circuit_air::components::CircuitInteractionElements;
use crate::circuit_prover::witness::components::qm31_ops;
use crate::circuit_prover::witness::preprocessed::PreProcessedTrace;
use crate::circuit_prover::witness::utils::TreeBuilder;
use stwo::core::fields::qm31::QM31;
use stwo::prover::backend::simd::SimdBackend;

pub fn write_trace(
    values: &[QM31],
    preprocessed_trace: &PreProcessedTrace,
    tree_builder: &mut impl TreeBuilder<SimdBackend>,
) -> (CircuitClaim, CircuitInteractionClaimGenerator) {
    let (qm31_ops_claim, qm31_ops_interaction_claim_generator) =
        qm31_ops::write_trace(values, preprocessed_trace, tree_builder);

    (
        CircuitClaim { qm31_ops: qm31_ops_claim },
        CircuitInteractionClaimGenerator { qm31_ops: qm31_ops_interaction_claim_generator },
    )
}

pub struct CircuitInteractionClaimGenerator {
    pub qm31_ops: qm31_ops::InteractionClaimGenerator,
    // ...
}
impl CircuitInteractionClaimGenerator {
    pub fn write_interaction_trace(
        self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
        interaction_elements: &CircuitInteractionElements,
    ) -> CircuitInteractionClaim {
        let qm31_ops_interaction_claim =
            self.qm31_ops.write_interaction_trace(tree_builder, &interaction_elements.gate);
        CircuitInteractionClaim { qm31_ops: qm31_ops_interaction_claim }
    }
}
