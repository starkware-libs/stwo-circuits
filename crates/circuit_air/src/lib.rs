pub mod component_utils;
pub mod components;
pub mod preprocessed_columns;
pub mod relations;
pub mod sample_evaluations;
pub mod statement;
pub mod test;

use crate::components::N_COMPONENTS;
use circuits::ivalue::qm31_from_u32s;
use circuits_stark_verifier::proof_from_stark_proof::{pack_component_log_sizes, pack_enable_bits};
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

        // mix the number of components.
        let n_components = log_sizes.len();
        channel.mix_felts(&[qm31_from_u32s(n_components as u32, 0, 0, 0)]);

        // mix the enable bits into the channel.
        channel.mix_felts(&pack_enable_bits(&[true, true]));
        channel.mix_felts(&pack_component_log_sizes(log_sizes));
        // public claim is empty.
        channel.mix_felts(&[]);
    }
}

pub struct CircuitInteractionElements {
    pub common_lookup_elements: relations::CommonLookupElements,
}
impl CircuitInteractionElements {
    pub fn draw(channel: &mut impl Channel) -> CircuitInteractionElements {
        CircuitInteractionElements {
            common_lookup_elements: relations::CommonLookupElements::draw(channel),
        }
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
