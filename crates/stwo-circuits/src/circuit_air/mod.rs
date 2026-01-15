pub mod components;
pub mod preprocessed_columns;
pub mod relations;
pub mod statement;

use crate::circuit_air::components::N_COMPONENTS;
use crate::stark_verifier::proof_from_stark_proof::{pack_component_log_sizes, pack_enable_bits};
use stwo::core::channel::Channel;
use stwo::core::fields::qm31::SecureField;

pub type ComponentLogSize = u32;
pub type ClaimedSum = SecureField;

pub struct CircuitClaim {
    pub log_sizes: [ComponentLogSize; N_COMPONENTS],
}
impl CircuitClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        let Self { log_sizes } = self;

        // mix the enable bits into the channel.
        channel.mix_felts(&pack_enable_bits(&[true, true]));
        channel.mix_felts(&pack_component_log_sizes(log_sizes));
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
