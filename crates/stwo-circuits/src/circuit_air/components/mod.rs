pub mod eq;
pub mod qm31_ops;

use crate::circuit_air::relations;
use itertools::chain;
use num_traits::Zero;
use stwo::core::air::Component;
use stwo::core::channel::Channel;
use stwo::core::fields::qm31::SecureField;
use stwo::core::pcs::TreeVec;
use stwo::prover::ComponentProver;
use stwo::prover::backend::simd::SimdBackend;
use stwo_constraint_framework::TraceLocationAllocator;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

pub struct CircuitClaim {
    pub qm31_ops_log_size: u32,
    pub eq_log_size: u32,
    // ...
}
impl CircuitClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_u64(self.qm31_ops_log_size as u64);
        channel.mix_u64(self.eq_log_size as u64);
    }

    /// Returns the log sizes of the components.
    /// Does not include the preprocessed trace log sizes.
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let log_sizes_list =
            vec![qm31_ops::log_sizes(self.qm31_ops_log_size), eq::log_sizes(self.eq_log_size)];

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
    pub qm31_ops_claimed_sum: SecureField,
    pub eq_claimed_sum: SecureField,
    // ...
}
impl CircuitInteractionClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_felts(&[self.qm31_ops_claimed_sum]);
        channel.mix_felts(&[self.eq_claimed_sum]);
    }
}

pub fn lookup_sum(
    _claim: &CircuitClaim,
    _interaction_elements: &CircuitInteractionElements,
    interaction_claim: &CircuitInteractionClaim,
) -> SecureField {
    let mut sum = SecureField::zero();

    sum += interaction_claim.qm31_ops_claimed_sum;
    sum += interaction_claim.eq_claimed_sum;

    sum
}

pub struct CircuitComponents {
    pub qm31_ops: qm31_ops::Component,
    pub eq: eq::Component,
    // ...
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
                log_size: circuit_claim.qm31_ops_log_size,
                gate_lookup_elements: interaction_elements.gate.clone(),
            },
            interaction_claim.qm31_ops_claimed_sum,
        );
        let eq_component = eq::Component::new(
            tree_span_provider,
            eq::Eval {
                log_size: circuit_claim.eq_log_size,
                gate_lookup_elements: interaction_elements.gate.clone(),
            },
            interaction_claim.eq_claimed_sum,
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

    pub fn components(&self) -> Vec<&dyn Component> {
        self.provers().into_iter().map(|component| component as &dyn Component).collect()
    }
}

impl std::fmt::Display for CircuitComponents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "CircuitComponents")?;
        writeln!(f, "Qm31Ops: {}", self.qm31_ops)?;
        writeln!(f, "Eq: {}", self.eq)?;
        Ok(())
    }
}
