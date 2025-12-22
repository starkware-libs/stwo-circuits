pub mod eq;
pub mod prelude;
pub mod qm31_ops;

use std::marker::PhantomData;

use crate::circuit_air::{CircuitClaim, CircuitInteractionClaim, CircuitInteractionElements};
use itertools::chain;
use stwo::core::air::Component;
use stwo::prover::ComponentProver;
use stwo::prover::backend::Backend;
use stwo_constraint_framework::TraceLocationAllocator;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

pub enum ComponentList {
    Qm31Ops,
    Eq,
}
pub const N_COMPONENTS: usize = std::mem::variant_count::<ComponentList>();

pub struct CircuitComponents<B: Backend> {
    pub qm31_ops: qm31_ops::Component,
    pub eq: eq::Component,
    _backend: PhantomData<B>,
}
impl<B: Backend> CircuitComponents<B>
where
    stwo_constraint_framework::FrameworkComponent<qm31_ops::Eval>: stwo::prover::ComponentProver<B>,
    stwo_constraint_framework::FrameworkComponent<eq::Eval>: stwo::prover::ComponentProver<B>,
{
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
            },
            interaction_claim.claimed_sums[ComponentList::Qm31Ops as usize],
        );
        let eq_component = eq::Component::new(
            tree_span_provider,
            eq::Eval {
                log_size: circuit_claim.log_sizes[ComponentList::Eq as usize],
                gate_lookup_elements: interaction_elements.gate.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::Eq as usize],
        );
        Self { qm31_ops: qm31_ops_component, eq: eq_component, _backend: PhantomData }
    }

    pub fn provers(&self) -> Vec<&dyn ComponentProver<B>> {
        chain!([&self.qm31_ops as &dyn ComponentProver<B>, &self.eq as &dyn ComponentProver<B>,])
            .collect()
    }

    pub fn components(&self) -> Vec<&dyn Component> {
        self.provers().into_iter().map(|component| component as &dyn Component).collect()
    }
}
