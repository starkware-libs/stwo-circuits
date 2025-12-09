pub mod qm31_ops;
use crate::circuit_air::relations;
use num_traits::Zero;
use stwo::core::channel::Channel;
use stwo::core::fields::qm31::SecureField;
use stwo::core::pcs::TreeVec;

pub struct CircuitClaim {
    pub qm31_ops: qm31_ops::Claim,
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
}
impl CircuitInteractionClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        self.qm31_ops.mix_into(channel);
    }
}

pub fn lookup_sum(interaction_claim: &CircuitInteractionClaim) -> SecureField {
    let mut sum = SecureField::zero();
    sum += interaction_claim.qm31_ops.claimed_sum;
    sum
}
