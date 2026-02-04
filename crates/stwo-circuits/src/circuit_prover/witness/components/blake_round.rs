// This file was created by the AIR team.

#![allow(unused_parens)]
use cairo_air::components::blake_round::{Claim, InteractionClaim, N_TRACE_COLUMNS};

use crate::circuit_prover::witness::components::prelude::*;
use crate::circuit_prover::witness::components::{blake_g, blake_message, blake_round_sigma};

pub type PackedInputType = (PackedM31, PackedM31, ([PackedUInt32; 16], PackedM31));
use blake_message::ClaimGenerator as BlakeMessage;

#[derive(Default)]
pub struct ClaimGenerator {
    pub packed_inputs: Vec<PackedInputType>,
    pub blake_message: BlakeMessage,
}
impl ClaimGenerator {
    pub fn new(blake_message: BlakeMessage) -> Self {
        Self { packed_inputs: Vec::new(), blake_message }
    }

    pub fn is_empty(&self) -> bool {
        self.packed_inputs.is_empty()
    }

    pub fn write_trace(
        mut self,
        blake_round_sigma_state: &blake_round_sigma::ClaimGenerator,
        blake_message_state: &blake_message::ClaimGenerator,
        blake_g_state: &mut blake_g::ClaimGenerator,
    ) -> (ComponentTrace<N_TRACE_COLUMNS>, Claim, InteractionClaimGenerator) {
        assert!(!self.packed_inputs.is_empty());
        let n_vec_rows = self.packed_inputs.len();
        let n_rows = n_vec_rows * N_LANES;
        let packed_size = n_vec_rows.next_power_of_two();
        let log_size = packed_size.ilog2() + LOG_N_LANES;
        self.packed_inputs.resize(packed_size, *self.packed_inputs.first().unwrap());

        let (trace, lookup_data, sub_component_inputs) = write_trace_simd(
            self.packed_inputs,
            n_rows,
            blake_round_sigma_state,
            blake_message_state,
            blake_g_state,
        );
        for inputs in sub_component_inputs.blake_round_sigma {
            blake_round_sigma_state.add_packed_inputs(&inputs, 0);
        }
        for inputs in sub_component_inputs.blake_g {
            blake_g_state.add_packed_inputs(&inputs, 0);
        }

        (trace, Claim { log_size }, InteractionClaimGenerator { n_rows, log_size, lookup_data })
    }

    pub fn add_packed_inputs(&mut self, inputs: &[PackedInputType], _relation_index: usize) {
        self.packed_inputs.extend(inputs);
    }

    pub fn deduce_output(
        &self,
        (chain, round, (state, message_id)): (
            PackedM31,
            PackedM31,
            ([PackedUInt32; 16], PackedM31),
        ),
    ) -> (PackedM31, PackedM31, ([PackedUInt32; 16], PackedM31)) {
        let (chain, round, (state, message_id)) = self.blake_round(
            chain.into_simd(),
            round.into_simd(),
            (state.map(|x| x.simd), message_id.into_simd()),
        );

        unsafe {
            (
                PackedM31::from_simd_unchecked(chain),
                PackedM31::from_simd_unchecked(round),
                (
                    state.map(|simd| PackedUInt32 { simd }),
                    PackedM31::from_simd_unchecked(message_id),
                ),
            )
        }
    }
    fn blake_round(
        &self,
        chain: u32x16,
        round: u32x16,
        (state, message_id): ([u32x16; 16], u32x16),
    ) -> (u32x16, u32x16, ([u32x16; 16], u32x16)) {
        let sigma = PackedBlakeRoundSigma::packed_sigma(round);

        let message: [_; 16] = from_fn(|i| {
            u32x16::from(from_fn(|j| {
                let row = self.blake_message.msg_per_id.get(message_id[j] as usize).unwrap();
                row[sigma[i][j] as usize] as u32
            }))
        });

        let mut state = state;
        for (row_index, &[i0, i1, i2, i3]) in G_STATE_INDICES.iter().enumerate() {
            [state[i0], state[i1], state[i2], state[i3]] = PackedBlakeG::blake_g([
                state[i0],
                state[i1],
                state[i2],
                state[i3],
                message[row_index * 2],
                message[row_index * 2 + 1],
            ]);
        }

        (chain, round + u32x16::splat(1), (state, message_id))
    }
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct SubComponentInputs {
    blake_round_sigma: [Vec<blake_round_sigma::PackedInputType>; 1],
    blake_g: [Vec<blake_g::PackedInputType>; 8],
}

#[allow(clippy::useless_conversion)]
#[allow(unused_variables)]
#[allow(clippy::double_parens)]
#[allow(non_snake_case)]
fn write_trace_simd(
    inputs: Vec<PackedInputType>,
    n_rows: usize,
    blake_round_sigma_state: &blake_round_sigma::ClaimGenerator,
    blake_message_state: &blake_message::ClaimGenerator,
    blake_g_state: &mut blake_g::ClaimGenerator,
) -> (ComponentTrace<N_TRACE_COLUMNS>, LookupData, SubComponentInputs) {
    let log_n_packed_rows = inputs.len().ilog2();
    let log_size = log_n_packed_rows + LOG_N_LANES;
    let (mut trace, mut lookup_data, mut sub_component_inputs) = unsafe {
        (
            ComponentTrace::<N_TRACE_COLUMNS>::uninitialized(log_size),
            LookupData::uninitialized(log_n_packed_rows),
            SubComponentInputs::uninitialized(log_n_packed_rows),
        )
    };

    let M31_1 = PackedM31::broadcast(M31::from(1));
    let M31_1139985212 = PackedM31::broadcast(M31::from(1139985212));
    let M31_1492981981 = PackedM31::broadcast(M31::from(1492981981));
    let M31_1805967942 = PackedM31::broadcast(M31::from(1805967942));
    let M31_40528774 = PackedM31::broadcast(M31::from(40528774));

    // TODO: Move enabler column to preprocessed trace.
    let enabler_col = Enabler::new(n_rows);

    (
        trace.par_iter_mut(),
        lookup_data.par_iter_mut(),
        sub_component_inputs.par_iter_mut(),
        inputs.into_par_iter(),
    )
        .into_par_iter()
        .enumerate()
        .for_each(
            |(row_index, (row, lookup_data, sub_component_inputs, blake_round_input))| {
                let input_limb_0_col0 = blake_round_input.0;
                *row[0] = input_limb_0_col0;
                let input_limb_1_col1 = blake_round_input.1;
                *row[1] = input_limb_1_col1;
                let input_limb_2_col2 = blake_round_input.2.0[0].low().as_m31();
                *row[2] = input_limb_2_col2;
                let input_limb_3_col3 = blake_round_input.2.0[0].high().as_m31();
                *row[3] = input_limb_3_col3;
                let input_limb_4_col4 = blake_round_input.2.0[1].low().as_m31();
                *row[4] = input_limb_4_col4;
                let input_limb_5_col5 = blake_round_input.2.0[1].high().as_m31();
                *row[5] = input_limb_5_col5;
                let input_limb_6_col6 = blake_round_input.2.0[2].low().as_m31();
                *row[6] = input_limb_6_col6;
                let input_limb_7_col7 = blake_round_input.2.0[2].high().as_m31();
                *row[7] = input_limb_7_col7;
                let input_limb_8_col8 = blake_round_input.2.0[3].low().as_m31();
                *row[8] = input_limb_8_col8;
                let input_limb_9_col9 = blake_round_input.2.0[3].high().as_m31();
                *row[9] = input_limb_9_col9;
                let input_limb_10_col10 = blake_round_input.2.0[4].low().as_m31();
                *row[10] = input_limb_10_col10;
                let input_limb_11_col11 = blake_round_input.2.0[4].high().as_m31();
                *row[11] = input_limb_11_col11;
                let input_limb_12_col12 = blake_round_input.2.0[5].low().as_m31();
                *row[12] = input_limb_12_col12;
                let input_limb_13_col13 = blake_round_input.2.0[5].high().as_m31();
                *row[13] = input_limb_13_col13;
                let input_limb_14_col14 = blake_round_input.2.0[6].low().as_m31();
                *row[14] = input_limb_14_col14;
                let input_limb_15_col15 = blake_round_input.2.0[6].high().as_m31();
                *row[15] = input_limb_15_col15;
                let input_limb_16_col16 = blake_round_input.2.0[7].low().as_m31();
                *row[16] = input_limb_16_col16;
                let input_limb_17_col17 = blake_round_input.2.0[7].high().as_m31();
                *row[17] = input_limb_17_col17;
                let input_limb_18_col18 = blake_round_input.2.0[8].low().as_m31();
                *row[18] = input_limb_18_col18;
                let input_limb_19_col19 = blake_round_input.2.0[8].high().as_m31();
                *row[19] = input_limb_19_col19;
                let input_limb_20_col20 = blake_round_input.2.0[9].low().as_m31();
                *row[20] = input_limb_20_col20;
                let input_limb_21_col21 = blake_round_input.2.0[9].high().as_m31();
                *row[21] = input_limb_21_col21;
                let input_limb_22_col22 = blake_round_input.2.0[10].low().as_m31();
                *row[22] = input_limb_22_col22;
                let input_limb_23_col23 = blake_round_input.2.0[10].high().as_m31();
                *row[23] = input_limb_23_col23;
                let input_limb_24_col24 = blake_round_input.2.0[11].low().as_m31();
                *row[24] = input_limb_24_col24;
                let input_limb_25_col25 = blake_round_input.2.0[11].high().as_m31();
                *row[25] = input_limb_25_col25;
                let input_limb_26_col26 = blake_round_input.2.0[12].low().as_m31();
                *row[26] = input_limb_26_col26;
                let input_limb_27_col27 = blake_round_input.2.0[12].high().as_m31();
                *row[27] = input_limb_27_col27;
                let input_limb_28_col28 = blake_round_input.2.0[13].low().as_m31();
                *row[28] = input_limb_28_col28;
                let input_limb_29_col29 = blake_round_input.2.0[13].high().as_m31();
                *row[29] = input_limb_29_col29;
                let input_limb_30_col30 = blake_round_input.2.0[14].low().as_m31();
                *row[30] = input_limb_30_col30;
                let input_limb_31_col31 = blake_round_input.2.0[14].high().as_m31();
                *row[31] = input_limb_31_col31;
                let input_limb_32_col32 = blake_round_input.2.0[15].low().as_m31();
                *row[32] = input_limb_32_col32;
                let input_limb_33_col33 = blake_round_input.2.0[15].high().as_m31();
                *row[33] = input_limb_33_col33;
                let input_limb_34_col34 = blake_round_input.2.1;
                *row[34] = input_limb_34_col34;
                *sub_component_inputs.blake_round_sigma[0] = [input_limb_1_col1];
                let blake_round_sigma_output_tmp_39a7a_0 =
                    PackedBlakeRoundSigma::deduce_output(input_limb_1_col1);
                let blake_round_sigma_output_limb_0_col35 = blake_round_sigma_output_tmp_39a7a_0[0];
                *row[35] = blake_round_sigma_output_limb_0_col35;
                let blake_round_sigma_output_limb_1_col36 = blake_round_sigma_output_tmp_39a7a_0[1];
                *row[36] = blake_round_sigma_output_limb_1_col36;
                let blake_round_sigma_output_limb_2_col37 = blake_round_sigma_output_tmp_39a7a_0[2];
                *row[37] = blake_round_sigma_output_limb_2_col37;
                let blake_round_sigma_output_limb_3_col38 = blake_round_sigma_output_tmp_39a7a_0[3];
                *row[38] = blake_round_sigma_output_limb_3_col38;
                let blake_round_sigma_output_limb_4_col39 = blake_round_sigma_output_tmp_39a7a_0[4];
                *row[39] = blake_round_sigma_output_limb_4_col39;
                let blake_round_sigma_output_limb_5_col40 = blake_round_sigma_output_tmp_39a7a_0[5];
                *row[40] = blake_round_sigma_output_limb_5_col40;
                let blake_round_sigma_output_limb_6_col41 = blake_round_sigma_output_tmp_39a7a_0[6];
                *row[41] = blake_round_sigma_output_limb_6_col41;
                let blake_round_sigma_output_limb_7_col42 = blake_round_sigma_output_tmp_39a7a_0[7];
                *row[42] = blake_round_sigma_output_limb_7_col42;
                let blake_round_sigma_output_limb_8_col43 = blake_round_sigma_output_tmp_39a7a_0[8];
                *row[43] = blake_round_sigma_output_limb_8_col43;
                let blake_round_sigma_output_limb_9_col44 = blake_round_sigma_output_tmp_39a7a_0[9];
                *row[44] = blake_round_sigma_output_limb_9_col44;
                let blake_round_sigma_output_limb_10_col45 =
                    blake_round_sigma_output_tmp_39a7a_0[10];
                *row[45] = blake_round_sigma_output_limb_10_col45;
                let blake_round_sigma_output_limb_11_col46 =
                    blake_round_sigma_output_tmp_39a7a_0[11];
                *row[46] = blake_round_sigma_output_limb_11_col46;
                let blake_round_sigma_output_limb_12_col47 =
                    blake_round_sigma_output_tmp_39a7a_0[12];
                *row[47] = blake_round_sigma_output_limb_12_col47;
                let blake_round_sigma_output_limb_13_col48 =
                    blake_round_sigma_output_tmp_39a7a_0[13];
                *row[48] = blake_round_sigma_output_limb_13_col48;
                let blake_round_sigma_output_limb_14_col49 =
                    blake_round_sigma_output_tmp_39a7a_0[14];
                *row[49] = blake_round_sigma_output_limb_14_col49;
                let blake_round_sigma_output_limb_15_col50 =
                    blake_round_sigma_output_tmp_39a7a_0[15];
                *row[50] = blake_round_sigma_output_limb_15_col50;
                *lookup_data.blake_round_sigma_0 = [
                    M31_1805967942,
                    input_limb_1_col1,
                    blake_round_sigma_output_limb_0_col35,
                    blake_round_sigma_output_limb_1_col36,
                    blake_round_sigma_output_limb_2_col37,
                    blake_round_sigma_output_limb_3_col38,
                    blake_round_sigma_output_limb_4_col39,
                    blake_round_sigma_output_limb_5_col40,
                    blake_round_sigma_output_limb_6_col41,
                    blake_round_sigma_output_limb_7_col42,
                    blake_round_sigma_output_limb_8_col43,
                    blake_round_sigma_output_limb_9_col44,
                    blake_round_sigma_output_limb_10_col45,
                    blake_round_sigma_output_limb_11_col46,
                    blake_round_sigma_output_limb_12_col47,
                    blake_round_sigma_output_limb_13_col48,
                    blake_round_sigma_output_limb_14_col49,
                    blake_round_sigma_output_limb_15_col50,
                ];
                let blake_message_output_tmp_39a7a_1 = blake_message_state
                    .deduce_output([input_limb_34_col34, blake_round_sigma_output_limb_0_col35]);
                let blake_message_output_message_limb_limb_0_col51 =
                    blake_message_output_tmp_39a7a_1.low().as_m31();
                *row[51] = blake_message_output_message_limb_limb_0_col51;
                let blake_message_output_message_limb_limb_1_col52 =
                    blake_message_output_tmp_39a7a_1.high().as_m31();
                *row[52] = blake_message_output_message_limb_limb_1_col52;
                *lookup_data.blake_message_0 = [
                    M31_1492981981,
                    input_limb_34_col34,
                    blake_round_sigma_output_limb_0_col35,
                    blake_message_output_message_limb_limb_0_col51,
                    blake_message_output_message_limb_limb_1_col52,
                ];
                let blake_message_output_tmp_39a7a_2 = blake_message_state
                    .deduce_output([input_limb_34_col34, blake_round_sigma_output_limb_1_col36]);
                let blake_message_output_message_limb_limb_0_col53 =
                    blake_message_output_tmp_39a7a_2.low().as_m31();
                *row[53] = blake_message_output_message_limb_limb_0_col53;
                let blake_message_output_message_limb_limb_1_col54 =
                    blake_message_output_tmp_39a7a_2.high().as_m31();
                *row[54] = blake_message_output_message_limb_limb_1_col54;
                *lookup_data.blake_message_1 = [
                    M31_1492981981,
                    input_limb_34_col34,
                    blake_round_sigma_output_limb_1_col36,
                    blake_message_output_message_limb_limb_0_col53,
                    blake_message_output_message_limb_limb_1_col54,
                ];
                let blake_message_output_tmp_39a7a_3 = blake_message_state
                    .deduce_output([input_limb_34_col34, blake_round_sigma_output_limb_2_col37]);
                let blake_message_output_message_limb_limb_0_col55 =
                    blake_message_output_tmp_39a7a_3.low().as_m31();
                *row[55] = blake_message_output_message_limb_limb_0_col55;
                let blake_message_output_message_limb_limb_1_col56 =
                    blake_message_output_tmp_39a7a_3.high().as_m31();
                *row[56] = blake_message_output_message_limb_limb_1_col56;
                *lookup_data.blake_message_2 = [
                    M31_1492981981,
                    input_limb_34_col34,
                    blake_round_sigma_output_limb_2_col37,
                    blake_message_output_message_limb_limb_0_col55,
                    blake_message_output_message_limb_limb_1_col56,
                ];
                let blake_message_output_tmp_39a7a_4 = blake_message_state
                    .deduce_output([input_limb_34_col34, blake_round_sigma_output_limb_3_col38]);
                let blake_message_output_message_limb_limb_0_col57 =
                    blake_message_output_tmp_39a7a_4.low().as_m31();
                *row[57] = blake_message_output_message_limb_limb_0_col57;
                let blake_message_output_message_limb_limb_1_col58 =
                    blake_message_output_tmp_39a7a_4.high().as_m31();
                *row[58] = blake_message_output_message_limb_limb_1_col58;
                *lookup_data.blake_message_3 = [
                    M31_1492981981,
                    input_limb_34_col34,
                    blake_round_sigma_output_limb_3_col38,
                    blake_message_output_message_limb_limb_0_col57,
                    blake_message_output_message_limb_limb_1_col58,
                ];
                let blake_message_output_tmp_39a7a_5 = blake_message_state
                    .deduce_output([input_limb_34_col34, blake_round_sigma_output_limb_4_col39]);
                let blake_message_output_message_limb_limb_0_col59 =
                    blake_message_output_tmp_39a7a_5.low().as_m31();
                *row[59] = blake_message_output_message_limb_limb_0_col59;
                let blake_message_output_message_limb_limb_1_col60 =
                    blake_message_output_tmp_39a7a_5.high().as_m31();
                *row[60] = blake_message_output_message_limb_limb_1_col60;
                *lookup_data.blake_message_4 = [
                    M31_1492981981,
                    input_limb_34_col34,
                    blake_round_sigma_output_limb_4_col39,
                    blake_message_output_message_limb_limb_0_col59,
                    blake_message_output_message_limb_limb_1_col60,
                ];
                let blake_message_output_tmp_39a7a_6 = blake_message_state
                    .deduce_output([input_limb_34_col34, blake_round_sigma_output_limb_5_col40]);
                let blake_message_output_message_limb_limb_0_col61 =
                    blake_message_output_tmp_39a7a_6.low().as_m31();
                *row[61] = blake_message_output_message_limb_limb_0_col61;
                let blake_message_output_message_limb_limb_1_col62 =
                    blake_message_output_tmp_39a7a_6.high().as_m31();
                *row[62] = blake_message_output_message_limb_limb_1_col62;
                *lookup_data.blake_message_5 = [
                    M31_1492981981,
                    input_limb_34_col34,
                    blake_round_sigma_output_limb_5_col40,
                    blake_message_output_message_limb_limb_0_col61,
                    blake_message_output_message_limb_limb_1_col62,
                ];
                let blake_message_output_tmp_39a7a_7 = blake_message_state
                    .deduce_output([input_limb_34_col34, blake_round_sigma_output_limb_6_col41]);
                let blake_message_output_message_limb_limb_0_col63 =
                    blake_message_output_tmp_39a7a_7.low().as_m31();
                *row[63] = blake_message_output_message_limb_limb_0_col63;
                let blake_message_output_message_limb_limb_1_col64 =
                    blake_message_output_tmp_39a7a_7.high().as_m31();
                *row[64] = blake_message_output_message_limb_limb_1_col64;
                *lookup_data.blake_message_6 = [
                    M31_1492981981,
                    input_limb_34_col34,
                    blake_round_sigma_output_limb_6_col41,
                    blake_message_output_message_limb_limb_0_col63,
                    blake_message_output_message_limb_limb_1_col64,
                ];
                let blake_message_output_tmp_39a7a_8 = blake_message_state
                    .deduce_output([input_limb_34_col34, blake_round_sigma_output_limb_7_col42]);
                let blake_message_output_message_limb_limb_0_col65 =
                    blake_message_output_tmp_39a7a_8.low().as_m31();
                *row[65] = blake_message_output_message_limb_limb_0_col65;
                let blake_message_output_message_limb_limb_1_col66 =
                    blake_message_output_tmp_39a7a_8.high().as_m31();
                *row[66] = blake_message_output_message_limb_limb_1_col66;
                *lookup_data.blake_message_7 = [
                    M31_1492981981,
                    input_limb_34_col34,
                    blake_round_sigma_output_limb_7_col42,
                    blake_message_output_message_limb_limb_0_col65,
                    blake_message_output_message_limb_limb_1_col66,
                ];
                let blake_message_output_tmp_39a7a_9 = blake_message_state
                    .deduce_output([input_limb_34_col34, blake_round_sigma_output_limb_8_col43]);
                let blake_message_output_message_limb_limb_0_col67 =
                    blake_message_output_tmp_39a7a_9.low().as_m31();
                *row[67] = blake_message_output_message_limb_limb_0_col67;
                let blake_message_output_message_limb_limb_1_col68 =
                    blake_message_output_tmp_39a7a_9.high().as_m31();
                *row[68] = blake_message_output_message_limb_limb_1_col68;
                *lookup_data.blake_message_8 = [
                    M31_1492981981,
                    input_limb_34_col34,
                    blake_round_sigma_output_limb_8_col43,
                    blake_message_output_message_limb_limb_0_col67,
                    blake_message_output_message_limb_limb_1_col68,
                ];
                let blake_message_output_tmp_39a7a_10 = blake_message_state
                    .deduce_output([input_limb_34_col34, blake_round_sigma_output_limb_9_col44]);
                let blake_message_output_message_limb_limb_0_col69 =
                    blake_message_output_tmp_39a7a_10.low().as_m31();
                *row[69] = blake_message_output_message_limb_limb_0_col69;
                let blake_message_output_message_limb_limb_1_col70 =
                    blake_message_output_tmp_39a7a_10.high().as_m31();
                *row[70] = blake_message_output_message_limb_limb_1_col70;
                *lookup_data.blake_message_9 = [
                    M31_1492981981,
                    input_limb_34_col34,
                    blake_round_sigma_output_limb_9_col44,
                    blake_message_output_message_limb_limb_0_col69,
                    blake_message_output_message_limb_limb_1_col70,
                ];
                let blake_message_output_tmp_39a7a_11 = blake_message_state
                    .deduce_output([input_limb_34_col34, blake_round_sigma_output_limb_10_col45]);
                let blake_message_output_message_limb_limb_0_col71 =
                    blake_message_output_tmp_39a7a_11.low().as_m31();
                *row[71] = blake_message_output_message_limb_limb_0_col71;
                let blake_message_output_message_limb_limb_1_col72 =
                    blake_message_output_tmp_39a7a_11.high().as_m31();
                *row[72] = blake_message_output_message_limb_limb_1_col72;
                *lookup_data.blake_message_10 = [
                    M31_1492981981,
                    input_limb_34_col34,
                    blake_round_sigma_output_limb_10_col45,
                    blake_message_output_message_limb_limb_0_col71,
                    blake_message_output_message_limb_limb_1_col72,
                ];
                let blake_message_output_tmp_39a7a_12 = blake_message_state
                    .deduce_output([input_limb_34_col34, blake_round_sigma_output_limb_11_col46]);
                let blake_message_output_message_limb_limb_0_col73 =
                    blake_message_output_tmp_39a7a_12.low().as_m31();
                *row[73] = blake_message_output_message_limb_limb_0_col73;
                let blake_message_output_message_limb_limb_1_col74 =
                    blake_message_output_tmp_39a7a_12.high().as_m31();
                *row[74] = blake_message_output_message_limb_limb_1_col74;
                *lookup_data.blake_message_11 = [
                    M31_1492981981,
                    input_limb_34_col34,
                    blake_round_sigma_output_limb_11_col46,
                    blake_message_output_message_limb_limb_0_col73,
                    blake_message_output_message_limb_limb_1_col74,
                ];
                let blake_message_output_tmp_39a7a_13 = blake_message_state
                    .deduce_output([input_limb_34_col34, blake_round_sigma_output_limb_12_col47]);
                let blake_message_output_message_limb_limb_0_col75 =
                    blake_message_output_tmp_39a7a_13.low().as_m31();
                *row[75] = blake_message_output_message_limb_limb_0_col75;
                let blake_message_output_message_limb_limb_1_col76 =
                    blake_message_output_tmp_39a7a_13.high().as_m31();
                *row[76] = blake_message_output_message_limb_limb_1_col76;
                *lookup_data.blake_message_12 = [
                    M31_1492981981,
                    input_limb_34_col34,
                    blake_round_sigma_output_limb_12_col47,
                    blake_message_output_message_limb_limb_0_col75,
                    blake_message_output_message_limb_limb_1_col76,
                ];
                let blake_message_output_tmp_39a7a_14 = blake_message_state
                    .deduce_output([input_limb_34_col34, blake_round_sigma_output_limb_13_col48]);
                let blake_message_output_message_limb_limb_0_col77 =
                    blake_message_output_tmp_39a7a_14.low().as_m31();
                *row[77] = blake_message_output_message_limb_limb_0_col77;
                let blake_message_output_message_limb_limb_1_col78 =
                    blake_message_output_tmp_39a7a_14.high().as_m31();
                *row[78] = blake_message_output_message_limb_limb_1_col78;
                *lookup_data.blake_message_13 = [
                    M31_1492981981,
                    input_limb_34_col34,
                    blake_round_sigma_output_limb_13_col48,
                    blake_message_output_message_limb_limb_0_col77,
                    blake_message_output_message_limb_limb_1_col78,
                ];
                let blake_message_output_tmp_39a7a_15 = blake_message_state
                    .deduce_output([input_limb_34_col34, blake_round_sigma_output_limb_14_col49]);
                let blake_message_output_message_limb_limb_0_col79 =
                    blake_message_output_tmp_39a7a_15.low().as_m31();
                *row[79] = blake_message_output_message_limb_limb_0_col79;
                let blake_message_output_message_limb_limb_1_col80 =
                    blake_message_output_tmp_39a7a_15.high().as_m31();
                *row[80] = blake_message_output_message_limb_limb_1_col80;
                *lookup_data.blake_message_14 = [
                    M31_1492981981,
                    input_limb_34_col34,
                    blake_round_sigma_output_limb_14_col49,
                    blake_message_output_message_limb_limb_0_col79,
                    blake_message_output_message_limb_limb_1_col80,
                ];
                let blake_message_output_tmp_39a7a_16 = blake_message_state
                    .deduce_output([input_limb_34_col34, blake_round_sigma_output_limb_15_col50]);
                let blake_message_output_message_limb_limb_0_col81 =
                    blake_message_output_tmp_39a7a_16.low().as_m31();
                *row[81] = blake_message_output_message_limb_limb_0_col81;
                let blake_message_output_message_limb_limb_1_col82 =
                    blake_message_output_tmp_39a7a_16.high().as_m31();
                *row[82] = blake_message_output_message_limb_limb_1_col82;
                *lookup_data.blake_message_15 = [
                    M31_1492981981,
                    input_limb_34_col34,
                    blake_round_sigma_output_limb_15_col50,
                    blake_message_output_message_limb_limb_0_col81,
                    blake_message_output_message_limb_limb_1_col82,
                ];
                *sub_component_inputs.blake_g[0] = [
                    blake_round_input.2.0[0],
                    blake_round_input.2.0[4],
                    blake_round_input.2.0[8],
                    blake_round_input.2.0[12],
                    blake_message_output_tmp_39a7a_1,
                    blake_message_output_tmp_39a7a_2,
                ];
                let blake_g_output_tmp_39a7a_17 = PackedBlakeG::deduce_output([
                    blake_round_input.2.0[0],
                    blake_round_input.2.0[4],
                    blake_round_input.2.0[8],
                    blake_round_input.2.0[12],
                    blake_message_output_tmp_39a7a_1,
                    blake_message_output_tmp_39a7a_2,
                ]);
                let blake_g_output_limb_0_col83 = blake_g_output_tmp_39a7a_17[0].low().as_m31();
                *row[83] = blake_g_output_limb_0_col83;
                let blake_g_output_limb_1_col84 = blake_g_output_tmp_39a7a_17[0].high().as_m31();
                *row[84] = blake_g_output_limb_1_col84;
                let blake_g_output_limb_2_col85 = blake_g_output_tmp_39a7a_17[1].low().as_m31();
                *row[85] = blake_g_output_limb_2_col85;
                let blake_g_output_limb_3_col86 = blake_g_output_tmp_39a7a_17[1].high().as_m31();
                *row[86] = blake_g_output_limb_3_col86;
                let blake_g_output_limb_4_col87 = blake_g_output_tmp_39a7a_17[2].low().as_m31();
                *row[87] = blake_g_output_limb_4_col87;
                let blake_g_output_limb_5_col88 = blake_g_output_tmp_39a7a_17[2].high().as_m31();
                *row[88] = blake_g_output_limb_5_col88;
                let blake_g_output_limb_6_col89 = blake_g_output_tmp_39a7a_17[3].low().as_m31();
                *row[89] = blake_g_output_limb_6_col89;
                let blake_g_output_limb_7_col90 = blake_g_output_tmp_39a7a_17[3].high().as_m31();
                *row[90] = blake_g_output_limb_7_col90;
                *lookup_data.blake_g_0 = [
                    M31_1139985212,
                    input_limb_2_col2,
                    input_limb_3_col3,
                    input_limb_10_col10,
                    input_limb_11_col11,
                    input_limb_18_col18,
                    input_limb_19_col19,
                    input_limb_26_col26,
                    input_limb_27_col27,
                    blake_message_output_message_limb_limb_0_col51,
                    blake_message_output_message_limb_limb_1_col52,
                    blake_message_output_message_limb_limb_0_col53,
                    blake_message_output_message_limb_limb_1_col54,
                    blake_g_output_limb_0_col83,
                    blake_g_output_limb_1_col84,
                    blake_g_output_limb_2_col85,
                    blake_g_output_limb_3_col86,
                    blake_g_output_limb_4_col87,
                    blake_g_output_limb_5_col88,
                    blake_g_output_limb_6_col89,
                    blake_g_output_limb_7_col90,
                ];
                *sub_component_inputs.blake_g[1] = [
                    blake_round_input.2.0[1],
                    blake_round_input.2.0[5],
                    blake_round_input.2.0[9],
                    blake_round_input.2.0[13],
                    blake_message_output_tmp_39a7a_3,
                    blake_message_output_tmp_39a7a_4,
                ];
                let blake_g_output_tmp_39a7a_18 = PackedBlakeG::deduce_output([
                    blake_round_input.2.0[1],
                    blake_round_input.2.0[5],
                    blake_round_input.2.0[9],
                    blake_round_input.2.0[13],
                    blake_message_output_tmp_39a7a_3,
                    blake_message_output_tmp_39a7a_4,
                ]);
                let blake_g_output_limb_0_col91 = blake_g_output_tmp_39a7a_18[0].low().as_m31();
                *row[91] = blake_g_output_limb_0_col91;
                let blake_g_output_limb_1_col92 = blake_g_output_tmp_39a7a_18[0].high().as_m31();
                *row[92] = blake_g_output_limb_1_col92;
                let blake_g_output_limb_2_col93 = blake_g_output_tmp_39a7a_18[1].low().as_m31();
                *row[93] = blake_g_output_limb_2_col93;
                let blake_g_output_limb_3_col94 = blake_g_output_tmp_39a7a_18[1].high().as_m31();
                *row[94] = blake_g_output_limb_3_col94;
                let blake_g_output_limb_4_col95 = blake_g_output_tmp_39a7a_18[2].low().as_m31();
                *row[95] = blake_g_output_limb_4_col95;
                let blake_g_output_limb_5_col96 = blake_g_output_tmp_39a7a_18[2].high().as_m31();
                *row[96] = blake_g_output_limb_5_col96;
                let blake_g_output_limb_6_col97 = blake_g_output_tmp_39a7a_18[3].low().as_m31();
                *row[97] = blake_g_output_limb_6_col97;
                let blake_g_output_limb_7_col98 = blake_g_output_tmp_39a7a_18[3].high().as_m31();
                *row[98] = blake_g_output_limb_7_col98;
                *lookup_data.blake_g_1 = [
                    M31_1139985212,
                    input_limb_4_col4,
                    input_limb_5_col5,
                    input_limb_12_col12,
                    input_limb_13_col13,
                    input_limb_20_col20,
                    input_limb_21_col21,
                    input_limb_28_col28,
                    input_limb_29_col29,
                    blake_message_output_message_limb_limb_0_col55,
                    blake_message_output_message_limb_limb_1_col56,
                    blake_message_output_message_limb_limb_0_col57,
                    blake_message_output_message_limb_limb_1_col58,
                    blake_g_output_limb_0_col91,
                    blake_g_output_limb_1_col92,
                    blake_g_output_limb_2_col93,
                    blake_g_output_limb_3_col94,
                    blake_g_output_limb_4_col95,
                    blake_g_output_limb_5_col96,
                    blake_g_output_limb_6_col97,
                    blake_g_output_limb_7_col98,
                ];
                *sub_component_inputs.blake_g[2] = [
                    blake_round_input.2.0[2],
                    blake_round_input.2.0[6],
                    blake_round_input.2.0[10],
                    blake_round_input.2.0[14],
                    blake_message_output_tmp_39a7a_5,
                    blake_message_output_tmp_39a7a_6,
                ];
                let blake_g_output_tmp_39a7a_19 = PackedBlakeG::deduce_output([
                    blake_round_input.2.0[2],
                    blake_round_input.2.0[6],
                    blake_round_input.2.0[10],
                    blake_round_input.2.0[14],
                    blake_message_output_tmp_39a7a_5,
                    blake_message_output_tmp_39a7a_6,
                ]);
                let blake_g_output_limb_0_col99 = blake_g_output_tmp_39a7a_19[0].low().as_m31();
                *row[99] = blake_g_output_limb_0_col99;
                let blake_g_output_limb_1_col100 = blake_g_output_tmp_39a7a_19[0].high().as_m31();
                *row[100] = blake_g_output_limb_1_col100;
                let blake_g_output_limb_2_col101 = blake_g_output_tmp_39a7a_19[1].low().as_m31();
                *row[101] = blake_g_output_limb_2_col101;
                let blake_g_output_limb_3_col102 = blake_g_output_tmp_39a7a_19[1].high().as_m31();
                *row[102] = blake_g_output_limb_3_col102;
                let blake_g_output_limb_4_col103 = blake_g_output_tmp_39a7a_19[2].low().as_m31();
                *row[103] = blake_g_output_limb_4_col103;
                let blake_g_output_limb_5_col104 = blake_g_output_tmp_39a7a_19[2].high().as_m31();
                *row[104] = blake_g_output_limb_5_col104;
                let blake_g_output_limb_6_col105 = blake_g_output_tmp_39a7a_19[3].low().as_m31();
                *row[105] = blake_g_output_limb_6_col105;
                let blake_g_output_limb_7_col106 = blake_g_output_tmp_39a7a_19[3].high().as_m31();
                *row[106] = blake_g_output_limb_7_col106;
                *lookup_data.blake_g_2 = [
                    M31_1139985212,
                    input_limb_6_col6,
                    input_limb_7_col7,
                    input_limb_14_col14,
                    input_limb_15_col15,
                    input_limb_22_col22,
                    input_limb_23_col23,
                    input_limb_30_col30,
                    input_limb_31_col31,
                    blake_message_output_message_limb_limb_0_col59,
                    blake_message_output_message_limb_limb_1_col60,
                    blake_message_output_message_limb_limb_0_col61,
                    blake_message_output_message_limb_limb_1_col62,
                    blake_g_output_limb_0_col99,
                    blake_g_output_limb_1_col100,
                    blake_g_output_limb_2_col101,
                    blake_g_output_limb_3_col102,
                    blake_g_output_limb_4_col103,
                    blake_g_output_limb_5_col104,
                    blake_g_output_limb_6_col105,
                    blake_g_output_limb_7_col106,
                ];
                *sub_component_inputs.blake_g[3] = [
                    blake_round_input.2.0[3],
                    blake_round_input.2.0[7],
                    blake_round_input.2.0[11],
                    blake_round_input.2.0[15],
                    blake_message_output_tmp_39a7a_7,
                    blake_message_output_tmp_39a7a_8,
                ];
                let blake_g_output_tmp_39a7a_20 = PackedBlakeG::deduce_output([
                    blake_round_input.2.0[3],
                    blake_round_input.2.0[7],
                    blake_round_input.2.0[11],
                    blake_round_input.2.0[15],
                    blake_message_output_tmp_39a7a_7,
                    blake_message_output_tmp_39a7a_8,
                ]);
                let blake_g_output_limb_0_col107 = blake_g_output_tmp_39a7a_20[0].low().as_m31();
                *row[107] = blake_g_output_limb_0_col107;
                let blake_g_output_limb_1_col108 = blake_g_output_tmp_39a7a_20[0].high().as_m31();
                *row[108] = blake_g_output_limb_1_col108;
                let blake_g_output_limb_2_col109 = blake_g_output_tmp_39a7a_20[1].low().as_m31();
                *row[109] = blake_g_output_limb_2_col109;
                let blake_g_output_limb_3_col110 = blake_g_output_tmp_39a7a_20[1].high().as_m31();
                *row[110] = blake_g_output_limb_3_col110;
                let blake_g_output_limb_4_col111 = blake_g_output_tmp_39a7a_20[2].low().as_m31();
                *row[111] = blake_g_output_limb_4_col111;
                let blake_g_output_limb_5_col112 = blake_g_output_tmp_39a7a_20[2].high().as_m31();
                *row[112] = blake_g_output_limb_5_col112;
                let blake_g_output_limb_6_col113 = blake_g_output_tmp_39a7a_20[3].low().as_m31();
                *row[113] = blake_g_output_limb_6_col113;
                let blake_g_output_limb_7_col114 = blake_g_output_tmp_39a7a_20[3].high().as_m31();
                *row[114] = blake_g_output_limb_7_col114;
                *lookup_data.blake_g_3 = [
                    M31_1139985212,
                    input_limb_8_col8,
                    input_limb_9_col9,
                    input_limb_16_col16,
                    input_limb_17_col17,
                    input_limb_24_col24,
                    input_limb_25_col25,
                    input_limb_32_col32,
                    input_limb_33_col33,
                    blake_message_output_message_limb_limb_0_col63,
                    blake_message_output_message_limb_limb_1_col64,
                    blake_message_output_message_limb_limb_0_col65,
                    blake_message_output_message_limb_limb_1_col66,
                    blake_g_output_limb_0_col107,
                    blake_g_output_limb_1_col108,
                    blake_g_output_limb_2_col109,
                    blake_g_output_limb_3_col110,
                    blake_g_output_limb_4_col111,
                    blake_g_output_limb_5_col112,
                    blake_g_output_limb_6_col113,
                    blake_g_output_limb_7_col114,
                ];
                *sub_component_inputs.blake_g[4] = [
                    blake_g_output_tmp_39a7a_17[0],
                    blake_g_output_tmp_39a7a_18[1],
                    blake_g_output_tmp_39a7a_19[2],
                    blake_g_output_tmp_39a7a_20[3],
                    blake_message_output_tmp_39a7a_9,
                    blake_message_output_tmp_39a7a_10,
                ];
                let blake_g_output_tmp_39a7a_21 = PackedBlakeG::deduce_output([
                    blake_g_output_tmp_39a7a_17[0],
                    blake_g_output_tmp_39a7a_18[1],
                    blake_g_output_tmp_39a7a_19[2],
                    blake_g_output_tmp_39a7a_20[3],
                    blake_message_output_tmp_39a7a_9,
                    blake_message_output_tmp_39a7a_10,
                ]);
                let blake_g_output_limb_0_col115 = blake_g_output_tmp_39a7a_21[0].low().as_m31();
                *row[115] = blake_g_output_limb_0_col115;
                let blake_g_output_limb_1_col116 = blake_g_output_tmp_39a7a_21[0].high().as_m31();
                *row[116] = blake_g_output_limb_1_col116;
                let blake_g_output_limb_2_col117 = blake_g_output_tmp_39a7a_21[1].low().as_m31();
                *row[117] = blake_g_output_limb_2_col117;
                let blake_g_output_limb_3_col118 = blake_g_output_tmp_39a7a_21[1].high().as_m31();
                *row[118] = blake_g_output_limb_3_col118;
                let blake_g_output_limb_4_col119 = blake_g_output_tmp_39a7a_21[2].low().as_m31();
                *row[119] = blake_g_output_limb_4_col119;
                let blake_g_output_limb_5_col120 = blake_g_output_tmp_39a7a_21[2].high().as_m31();
                *row[120] = blake_g_output_limb_5_col120;
                let blake_g_output_limb_6_col121 = blake_g_output_tmp_39a7a_21[3].low().as_m31();
                *row[121] = blake_g_output_limb_6_col121;
                let blake_g_output_limb_7_col122 = blake_g_output_tmp_39a7a_21[3].high().as_m31();
                *row[122] = blake_g_output_limb_7_col122;
                *lookup_data.blake_g_4 = [
                    M31_1139985212,
                    blake_g_output_limb_0_col83,
                    blake_g_output_limb_1_col84,
                    blake_g_output_limb_2_col93,
                    blake_g_output_limb_3_col94,
                    blake_g_output_limb_4_col103,
                    blake_g_output_limb_5_col104,
                    blake_g_output_limb_6_col113,
                    blake_g_output_limb_7_col114,
                    blake_message_output_message_limb_limb_0_col67,
                    blake_message_output_message_limb_limb_1_col68,
                    blake_message_output_message_limb_limb_0_col69,
                    blake_message_output_message_limb_limb_1_col70,
                    blake_g_output_limb_0_col115,
                    blake_g_output_limb_1_col116,
                    blake_g_output_limb_2_col117,
                    blake_g_output_limb_3_col118,
                    blake_g_output_limb_4_col119,
                    blake_g_output_limb_5_col120,
                    blake_g_output_limb_6_col121,
                    blake_g_output_limb_7_col122,
                ];
                *sub_component_inputs.blake_g[5] = [
                    blake_g_output_tmp_39a7a_18[0],
                    blake_g_output_tmp_39a7a_19[1],
                    blake_g_output_tmp_39a7a_20[2],
                    blake_g_output_tmp_39a7a_17[3],
                    blake_message_output_tmp_39a7a_11,
                    blake_message_output_tmp_39a7a_12,
                ];
                let blake_g_output_tmp_39a7a_22 = PackedBlakeG::deduce_output([
                    blake_g_output_tmp_39a7a_18[0],
                    blake_g_output_tmp_39a7a_19[1],
                    blake_g_output_tmp_39a7a_20[2],
                    blake_g_output_tmp_39a7a_17[3],
                    blake_message_output_tmp_39a7a_11,
                    blake_message_output_tmp_39a7a_12,
                ]);
                let blake_g_output_limb_0_col123 = blake_g_output_tmp_39a7a_22[0].low().as_m31();
                *row[123] = blake_g_output_limb_0_col123;
                let blake_g_output_limb_1_col124 = blake_g_output_tmp_39a7a_22[0].high().as_m31();
                *row[124] = blake_g_output_limb_1_col124;
                let blake_g_output_limb_2_col125 = blake_g_output_tmp_39a7a_22[1].low().as_m31();
                *row[125] = blake_g_output_limb_2_col125;
                let blake_g_output_limb_3_col126 = blake_g_output_tmp_39a7a_22[1].high().as_m31();
                *row[126] = blake_g_output_limb_3_col126;
                let blake_g_output_limb_4_col127 = blake_g_output_tmp_39a7a_22[2].low().as_m31();
                *row[127] = blake_g_output_limb_4_col127;
                let blake_g_output_limb_5_col128 = blake_g_output_tmp_39a7a_22[2].high().as_m31();
                *row[128] = blake_g_output_limb_5_col128;
                let blake_g_output_limb_6_col129 = blake_g_output_tmp_39a7a_22[3].low().as_m31();
                *row[129] = blake_g_output_limb_6_col129;
                let blake_g_output_limb_7_col130 = blake_g_output_tmp_39a7a_22[3].high().as_m31();
                *row[130] = blake_g_output_limb_7_col130;
                *lookup_data.blake_g_5 = [
                    M31_1139985212,
                    blake_g_output_limb_0_col91,
                    blake_g_output_limb_1_col92,
                    blake_g_output_limb_2_col101,
                    blake_g_output_limb_3_col102,
                    blake_g_output_limb_4_col111,
                    blake_g_output_limb_5_col112,
                    blake_g_output_limb_6_col89,
                    blake_g_output_limb_7_col90,
                    blake_message_output_message_limb_limb_0_col71,
                    blake_message_output_message_limb_limb_1_col72,
                    blake_message_output_message_limb_limb_0_col73,
                    blake_message_output_message_limb_limb_1_col74,
                    blake_g_output_limb_0_col123,
                    blake_g_output_limb_1_col124,
                    blake_g_output_limb_2_col125,
                    blake_g_output_limb_3_col126,
                    blake_g_output_limb_4_col127,
                    blake_g_output_limb_5_col128,
                    blake_g_output_limb_6_col129,
                    blake_g_output_limb_7_col130,
                ];
                *sub_component_inputs.blake_g[6] = [
                    blake_g_output_tmp_39a7a_19[0],
                    blake_g_output_tmp_39a7a_20[1],
                    blake_g_output_tmp_39a7a_17[2],
                    blake_g_output_tmp_39a7a_18[3],
                    blake_message_output_tmp_39a7a_13,
                    blake_message_output_tmp_39a7a_14,
                ];
                let blake_g_output_tmp_39a7a_23 = PackedBlakeG::deduce_output([
                    blake_g_output_tmp_39a7a_19[0],
                    blake_g_output_tmp_39a7a_20[1],
                    blake_g_output_tmp_39a7a_17[2],
                    blake_g_output_tmp_39a7a_18[3],
                    blake_message_output_tmp_39a7a_13,
                    blake_message_output_tmp_39a7a_14,
                ]);
                let blake_g_output_limb_0_col131 = blake_g_output_tmp_39a7a_23[0].low().as_m31();
                *row[131] = blake_g_output_limb_0_col131;
                let blake_g_output_limb_1_col132 = blake_g_output_tmp_39a7a_23[0].high().as_m31();
                *row[132] = blake_g_output_limb_1_col132;
                let blake_g_output_limb_2_col133 = blake_g_output_tmp_39a7a_23[1].low().as_m31();
                *row[133] = blake_g_output_limb_2_col133;
                let blake_g_output_limb_3_col134 = blake_g_output_tmp_39a7a_23[1].high().as_m31();
                *row[134] = blake_g_output_limb_3_col134;
                let blake_g_output_limb_4_col135 = blake_g_output_tmp_39a7a_23[2].low().as_m31();
                *row[135] = blake_g_output_limb_4_col135;
                let blake_g_output_limb_5_col136 = blake_g_output_tmp_39a7a_23[2].high().as_m31();
                *row[136] = blake_g_output_limb_5_col136;
                let blake_g_output_limb_6_col137 = blake_g_output_tmp_39a7a_23[3].low().as_m31();
                *row[137] = blake_g_output_limb_6_col137;
                let blake_g_output_limb_7_col138 = blake_g_output_tmp_39a7a_23[3].high().as_m31();
                *row[138] = blake_g_output_limb_7_col138;
                *lookup_data.blake_g_6 = [
                    M31_1139985212,
                    blake_g_output_limb_0_col99,
                    blake_g_output_limb_1_col100,
                    blake_g_output_limb_2_col109,
                    blake_g_output_limb_3_col110,
                    blake_g_output_limb_4_col87,
                    blake_g_output_limb_5_col88,
                    blake_g_output_limb_6_col97,
                    blake_g_output_limb_7_col98,
                    blake_message_output_message_limb_limb_0_col75,
                    blake_message_output_message_limb_limb_1_col76,
                    blake_message_output_message_limb_limb_0_col77,
                    blake_message_output_message_limb_limb_1_col78,
                    blake_g_output_limb_0_col131,
                    blake_g_output_limb_1_col132,
                    blake_g_output_limb_2_col133,
                    blake_g_output_limb_3_col134,
                    blake_g_output_limb_4_col135,
                    blake_g_output_limb_5_col136,
                    blake_g_output_limb_6_col137,
                    blake_g_output_limb_7_col138,
                ];
                *sub_component_inputs.blake_g[7] = [
                    blake_g_output_tmp_39a7a_20[0],
                    blake_g_output_tmp_39a7a_17[1],
                    blake_g_output_tmp_39a7a_18[2],
                    blake_g_output_tmp_39a7a_19[3],
                    blake_message_output_tmp_39a7a_15,
                    blake_message_output_tmp_39a7a_16,
                ];
                let blake_g_output_tmp_39a7a_24 = PackedBlakeG::deduce_output([
                    blake_g_output_tmp_39a7a_20[0],
                    blake_g_output_tmp_39a7a_17[1],
                    blake_g_output_tmp_39a7a_18[2],
                    blake_g_output_tmp_39a7a_19[3],
                    blake_message_output_tmp_39a7a_15,
                    blake_message_output_tmp_39a7a_16,
                ]);
                let blake_g_output_limb_0_col139 = blake_g_output_tmp_39a7a_24[0].low().as_m31();
                *row[139] = blake_g_output_limb_0_col139;
                let blake_g_output_limb_1_col140 = blake_g_output_tmp_39a7a_24[0].high().as_m31();
                *row[140] = blake_g_output_limb_1_col140;
                let blake_g_output_limb_2_col141 = blake_g_output_tmp_39a7a_24[1].low().as_m31();
                *row[141] = blake_g_output_limb_2_col141;
                let blake_g_output_limb_3_col142 = blake_g_output_tmp_39a7a_24[1].high().as_m31();
                *row[142] = blake_g_output_limb_3_col142;
                let blake_g_output_limb_4_col143 = blake_g_output_tmp_39a7a_24[2].low().as_m31();
                *row[143] = blake_g_output_limb_4_col143;
                let blake_g_output_limb_5_col144 = blake_g_output_tmp_39a7a_24[2].high().as_m31();
                *row[144] = blake_g_output_limb_5_col144;
                let blake_g_output_limb_6_col145 = blake_g_output_tmp_39a7a_24[3].low().as_m31();
                *row[145] = blake_g_output_limb_6_col145;
                let blake_g_output_limb_7_col146 = blake_g_output_tmp_39a7a_24[3].high().as_m31();
                *row[146] = blake_g_output_limb_7_col146;
                *lookup_data.blake_g_7 = [
                    M31_1139985212,
                    blake_g_output_limb_0_col107,
                    blake_g_output_limb_1_col108,
                    blake_g_output_limb_2_col85,
                    blake_g_output_limb_3_col86,
                    blake_g_output_limb_4_col95,
                    blake_g_output_limb_5_col96,
                    blake_g_output_limb_6_col105,
                    blake_g_output_limb_7_col106,
                    blake_message_output_message_limb_limb_0_col79,
                    blake_message_output_message_limb_limb_1_col80,
                    blake_message_output_message_limb_limb_0_col81,
                    blake_message_output_message_limb_limb_1_col82,
                    blake_g_output_limb_0_col139,
                    blake_g_output_limb_1_col140,
                    blake_g_output_limb_2_col141,
                    blake_g_output_limb_3_col142,
                    blake_g_output_limb_4_col143,
                    blake_g_output_limb_5_col144,
                    blake_g_output_limb_6_col145,
                    blake_g_output_limb_7_col146,
                ];
                *lookup_data.blake_round_0 = [
                    M31_40528774,
                    input_limb_0_col0,
                    input_limb_1_col1,
                    input_limb_2_col2,
                    input_limb_3_col3,
                    input_limb_4_col4,
                    input_limb_5_col5,
                    input_limb_6_col6,
                    input_limb_7_col7,
                    input_limb_8_col8,
                    input_limb_9_col9,
                    input_limb_10_col10,
                    input_limb_11_col11,
                    input_limb_12_col12,
                    input_limb_13_col13,
                    input_limb_14_col14,
                    input_limb_15_col15,
                    input_limb_16_col16,
                    input_limb_17_col17,
                    input_limb_18_col18,
                    input_limb_19_col19,
                    input_limb_20_col20,
                    input_limb_21_col21,
                    input_limb_22_col22,
                    input_limb_23_col23,
                    input_limb_24_col24,
                    input_limb_25_col25,
                    input_limb_26_col26,
                    input_limb_27_col27,
                    input_limb_28_col28,
                    input_limb_29_col29,
                    input_limb_30_col30,
                    input_limb_31_col31,
                    input_limb_32_col32,
                    input_limb_33_col33,
                    input_limb_34_col34,
                ];
                *lookup_data.blake_round_1 = [
                    M31_40528774,
                    input_limb_0_col0,
                    ((input_limb_1_col1) + (M31_1)),
                    blake_g_output_limb_0_col115,
                    blake_g_output_limb_1_col116,
                    blake_g_output_limb_0_col123,
                    blake_g_output_limb_1_col124,
                    blake_g_output_limb_0_col131,
                    blake_g_output_limb_1_col132,
                    blake_g_output_limb_0_col139,
                    blake_g_output_limb_1_col140,
                    blake_g_output_limb_2_col141,
                    blake_g_output_limb_3_col142,
                    blake_g_output_limb_2_col117,
                    blake_g_output_limb_3_col118,
                    blake_g_output_limb_2_col125,
                    blake_g_output_limb_3_col126,
                    blake_g_output_limb_2_col133,
                    blake_g_output_limb_3_col134,
                    blake_g_output_limb_4_col135,
                    blake_g_output_limb_5_col136,
                    blake_g_output_limb_4_col143,
                    blake_g_output_limb_5_col144,
                    blake_g_output_limb_4_col119,
                    blake_g_output_limb_5_col120,
                    blake_g_output_limb_4_col127,
                    blake_g_output_limb_5_col128,
                    blake_g_output_limb_6_col129,
                    blake_g_output_limb_7_col130,
                    blake_g_output_limb_6_col137,
                    blake_g_output_limb_7_col138,
                    blake_g_output_limb_6_col145,
                    blake_g_output_limb_7_col146,
                    blake_g_output_limb_6_col121,
                    blake_g_output_limb_7_col122,
                    input_limb_34_col34,
                ];
                *row[147] = enabler_col.packed_at(row_index);
            },
        );

    (trace, lookup_data, sub_component_inputs)
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
    blake_g_0: Vec<[PackedM31; 21]>,
    blake_g_1: Vec<[PackedM31; 21]>,
    blake_g_2: Vec<[PackedM31; 21]>,
    blake_g_3: Vec<[PackedM31; 21]>,
    blake_g_4: Vec<[PackedM31; 21]>,
    blake_g_5: Vec<[PackedM31; 21]>,
    blake_g_6: Vec<[PackedM31; 21]>,
    blake_g_7: Vec<[PackedM31; 21]>,
    blake_message_0: Vec<[PackedM31; 5]>,
    blake_message_1: Vec<[PackedM31; 5]>,
    blake_message_2: Vec<[PackedM31; 5]>,
    blake_message_3: Vec<[PackedM31; 5]>,
    blake_message_4: Vec<[PackedM31; 5]>,
    blake_message_5: Vec<[PackedM31; 5]>,
    blake_message_6: Vec<[PackedM31; 5]>,
    blake_message_7: Vec<[PackedM31; 5]>,
    blake_message_8: Vec<[PackedM31; 5]>,
    blake_message_9: Vec<[PackedM31; 5]>,
    blake_message_10: Vec<[PackedM31; 5]>,
    blake_message_11: Vec<[PackedM31; 5]>,
    blake_message_12: Vec<[PackedM31; 5]>,
    blake_message_13: Vec<[PackedM31; 5]>,
    blake_message_14: Vec<[PackedM31; 5]>,
    blake_message_15: Vec<[PackedM31; 5]>,
    blake_round_0: Vec<[PackedM31; 36]>,
    blake_round_1: Vec<[PackedM31; 36]>,
    blake_round_sigma_0: Vec<[PackedM31; 18]>,
}

pub struct InteractionClaimGenerator {
    n_rows: usize,
    log_size: u32,
    lookup_data: LookupData,
}
impl InteractionClaimGenerator {
    pub fn write_interaction_trace(
        self,
        common_lookup_elements: &relations::CommonLookupElements,
    ) -> (Vec<CircleEvaluation<SimdBackend, M31, BitReversedOrder>>, InteractionClaim) {
        let enabler_col = Enabler::new(self.n_rows);
        let mut logup_gen = LogupTraceGenerator::new(self.log_size);

        // Sum logup terms in pairs.
        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.blake_round_sigma_0,
            &self.lookup_data.blake_message_0,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.blake_message_1,
            &self.lookup_data.blake_message_2,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.blake_message_3,
            &self.lookup_data.blake_message_4,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.blake_message_5,
            &self.lookup_data.blake_message_6,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.blake_message_7,
            &self.lookup_data.blake_message_8,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.blake_message_9,
            &self.lookup_data.blake_message_10,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.blake_message_11,
            &self.lookup_data.blake_message_12,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.blake_message_13,
            &self.lookup_data.blake_message_14,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (col_gen.par_iter_mut(), &self.lookup_data.blake_message_15, &self.lookup_data.blake_g_0)
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (col_gen.par_iter_mut(), &self.lookup_data.blake_g_1, &self.lookup_data.blake_g_2)
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (col_gen.par_iter_mut(), &self.lookup_data.blake_g_3, &self.lookup_data.blake_g_4)
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (col_gen.par_iter_mut(), &self.lookup_data.blake_g_5, &self.lookup_data.blake_g_6)
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (col_gen.par_iter_mut(), &self.lookup_data.blake_g_7, &self.lookup_data.blake_round_0)
            .into_par_iter()
            .enumerate()
            .for_each(|(i, (writer, values0, values1))| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 * enabler_col.packed_at(i) + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        // Sum last logup term.
        let mut col_gen = logup_gen.new_col();
        (col_gen.par_iter_mut(), &self.lookup_data.blake_round_1)
            .into_par_iter()
            .enumerate()
            .for_each(|(i, (writer, values))| {
                let denom = common_lookup_elements.combine(values);
                writer.write_frac(-PackedQM31::one() * enabler_col.packed_at(i), denom);
            });
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize_last();

        (trace, InteractionClaim { claimed_sum })
    }
}
