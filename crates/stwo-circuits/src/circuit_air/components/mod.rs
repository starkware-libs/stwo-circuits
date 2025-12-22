pub mod qm31_ops;
use crate::circuit_air::relations;
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
    pub qm31_ops_log_size: Option<u32>,
    // ...
}
impl CircuitClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_u64(self.qm31_ops_log_size.unwrap_or(0) as u64);
    }

    /// Returns the log sizes of the components.
    /// Does not include the preprocessed trace log sizes.
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let mut log_sizes_list = vec![];

        if let Some(log_size) = self.qm31_ops_log_size {
            log_sizes_list.push(qm31_ops::log_sizes(log_size));
        }

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
    pub qm31_ops_claimed_sum: Option<SecureField>,
    // ...
}
impl CircuitInteractionClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_felts(&[self.qm31_ops_claimed_sum.unwrap_or(SecureField::zero())]);
    }
}

pub fn lookup_sum(
    _claim: &CircuitClaim,
    _interaction_elements: &CircuitInteractionElements,
    interaction_claim: &CircuitInteractionClaim,
) -> SecureField {
    let mut sum = SecureField::zero();

    if let Some(claimed_sum) = interaction_claim.qm31_ops_claimed_sum {
        sum += claimed_sum;
    }

    sum
}

pub struct CircuitComponents {
    pub qm31_ops: Option<qm31_ops::Component>,
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

        let qm31_ops_component = circuit_claim.qm31_ops_log_size.map(|log_size| {
            qm31_ops::Component::new(
                tree_span_provider,
                qm31_ops::Eval {
                    log_size,
                    gate_lookup_elements: interaction_elements.gate.clone(),
                },
                interaction_claim.qm31_ops_claimed_sum.expect("qm31_ops_claimed_sum must be Some"),
            )
        });

        Self { qm31_ops: qm31_ops_component }
    }

    pub fn provers(&self) -> Vec<&dyn ComponentProver<SimdBackend>> {
        let mut provers: Vec<&dyn ComponentProver<SimdBackend>> = Vec::new();

        if let Some(qm31_ops) = &self.qm31_ops {
            provers.push(qm31_ops as &dyn ComponentProver<SimdBackend>);
        }

        provers
    }

    pub fn components(&self) -> Vec<&dyn Component> {
        self.provers().into_iter().map(|component| component as &dyn Component).collect()
    }
}

impl std::fmt::Display for CircuitComponents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "CircuitComponents")?;
        if let Some(qm31_ops) = &self.qm31_ops {
            writeln!(f, "Qm31Ops: {qm31_ops}")?;
        }
        Ok(())
    }
}
