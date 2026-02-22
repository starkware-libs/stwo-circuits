pub mod component_utils;
pub mod components;
pub mod preprocessed_columns;
pub mod relations;
pub mod statement;

use crate::components::N_COMPONENTS;
use crate::relations::CommonLookupElements;
use circuits::ivalue::qm31_from_u32s;
use circuits_stark_verifier::proof_from_stark_proof::{pack_component_log_sizes, pack_enable_bits};
use itertools::zip_eq;
use num_traits::Zero;
use stwo::core::channel::Channel;
use stwo::core::fields::FieldExpOps;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo_constraint_framework::Relation;

pub type ComponentLogSize = u32;
pub type ClaimedSum = QM31;

pub struct CircuitClaim {
    pub log_sizes: [ComponentLogSize; N_COMPONENTS],
    pub output_values: Vec<QM31>,
}
impl CircuitClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        let Self { log_sizes, output_values } = self;

        // mix the number of components.
        let n_components = log_sizes.len();
        channel.mix_felts(&[qm31_from_u32s(n_components as u32, 0, 0, 0)]);

        // mix the enable bits into the channel.
        channel.mix_felts(&pack_enable_bits(&[true, true]));
        channel.mix_felts(&pack_component_log_sizes(log_sizes));
        // mix the output values into the channel.
        channel.mix_felts(output_values);
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

pub const BLAKE2S_IV: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];

pub fn blake2s_initial_state() -> [u32; 8] {
    let mut h = BLAKE2S_IV;
    h[0] ^= 0x01010020;
    h
}

fn blake_iv_public_logup_sum(
    n_blake_gates: usize,
    common_lookup_elements: &CommonLookupElements,
) -> QM31 {
    let state_id = M31::from(1061955672);
    let initial_state = blake2s_initial_state();
    let initial_state_limbs = [
        M31::from(initial_state[0] & 0xffff),
        M31::from((initial_state[0] >> 16) & 0xffff),
        M31::from(initial_state[1] & 0xffff),
        M31::from((initial_state[1] >> 16) & 0xffff),
        M31::from(initial_state[2] & 0xffff),
        M31::from((initial_state[2] >> 16) & 0xffff),
        M31::from(initial_state[3] & 0xffff),
        M31::from((initial_state[3] >> 16) & 0xffff),
        M31::from(initial_state[4] & 0xffff),
        M31::from((initial_state[4] >> 16) & 0xffff),
        M31::from(initial_state[5] & 0xffff),
        M31::from((initial_state[5] >> 16) & 0xffff),
        M31::from(initial_state[6] & 0xffff),
        M31::from((initial_state[6] >> 16) & 0xffff),
        M31::from(initial_state[7] & 0xffff),
        M31::from((initial_state[7] >> 16) & 0xffff),
    ];

    let limbs = [
        state_id,
        M31::from(0u32),
        initial_state_limbs[0],
        initial_state_limbs[1],
        initial_state_limbs[2],
        initial_state_limbs[3],
        initial_state_limbs[4],
        initial_state_limbs[5],
        initial_state_limbs[6],
        initial_state_limbs[7],
        initial_state_limbs[8],
        initial_state_limbs[9],
        initial_state_limbs[10],
        initial_state_limbs[11],
        initial_state_limbs[12],
        initial_state_limbs[13],
        initial_state_limbs[14],
        initial_state_limbs[15],
    ];
    let denom: QM31 = common_lookup_elements.combine(&limbs);
    denom.inverse() * M31::from(n_blake_gates)
}

pub fn lookup_sum(
    claim: &CircuitClaim,
    interaction_claim: &CircuitInteractionClaim,
    interaction_elements: &CircuitInteractionElements,
    output_addresses: &[M31],
    n_blake_gates: usize,
) -> QM31 {
    let CircuitInteractionClaim { claimed_sums } = interaction_claim;
    let component_sum: QM31 = claimed_sums.iter().sum();

    // Compute the public logup sum from output gates.
    let mut output_sum = QM31::zero();
    let gate_relation_id = M31::from(378353459);
    for (addr, value) in zip_eq(output_addresses, &claim.output_values) {
        let values = [gate_relation_id, *addr, value.0.0, value.0.1, value.1.0, value.1.1];
        let denom: QM31 = interaction_elements.common_lookup_elements.combine(&values);
        output_sum += denom.inverse();
    }

    // Subtract the blake IV public logup sum (blake IV state is used but never yielded).
    let blake_iv_sum =
        blake_iv_public_logup_sum(n_blake_gates, &interaction_elements.common_lookup_elements);

    component_sum + output_sum - blake_iv_sum
}
