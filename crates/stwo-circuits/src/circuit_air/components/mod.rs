pub mod eq;
pub mod prelude;
pub mod qm31_ops;

use crate::circuit_air::{CircuitClaim, CircuitInteractionClaim, CircuitInteractionElements};
use itertools::chain;
use stwo::core::air::Component;
use stwo::prover::ComponentProver;
use stwo::prover::backend::simd::SimdBackend;
use stwo_constraint_framework::TraceLocationAllocator;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

pub enum ComponentList {
    Eq,
    Qm31Ops,
}
pub const N_COMPONENTS: usize = std::mem::variant_count::<ComponentList>();

pub struct CircuitComponents {
    pub eq: eq::Component,
    pub qm31_ops: qm31_ops::Component,
}
impl CircuitComponents {
    pub fn new(
        circuit_claim: &CircuitClaim,
        interaction_elements: &CircuitInteractionElements,
        interaction_claim: &CircuitInteractionClaim,
        // Describes the structure of the preprocessed trace. Sensitive to order.
        preprocessed_column_ids: &[PreProcessedColumnId],
    ) -> Self {
        let tree_span_provider =
            &mut TraceLocationAllocator::new_with_preprocessed_columns(preprocessed_column_ids);

        let eq_component = eq::Component::new(
            tree_span_provider,
            eq::Eval {
                log_size: circuit_claim.log_sizes[ComponentList::Eq as usize],
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::Eq as usize],
        );
        let qm31_ops_component = qm31_ops::Component::new(
            tree_span_provider,
            qm31_ops::Eval {
                log_size: circuit_claim.log_sizes[ComponentList::Qm31Ops as usize],
                common_lookup_elements: interaction_elements.common_lookup_elements.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::Qm31Ops as usize],
        );
        Self { eq: eq_component, qm31_ops: qm31_ops_component }
    }

    pub fn provers(&self) -> Vec<&dyn ComponentProver<SimdBackend>> {
        chain!([
            &self.eq as &dyn ComponentProver<SimdBackend>,
            &self.qm31_ops as &dyn ComponentProver<SimdBackend>,
        ])
        .collect()
    }

    pub fn components(self) -> Vec<Box<dyn Component>> {
        vec![Box::new(self.eq) as Box<dyn Component>, Box::new(self.qm31_ops) as Box<dyn Component>]
    }
}
pub mod add_mul_gate;
pub mod blake_g;
pub mod blake_gate;
pub mod blake_output;
pub mod blake_round;
pub mod blake_round_sigma;
pub mod range_check_15;
pub mod range_check_16;
pub mod subroutines;
pub mod triple_xor_32;
pub mod verify_bitwise_xor_4;
pub mod verify_bitwise_xor_7;
pub mod verify_bitwise_xor_8;
pub mod verify_bitwise_xor_9;
