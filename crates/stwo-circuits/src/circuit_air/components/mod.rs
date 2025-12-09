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
    pub qm31_ops: qm31_ops::Claim,
    // ...
}
impl CircuitClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        self.qm31_ops.mix_into(channel);
    }

    /// Returns the log sizes of the components.
    /// Does not include the preprocessed trace log sizes.
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let log_sizes_list = vec![self.qm31_ops.log_sizes()];

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
    pub qm31_ops: qm31_ops::InteractionClaim,
    // ...
}
impl CircuitInteractionClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        self.qm31_ops.mix_into(channel);
    }
}

pub fn lookup_sum(
    _claim: &CircuitClaim,
    _interaction_elements: &CircuitInteractionElements,
    interaction_claim: &CircuitInteractionClaim,
) -> SecureField {
    let mut sum = SecureField::zero();

    sum += interaction_claim.qm31_ops.claimed_sum;

    sum
}

pub struct CircuitComponents {
    pub qm31_ops: qm31_ops::Component,
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
                claim: circuit_claim.qm31_ops,
                gate_lookup_elements: interaction_elements.gate.clone(),
            },
            interaction_claim.qm31_ops.claimed_sum,
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

impl std::fmt::Display for CircuitComponents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "CircuitComponents")?;
        writeln!(f, "Qm31Ops: {}", self.qm31_ops)?;
        Ok(())
    }
}
