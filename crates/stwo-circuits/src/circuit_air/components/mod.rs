pub mod qm31_ops;
use crate::circuit_air::relations;
use num_traits::Zero;
use stwo::core::channel::Channel;
use stwo::core::fields::qm31::SecureField;

pub struct CircuitClaim {
    pub qm31_ops: qm31_ops::Claim,
}
impl CircuitClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        let Self { qm31_ops } = self;
        qm31_ops.mix_into(channel);
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
        let Self { qm31_ops } = self;
        qm31_ops.mix_into(channel);
    }
}

pub fn lookup_sum(interaction_claim: &CircuitInteractionClaim) -> SecureField {
    let CircuitInteractionClaim { qm31_ops } = interaction_claim;
    let mut sum = SecureField::zero();
    sum += qm31_ops.claimed_sum;
    sum
}
