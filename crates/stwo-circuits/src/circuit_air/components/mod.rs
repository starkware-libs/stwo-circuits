pub mod qm31_ops;
use std::marker::PhantomData;

use crate::circuit_air::relations;
use itertools::chain;
use num_traits::Zero;
use stwo::core::air::Component;
use stwo::core::channel::Channel;
use stwo::core::fields::qm31::SecureField;
use stwo::core::pcs::TreeVec;
use stwo::prover::ComponentProver;
use stwo::prover::backend::Backend;
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
        for log_size in self.log_sizes {
            channel.mix_u64(log_size as u64);
        }
    }

    /// Returns the log sizes of the components.
    /// Does not include the preprocessed trace log sizes.
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let log_sizes_list =
            vec![qm31_ops::log_sizes(self.log_sizes[ComponentList::Qm31Ops as usize])];

        TreeVec::concat_cols(log_sizes_list.into_iter())
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
        for claimed_sum in self.claimed_sums {
            channel.mix_felts(&[claimed_sum]);
        }
    }
}

pub fn lookup_sum(interaction_claim: &CircuitInteractionClaim) -> SecureField {
    let mut sum = SecureField::zero();
    for claimed_sum in interaction_claim.claimed_sums {
        sum += claimed_sum;
    }
    sum
}

pub struct CircuitComponents<B: Backend> {
    pub qm31_ops: qm31_ops::Component,
    _backend: PhantomData<B>,
}
impl<B: Backend> CircuitComponents<B>
where
    stwo_constraint_framework::FrameworkComponent<crate::circuit_air::components::qm31_ops::Eval>:
        stwo::prover::ComponentProver<B>,
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

        Self { qm31_ops: qm31_ops_component, _backend: PhantomData }
    }

    pub fn provers(&self) -> Vec<&dyn ComponentProver<B>> {
        chain!([&self.qm31_ops as &dyn ComponentProver<B>,]).collect()
    }

    pub fn components(&self) -> Vec<&dyn Component> {
        self.provers().into_iter().map(|component| component as &dyn Component).collect()
    }
}
