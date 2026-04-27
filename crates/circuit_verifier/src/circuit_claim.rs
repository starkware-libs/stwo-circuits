use std::iter::repeat_n;

use crate::blake2s_consts::blake2s_initial_state;
use crate::circuit_components::N_COMPONENTS;
use crate::relations::{BLAKE_STATE_RELATION_ID, CommonLookupElements, GATE_RELATION_ID};
use crate::statement::all_circuit_components;
use circuits::ivalue::{NoValue, qm31_from_u32s};
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

#[derive(Debug, PartialEq)]
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
        channel.mix_felts(&pack_enable_bits(&[true; N_COMPONENTS]));
        channel.mix_felts(&pack_component_log_sizes(log_sizes));
        // mix the output values into the channel.
        channel.mix_felts(output_values);
    }

    /// Returns `[trace_log_sizes, interaction_log_sizes]` for `tree[1]` and `tree[2]`,
    /// in the order the prover commits columns. Each component contributes its
    /// `log_size` repeated by its number of trace and interaction columns respectively.
    pub fn column_log_sizes_per_tree(&self) -> [Vec<u32>; 2] {
        let Self {
            log_sizes:
                [
                    eq_log_size,
                    qm31_ops_log_size,
                    blake_gate_log_size,
                    blake_round_log_size,
                    blake_round_sigma_log_size,
                    blake_g_log_size,
                    blake_output_log_size,
                    triple_xor_32_log_size,
                    m_31_to_u_32_log_size,
                    verify_bitwise_xor_8_log_size,
                    verify_bitwise_xor_12_log_size,
                    verify_bitwise_xor_4_log_size,
                    verify_bitwise_xor_7_log_size,
                    verify_bitwise_xor_9_log_size,
                    range_check_15_log_size,
                    range_check_16_log_size,
                ],
            ..
        } = self;
        let components: Vec<_> = all_circuit_components::<NoValue>().into_values().collect();
        let Ok(
            [
                eq_component,
                qm31_ops_component,
                blake_gate_component,
                blake_round_component,
                blake_round_sigma_component,
                blake_g_component,
                blake_output_component,
                triple_xor_32_component,
                m_31_to_u_32_component,
                verify_bitwise_xor_8_component,
                verify_bitwise_xor_12_component,
                verify_bitwise_xor_4_component,
                verify_bitwise_xor_7_component,
                verify_bitwise_xor_9_component,
                range_check_15_component,
                range_check_16_component,
            ],
        ): Result<[_; N_COMPONENTS], _> = components.try_into()
        else {
            panic!("Failed to convert components to array");
        };

        let mut trace = Vec::new();
        let mut interaction = Vec::new();
        trace.extend(repeat_n(*eq_log_size, eq_component.trace_columns()));
        interaction.extend(repeat_n(*eq_log_size, eq_component.interaction_columns()));
        trace.extend(repeat_n(*qm31_ops_log_size, qm31_ops_component.trace_columns()));
        interaction.extend(repeat_n(*qm31_ops_log_size, qm31_ops_component.interaction_columns()));
        trace.extend(repeat_n(*blake_gate_log_size, blake_gate_component.trace_columns()));
        interaction
            .extend(repeat_n(*blake_gate_log_size, blake_gate_component.interaction_columns()));
        trace.extend(repeat_n(*blake_round_log_size, blake_round_component.trace_columns()));
        interaction
            .extend(repeat_n(*blake_round_log_size, blake_round_component.interaction_columns()));
        trace.extend(repeat_n(
            *blake_round_sigma_log_size,
            blake_round_sigma_component.trace_columns(),
        ));
        interaction.extend(repeat_n(
            *blake_round_sigma_log_size,
            blake_round_sigma_component.interaction_columns(),
        ));
        trace.extend(repeat_n(*blake_g_log_size, blake_g_component.trace_columns()));
        interaction.extend(repeat_n(*blake_g_log_size, blake_g_component.interaction_columns()));
        trace.extend(repeat_n(*blake_output_log_size, blake_output_component.trace_columns()));
        interaction
            .extend(repeat_n(*blake_output_log_size, blake_output_component.interaction_columns()));
        trace.extend(repeat_n(*triple_xor_32_log_size, triple_xor_32_component.trace_columns()));
        interaction.extend(repeat_n(
            *triple_xor_32_log_size,
            triple_xor_32_component.interaction_columns(),
        ));
        trace.extend(repeat_n(*m_31_to_u_32_log_size, m_31_to_u_32_component.trace_columns()));
        interaction
            .extend(repeat_n(*m_31_to_u_32_log_size, m_31_to_u_32_component.interaction_columns()));
        trace.extend(repeat_n(
            *verify_bitwise_xor_8_log_size,
            verify_bitwise_xor_8_component.trace_columns(),
        ));
        interaction.extend(repeat_n(
            *verify_bitwise_xor_8_log_size,
            verify_bitwise_xor_8_component.interaction_columns(),
        ));
        trace.extend(repeat_n(
            *verify_bitwise_xor_12_log_size,
            verify_bitwise_xor_12_component.trace_columns(),
        ));
        interaction.extend(repeat_n(
            *verify_bitwise_xor_12_log_size,
            verify_bitwise_xor_12_component.interaction_columns(),
        ));
        trace.extend(repeat_n(
            *verify_bitwise_xor_4_log_size,
            verify_bitwise_xor_4_component.trace_columns(),
        ));
        interaction.extend(repeat_n(
            *verify_bitwise_xor_4_log_size,
            verify_bitwise_xor_4_component.interaction_columns(),
        ));
        trace.extend(repeat_n(
            *verify_bitwise_xor_7_log_size,
            verify_bitwise_xor_7_component.trace_columns(),
        ));
        interaction.extend(repeat_n(
            *verify_bitwise_xor_7_log_size,
            verify_bitwise_xor_7_component.interaction_columns(),
        ));
        trace.extend(repeat_n(
            *verify_bitwise_xor_9_log_size,
            verify_bitwise_xor_9_component.trace_columns(),
        ));
        interaction.extend(repeat_n(
            *verify_bitwise_xor_9_log_size,
            verify_bitwise_xor_9_component.interaction_columns(),
        ));
        trace.extend(repeat_n(*range_check_15_log_size, range_check_15_component.trace_columns()));
        interaction.extend(repeat_n(
            *range_check_15_log_size,
            range_check_15_component.interaction_columns(),
        ));
        trace.extend(repeat_n(*range_check_16_log_size, range_check_16_component.trace_columns()));
        interaction.extend(repeat_n(
            *range_check_16_log_size,
            range_check_16_component.interaction_columns(),
        ));
        [trace, interaction]
    }
}

pub struct CircuitInteractionElements {
    pub common_lookup_elements: CommonLookupElements,
}
impl CircuitInteractionElements {
    pub fn draw(channel: &mut impl Channel) -> CircuitInteractionElements {
        CircuitInteractionElements { common_lookup_elements: CommonLookupElements::draw(channel) }
    }
}

#[derive(Debug, PartialEq)]
pub struct CircuitInteractionClaim {
    pub claimed_sums: [ClaimedSum; N_COMPONENTS],
}
impl CircuitInteractionClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        let Self { claimed_sums } = self;
        channel.mix_felts(claimed_sums);
    }
}

fn blake_iv_public_logup_sum(
    n_blake_gates: usize,
    common_lookup_elements: &CommonLookupElements,
) -> QM31 {
    // Each Blake gate uses the initial state once and creates one row in blake_output.
    // Then blake_output is padded to a power of two, and each padding row uses the
    // initial state once. In total we have n_blake_gates.next_power_of_two() uses.
    let initial_state_uses = n_blake_gates.next_power_of_two();

    let state_relation_id = M31::from(BLAKE_STATE_RELATION_ID);
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
        state_relation_id,
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
    denom.inverse() * M31::from(initial_state_uses)
}

pub fn lookup_sum(
    claim: &CircuitClaim,
    interaction_claim: &CircuitInteractionClaim,
    interaction_elements: &CircuitInteractionElements,
    output_addresses: &[usize],
    n_blake_gates: usize,
) -> QM31 {
    let CircuitInteractionClaim { claimed_sums } = interaction_claim;
    let component_sum: QM31 = claimed_sums.iter().sum();

    // Compute the public logup sum from output gates.
    let mut output_sum = QM31::zero();
    let gate_relation_id = M31::from(GATE_RELATION_ID);
    for (addr, value) in zip_eq(output_addresses, &claim.output_values) {
        let values =
            [gate_relation_id, M31::from(*addr as u32), value.0.0, value.0.1, value.1.0, value.1.1];
        let denom: QM31 = interaction_elements.common_lookup_elements.combine(&values);
        output_sum += denom.inverse();
    }

    // Subtract the blake IV public logup sum (blake IV state is used but never yielded).
    let blake_iv_sum =
        blake_iv_public_logup_sum(n_blake_gates, &interaction_elements.common_lookup_elements);

    component_sum + output_sum - blake_iv_sum
}
