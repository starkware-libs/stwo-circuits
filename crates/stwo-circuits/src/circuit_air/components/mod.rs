pub mod eq;
pub mod qm31_ops;

use crate::circuit_air::relations;
use itertools::chain;
use stwo::core::air::Component;
use stwo::core::channel::Channel;
use stwo::core::fields::qm31::SecureField;
use stwo::prover::ComponentProver;
use stwo::prover::backend::simd::SimdBackend;
use stwo_constraint_framework::TraceLocationAllocator;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

pub enum ComponentList {
    Qm31Ops,
}
pub const N_COMPONENTS: usize = std::mem::variant_count::<ComponentList>();

pub type ComponentLogSize = u32;
pub type ClaimedSum = SecureField;

pub struct CircuitClaim {
    pub log_sizes: [ComponentLogSize; N_COMPONENTS],
}
impl CircuitClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        let Self { log_sizes } = self;
        channel.mix_u32s(log_sizes);
    }
}

pub struct CircuitInteractionElements {
    pub gate: relations::Gate,
}
impl CircuitInteractionElements {
    pub fn draw(channel: &mut impl Channel) -> CircuitInteractionElements {
        CircuitInteractionElements { gate: relations::Gate::draw(channel) }
    }
}

pub struct CircuitInteractionClaim {
    pub claimed_sums: [ClaimedSum; N_COMPONENTS],
}
impl CircuitInteractionClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        let Self { claimed_sums } = self;
        channel.mix_felts(claimed_sums);
    }
}

pub fn lookup_sum(interaction_claim: &CircuitInteractionClaim) -> SecureField {
    let CircuitInteractionClaim { claimed_sums } = interaction_claim;
    claimed_sums.iter().sum()
}

pub struct CircuitComponents {
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

        let qm31_ops_component = qm31_ops::Component::new(
            tree_span_provider,
            qm31_ops::Eval {
                log_size: circuit_claim.log_sizes[ComponentList::Qm31Ops as usize],
                gate_lookup_elements: interaction_elements.gate.clone(),
            },
            interaction_claim.claimed_sums[ComponentList::Qm31Ops as usize],
        );

        Self { qm31_ops: qm31_ops_component }
    }

    pub fn provers(&self) -> Vec<&dyn ComponentProver<SimdBackend>> {
        chain!([&self.qm31_ops as &dyn ComponentProver<SimdBackend>,]).collect()
    }

    pub fn components(&self) -> Vec<&dyn Component> {
        self.provers().into_iter().map(|component| component as &dyn Component).collect()
    }
}
