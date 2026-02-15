pub mod components;
pub mod preprocessed_columns;
pub mod relations;
pub mod statement;

use crate::circuit_air::components::N_COMPONENTS;
use crate::circuits::ivalue::qm31_from_u32s;
use crate::stark_verifier::proof_from_stark_proof::{
    pack_component_log_sizes, pack_enable_bits, pack_public_claim,
};
use itertools::Itertools;
use stwo::core::channel::Channel;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

pub type ComponentLogSize = u32;
pub type ClaimedSum = QM31;

pub struct CircuitClaim {
    pub log_sizes: [ComponentLogSize; N_COMPONENTS],
    /// Output gate data: (address, value) for each output gate in the circuit.
    /// Each entry stores the variable index and the 4 M31 components of its QM31 value.
    pub outputs: Vec<(M31, QM31)>,
}
impl CircuitClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        let Self { log_sizes, outputs } = self;

        // mix the number of components.
        let n_components = log_sizes.len();
        channel.mix_felts(&[qm31_from_u32s(n_components as u32, 0, 0, 0)]);

        // mix the enable bits into the channel.
        channel.mix_felts(&pack_enable_bits(&[true, true]));
        channel.mix_felts(&pack_component_log_sizes(log_sizes));

        // Mix the output gates data into the channel.
        let output_addresses: Vec<M31> = outputs.iter().map(|(addr, _)| *addr).collect_vec();
        let output_values: Vec<QM31> = outputs.iter().map(|(_, value)| *value).collect_vec();
        channel.mix_felts(&pack_public_claim(&output_addresses));
        channel.mix_felts(&output_values);
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

pub fn lookup_sum(interaction_claim: &CircuitInteractionClaim) -> QM31 {
    let CircuitInteractionClaim { claimed_sums } = interaction_claim;
    claimed_sums.iter().sum()
}
