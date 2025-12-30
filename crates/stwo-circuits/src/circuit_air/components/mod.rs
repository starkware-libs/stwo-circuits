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
    Qm31Ops,
    Eq,
}
pub const N_COMPONENTS: usize = std::mem::variant_count::<ComponentList>();

pub struct CircuitComponents {
    pub qm31_ops: qm31_ops::Component,
    pub eq: eq::Component,
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

        let qm31_ops_component = qm31_ops::Component::new(
            tree_span_provider,
            qm31_ops::Eval {
                log_size: circuit_claim.log_sizes[ComponentList::Qm31Ops as usize],
                gate_lookup_elements: interaction_elements.gate.clone(),
                preprocessed_column_indices: [0, 1, 2, 3, 4, 5, 6, 7],
            },
            interaction_claim.claimed_sums[ComponentList::Qm31Ops as usize],
        );
        let eq_component = eq::Component::new(
            tree_span_provider,
            eq::Eval {
                log_size: circuit_claim.log_sizes[ComponentList::Eq as usize],
                gate_lookup_elements: interaction_elements.gate.clone(),
                preprocessed_column_indices: [8, 9],
            },
            interaction_claim.claimed_sums[ComponentList::Eq as usize],
        );
        Self { qm31_ops: qm31_ops_component, eq: eq_component }
    }

    pub fn provers(&self) -> Vec<&dyn ComponentProver<SimdBackend>> {
        chain!([
            &self.qm31_ops as &dyn ComponentProver<SimdBackend>,
            &self.eq as &dyn ComponentProver<SimdBackend>,
        ])
        .collect()
    }

    pub fn components(self) -> Vec<Box<dyn Component>> {
        vec![Box::new(self.qm31_ops) as Box<dyn Component>, Box::new(self.eq) as Box<dyn Component>]
    }
}
