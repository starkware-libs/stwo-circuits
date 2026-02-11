// This file was created by the AIR team.

#![allow(unused_parens)]
use cairo_air::components::blake_gate::{Claim, InteractionClaim, N_TRACE_COLUMNS};

use crate::witness::components::{
    blake_round, range_check_15, range_check_16, triple_xor_32, verify_bitwise_xor_8,
};
use crate::witness::prelude::*;

pub type PackedInputType = ([[PackedUInt32; 8]; 2], [PackedM31; 16]);

#[derive(Default)]
pub struct ClaimGenerator {
    pub packed_inputs: Vec<PackedInputType>,
    preprocessed_trace: Arc<PreProcessedTrace>,
}

impl ClaimGenerator {
    pub fn new(
        packed_inputs: Vec<PackedInputType>,
        preprocessed_trace: Arc<PreProcessedTrace>,
    ) -> Self {
        Self {
            packed_inputs,
            preprocessed_trace,
        }
    }

    pub fn write_trace(
        self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
        verify_bitwise_xor_8_state: &verify_bitwise_xor_8::ClaimGenerator,
        range_check_16_state: &range_check_16::ClaimGenerator,
        range_check_15_state: &range_check_15::ClaimGenerator,
        blake_round_state: &mut blake_round::ClaimGenerator,
        triple_xor_32_state: &mut triple_xor_32::ClaimGenerator,
    ) -> (Claim, InteractionClaimGenerator) {
        assert!(!self.packed_inputs.is_empty());
        let n_vec_rows = self.packed_inputs.len();
        let n_rows = n_vec_rows * N_LANES;
        let packed_size = n_vec_rows.next_power_of_two();
        let log_size = packed_size.ilog2() + LOG_N_LANES;
        self.packed_inputs
            .resize(packed_size, *self.packed_inputs.first().unwrap());

        let (trace, lookup_data, sub_component_inputs) = write_trace_simd(
            self.packed_inputs,
            &self.preprocessed_trace,
            verify_bitwise_xor_8_state,
            range_check_16_state,
            range_check_15_state,
            blake_round_state,
            triple_xor_32_state,
        );
        for inputs in sub_component_inputs.verify_bitwise_xor_8 {
            verify_bitwise_xor_8_state.add_packed_inputs(&inputs, 0);
        }
        for inputs in sub_component_inputs.range_check_16 {
            range_check_16_state.add_packed_inputs(&inputs, 0);
        }
        for inputs in sub_component_inputs.range_check_15 {
            range_check_15_state.add_packed_inputs(&inputs, 0);
        }
        for inputs in sub_component_inputs.blake_round {
            blake_round_state.add_packed_inputs(&inputs, 0);
        }
        for inputs in sub_component_inputs.triple_xor_32 {
            triple_xor_32_state.add_packed_inputs(&inputs, 0);
        }
        tree_builder.extend_evals(trace.to_evals());

        (
            Claim { log_size },
            InteractionClaimGenerator {
                log_size,
                lookup_data,
            },
        )
    }
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct SubComponentInputs {
    verify_bitwise_xor_8: [Vec<verify_bitwise_xor_8::PackedInputType>; 4],
    range_check_16: [Vec<range_check_16::PackedInputType>; 16],
    range_check_15: [Vec<range_check_15::PackedInputType>; 16],
    blake_round: [Vec<blake_round::PackedInputType>; 10],
    triple_xor_32: [Vec<triple_xor_32::PackedInputType>; 8],
}

#[allow(clippy::useless_conversion)]
#[allow(unused_variables)]
#[allow(clippy::double_parens)]
#[allow(non_snake_case)]
fn write_trace_simd(
    packed_inputs: Vec<PackedInputType>,
    preprocessed_trace: &PreProcessedTrace,
    verify_bitwise_xor_8_state: &verify_bitwise_xor_8::ClaimGenerator,
    range_check_16_state: &range_check_16::ClaimGenerator,
    range_check_15_state: &range_check_15::ClaimGenerator,
    blake_round_state: &mut blake_round::ClaimGenerator,
    triple_xor_32_state: &mut triple_xor_32::ClaimGenerator,
) -> (
    ComponentTrace<N_TRACE_COLUMNS>,
    LookupData,
    SubComponentInputs,
) {
    let log_n_packed_rows = inputs.len().ilog2();
    let log_size = log_n_packed_rows + LOG_N_LANES;
    let (mut trace, mut lookup_data, mut sub_component_inputs) = unsafe {
        (
            ComponentTrace::<N_TRACE_COLUMNS>::uninitialized(log_size),
            LookupData::uninitialized(log_n_packed_rows),
            SubComponentInputs::uninitialized(log_n_packed_rows),
        )
    };

    let M31_0 = PackedM31::broadcast(M31::from(0));
    let M31_1 = PackedM31::broadcast(M31::from(1));
    let M31_10 = PackedM31::broadcast(M31::from(10));
    let M31_1008385708 = PackedM31::broadcast(M31::from(1008385708));
    let M31_1058718565 = PackedM31::broadcast(M31::from(1058718565));
    let M31_1061955672 = PackedM31::broadcast(M31::from(1061955672));
    let M31_11 = PackedM31::broadcast(M31::from(11));
    let M31_112558620 = PackedM31::broadcast(M31::from(112558620));
    let M31_12 = PackedM31::broadcast(M31::from(12));
    let M31_127 = PackedM31::broadcast(M31::from(127));
    let M31_13 = PackedM31::broadcast(M31::from(13));
    let M31_14 = PackedM31::broadcast(M31::from(14));
    let M31_1492981981 = PackedM31::broadcast(M31::from(1492981981));
    let M31_15 = PackedM31::broadcast(M31::from(15));
    let M31_15470 = PackedM31::broadcast(M31::from(15470));
    let M31_2 = PackedM31::broadcast(M31::from(2));
    let M31_23520 = PackedM31::broadcast(M31::from(23520));
    let M31_256 = PackedM31::broadcast(M31::from(256));
    let M31_26764 = PackedM31::broadcast(M31::from(26764));
    let M31_27145 = PackedM31::broadcast(M31::from(27145));
    let M31_3 = PackedM31::broadcast(M31::from(3));
    let M31_378353459 = PackedM31::broadcast(M31::from(378353459));
    let M31_39685 = PackedM31::broadcast(M31::from(39685));
    let M31_4 = PackedM31::broadcast(M31::from(4));
    let M31_40528774 = PackedM31::broadcast(M31::from(40528774));
    let M31_42319 = PackedM31::broadcast(M31::from(42319));
    let M31_44677 = PackedM31::broadcast(M31::from(44677));
    let M31_47975 = PackedM31::broadcast(M31::from(47975));
    let M31_5 = PackedM31::broadcast(M31::from(5));
    let M31_52505 = PackedM31::broadcast(M31::from(52505));
    let M31_55723 = PackedM31::broadcast(M31::from(55723));
    let M31_57468 = PackedM31::broadcast(M31::from(57468));
    let M31_58983 = PackedM31::broadcast(M31::from(58983));
    let M31_6 = PackedM31::broadcast(M31::from(6));
    let M31_62322 = PackedM31::broadcast(M31::from(62322));
    let M31_62778 = PackedM31::broadcast(M31::from(62778));
    let M31_7 = PackedM31::broadcast(M31::from(7));
    let M31_8 = PackedM31::broadcast(M31::from(8));
    let M31_8067 = PackedM31::broadcast(M31::from(8067));
    let M31_81 = PackedM31::broadcast(M31::from(81));
    let M31_82 = PackedM31::broadcast(M31::from(82));
    let M31_9 = PackedM31::broadcast(M31::from(9));
    let M31_9812 = PackedM31::broadcast(M31::from(9812));
    let M31_990559919 = PackedM31::broadcast(M31::from(990559919));
    let UInt16_127 = PackedUInt16::broadcast(UInt16::from(127));
    let UInt16_14 = PackedUInt16::broadcast(UInt16::from(14));
    let UInt16_8 = PackedUInt16::broadcast(UInt16::from(8));
    let UInt16_81 = PackedUInt16::broadcast(UInt16::from(81));
    let UInt16_82 = PackedUInt16::broadcast(UInt16::from(82));
    let UInt32_1013904242 = PackedUInt32::broadcast(UInt32::from(1013904242));
    let UInt32_1541459225 = PackedUInt32::broadcast(UInt32::from(1541459225));
    let UInt32_1779033703 = PackedUInt32::broadcast(UInt32::from(1779033703));
    let UInt32_2600822924 = PackedUInt32::broadcast(UInt32::from(2600822924));
    let UInt32_2773480762 = PackedUInt32::broadcast(UInt32::from(2773480762));
    let UInt32_3144134277 = PackedUInt32::broadcast(UInt32::from(3144134277));
    let t0 = preprocessed_trace.get_column(&PreProcessedColumnId {
        id: "t0".to_owned(),
    });
    let t1 = preprocessed_trace.get_column(&PreProcessedColumnId {
        id: "t1".to_owned(),
    });
    let finalize_flag = preprocessed_trace.get_column(&PreProcessedColumnId {
        id: "finalize_flag".to_owned(),
    });
    let seq = Seq::new(log_size);
    let state_before_addr = preprocessed_trace.get_column(&PreProcessedColumnId {
        id: "state_before_addr".to_owned(),
    });
    let state_after_addr = preprocessed_trace.get_column(&PreProcessedColumnId {
        id: "state_after_addr".to_owned(),
    });
    let message0_addr = preprocessed_trace.get_column(&PreProcessedColumnId {
        id: "message0_addr".to_owned(),
    });
    let message1_addr = preprocessed_trace.get_column(&PreProcessedColumnId {
        id: "message1_addr".to_owned(),
    });
    let message2_addr = preprocessed_trace.get_column(&PreProcessedColumnId {
        id: "message2_addr".to_owned(),
    });
    let message3_addr = preprocessed_trace.get_column(&PreProcessedColumnId {
        id: "message3_addr".to_owned(),
    });
    let mults = preprocessed_trace.get_column(&PreProcessedColumnId {
        id: blake_gate_mults,
    });

    (
        trace.par_iter_mut(),
        lookup_data.par_iter_mut(),
        sub_component_inputs.par_iter_mut(),
        inputs.into_par_iter(),
    )
        .into_par_iter()
        .enumerate()
        .for_each(
            |(row_index, (row, lookup_data, sub_component_inputs, blake_gate_input))| {
                let t0 = t0.packed_at(row_index);
                let t1 = t1.packed_at(row_index);
                let finalize_flag = finalize_flag.packed_at(row_index);
                let seq = seq.packed_at(row_index);
                let state_before_addr = state_before_addr.packed_at(row_index);
                let state_after_addr = state_after_addr.packed_at(row_index);
                let message0_addr = message0_addr.packed_at(row_index);
                let message1_addr = message1_addr.packed_at(row_index);
                let message2_addr = message2_addr.packed_at(row_index);
                let message3_addr = message3_addr.packed_at(row_index);
                let input_state_before_limb0_limb_0_col0 = blake_gate_input.0[0][0].low().as_m31();
                *row[0] = input_state_before_limb0_limb_0_col0;
                let input_state_before_limb0_limb_1_col1 = blake_gate_input.0[0][0].high().as_m31();
                *row[1] = input_state_before_limb0_limb_1_col1;
                let input_state_before_limb1_limb_0_col2 = blake_gate_input.0[0][1].low().as_m31();
                *row[2] = input_state_before_limb1_limb_0_col2;
                let input_state_before_limb1_limb_1_col3 = blake_gate_input.0[0][1].high().as_m31();
                *row[3] = input_state_before_limb1_limb_1_col3;
                let input_state_before_limb2_limb_0_col4 = blake_gate_input.0[0][2].low().as_m31();
                *row[4] = input_state_before_limb2_limb_0_col4;
                let input_state_before_limb2_limb_1_col5 = blake_gate_input.0[0][2].high().as_m31();
                *row[5] = input_state_before_limb2_limb_1_col5;
                let input_state_before_limb3_limb_0_col6 = blake_gate_input.0[0][3].low().as_m31();
                *row[6] = input_state_before_limb3_limb_0_col6;
                let input_state_before_limb3_limb_1_col7 = blake_gate_input.0[0][3].high().as_m31();
                *row[7] = input_state_before_limb3_limb_1_col7;
                let input_state_before_limb4_limb_0_col8 = blake_gate_input.0[0][4].low().as_m31();
                *row[8] = input_state_before_limb4_limb_0_col8;
                let input_state_before_limb4_limb_1_col9 = blake_gate_input.0[0][4].high().as_m31();
                *row[9] = input_state_before_limb4_limb_1_col9;
                let input_state_before_limb5_limb_0_col10 = blake_gate_input.0[0][5].low().as_m31();
                *row[10] = input_state_before_limb5_limb_0_col10;
                let input_state_before_limb5_limb_1_col11 =
                    blake_gate_input.0[0][5].high().as_m31();
                *row[11] = input_state_before_limb5_limb_1_col11;
                let input_state_before_limb6_limb_0_col12 = blake_gate_input.0[0][6].low().as_m31();
                *row[12] = input_state_before_limb6_limb_0_col12;
                let input_state_before_limb6_limb_1_col13 =
                    blake_gate_input.0[0][6].high().as_m31();
                *row[13] = input_state_before_limb6_limb_1_col13;
                let input_state_before_limb7_limb_0_col14 = blake_gate_input.0[0][7].low().as_m31();
                *row[14] = input_state_before_limb7_limb_0_col14;
                let input_state_before_limb7_limb_1_col15 =
                    blake_gate_input.0[0][7].high().as_m31();
                *row[15] = input_state_before_limb7_limb_1_col15;
                let input_state_after_limb0_limb_0_col16 = blake_gate_input.0[1][0].low().as_m31();
                *row[16] = input_state_after_limb0_limb_0_col16;
                let input_state_after_limb0_limb_1_col17 = blake_gate_input.0[1][0].high().as_m31();
                *row[17] = input_state_after_limb0_limb_1_col17;
                let input_state_after_limb1_limb_0_col18 = blake_gate_input.0[1][1].low().as_m31();
                *row[18] = input_state_after_limb1_limb_0_col18;
                let input_state_after_limb1_limb_1_col19 = blake_gate_input.0[1][1].high().as_m31();
                *row[19] = input_state_after_limb1_limb_1_col19;
                let input_state_after_limb2_limb_0_col20 = blake_gate_input.0[1][2].low().as_m31();
                *row[20] = input_state_after_limb2_limb_0_col20;
                let input_state_after_limb2_limb_1_col21 = blake_gate_input.0[1][2].high().as_m31();
                *row[21] = input_state_after_limb2_limb_1_col21;
                let input_state_after_limb3_limb_0_col22 = blake_gate_input.0[1][3].low().as_m31();
                *row[22] = input_state_after_limb3_limb_0_col22;
                let input_state_after_limb3_limb_1_col23 = blake_gate_input.0[1][3].high().as_m31();
                *row[23] = input_state_after_limb3_limb_1_col23;
                let input_state_after_limb4_limb_0_col24 = blake_gate_input.0[1][4].low().as_m31();
                *row[24] = input_state_after_limb4_limb_0_col24;
                let input_state_after_limb4_limb_1_col25 = blake_gate_input.0[1][4].high().as_m31();
                *row[25] = input_state_after_limb4_limb_1_col25;
                let input_state_after_limb5_limb_0_col26 = blake_gate_input.0[1][5].low().as_m31();
                *row[26] = input_state_after_limb5_limb_0_col26;
                let input_state_after_limb5_limb_1_col27 = blake_gate_input.0[1][5].high().as_m31();
                *row[27] = input_state_after_limb5_limb_1_col27;
                let input_state_after_limb6_limb_0_col28 = blake_gate_input.0[1][6].low().as_m31();
                *row[28] = input_state_after_limb6_limb_0_col28;
                let input_state_after_limb6_limb_1_col29 = blake_gate_input.0[1][6].high().as_m31();
                *row[29] = input_state_after_limb6_limb_1_col29;
                let input_state_after_limb7_limb_0_col30 = blake_gate_input.0[1][7].low().as_m31();
                *row[30] = input_state_after_limb7_limb_0_col30;
                let input_state_after_limb7_limb_1_col31 = blake_gate_input.0[1][7].high().as_m31();
                *row[31] = input_state_after_limb7_limb_1_col31;
                let input_message_limb0_col32 = blake_gate_input.1[0];
                *row[32] = input_message_limb0_col32;
                let input_message_limb1_col33 = blake_gate_input.1[1];
                *row[33] = input_message_limb1_col33;
                let input_message_limb2_col34 = blake_gate_input.1[2];
                *row[34] = input_message_limb2_col34;
                let input_message_limb3_col35 = blake_gate_input.1[3];
                *row[35] = input_message_limb3_col35;
                let input_message_limb4_col36 = blake_gate_input.1[4];
                *row[36] = input_message_limb4_col36;
                let input_message_limb5_col37 = blake_gate_input.1[5];
                *row[37] = input_message_limb5_col37;
                let input_message_limb6_col38 = blake_gate_input.1[6];
                *row[38] = input_message_limb6_col38;
                let input_message_limb7_col39 = blake_gate_input.1[7];
                *row[39] = input_message_limb7_col39;
                let input_message_limb8_col40 = blake_gate_input.1[8];
                *row[40] = input_message_limb8_col40;
                let input_message_limb9_col41 = blake_gate_input.1[9];
                *row[41] = input_message_limb9_col41;
                let input_message_limb10_col42 = blake_gate_input.1[10];
                *row[42] = input_message_limb10_col42;
                let input_message_limb11_col43 = blake_gate_input.1[11];
                *row[43] = input_message_limb11_col43;
                let input_message_limb12_col44 = blake_gate_input.1[12];
                *row[44] = input_message_limb12_col44;
                let input_message_limb13_col45 = blake_gate_input.1[13];
                *row[45] = input_message_limb13_col45;
                let input_message_limb14_col46 = blake_gate_input.1[14];
                *row[46] = input_message_limb14_col46;
                let input_message_limb15_col47 = blake_gate_input.1[15];
                *row[47] = input_message_limb15_col47;

                // Create Blake Round Input.

                // Split 16 Low Part Size 8.

                let ms_8_bits_tmp_8e0ec_0 = ((PackedUInt16::from_m31(t0)) >> (UInt16_8));
                let ms_8_bits_col48 = ms_8_bits_tmp_8e0ec_0.as_m31();
                *row[48] = ms_8_bits_col48;
                let split_16_low_part_size_8_output_tmp_8e0ec_1 =
                    [((t0) - ((ms_8_bits_col48) * (M31_256))), ms_8_bits_col48];

                // Split 16 Low Part Size 8.

                let ms_8_bits_tmp_8e0ec_2 = ((PackedUInt16::from_m31(t1)) >> (UInt16_8));
                let ms_8_bits_col49 = ms_8_bits_tmp_8e0ec_2.as_m31();
                *row[49] = ms_8_bits_col49;
                let split_16_low_part_size_8_output_tmp_8e0ec_3 =
                    [((t1) - ((ms_8_bits_col49) * (M31_256))), ms_8_bits_col49];

                // Bitwise Xor Num Bits 8.

                let xor_tmp_8e0ec_4 =
                    ((PackedUInt16::from_m31(split_16_low_part_size_8_output_tmp_8e0ec_1[0]))
                        ^ (UInt16_127));
                let xor_col50 = xor_tmp_8e0ec_4.as_m31();
                *row[50] = xor_col50;
                *sub_component_inputs.verify_bitwise_xor_8[0] = [
                    split_16_low_part_size_8_output_tmp_8e0ec_1[0],
                    M31_127,
                    xor_col50,
                ];
                *lookup_data.verify_bitwise_xor_8_0 = [
                    M31_112558620,
                    split_16_low_part_size_8_output_tmp_8e0ec_1[0],
                    M31_127,
                    xor_col50,
                ];

                // Bitwise Xor Num Bits 8.

                let xor_tmp_8e0ec_6 = ((PackedUInt16::from_m31(ms_8_bits_col48)) ^ (UInt16_82));
                let xor_col51 = xor_tmp_8e0ec_6.as_m31();
                *row[51] = xor_col51;
                *sub_component_inputs.verify_bitwise_xor_8[1] =
                    [ms_8_bits_col48, M31_82, xor_col51];
                *lookup_data.verify_bitwise_xor_8_1 =
                    [M31_112558620, ms_8_bits_col48, M31_82, xor_col51];

                // Bitwise Xor Num Bits 8.

                let xor_tmp_8e0ec_8 =
                    ((PackedUInt16::from_m31(split_16_low_part_size_8_output_tmp_8e0ec_3[0]))
                        ^ (UInt16_14));
                let xor_col52 = xor_tmp_8e0ec_8.as_m31();
                *row[52] = xor_col52;
                *sub_component_inputs.verify_bitwise_xor_8[2] = [
                    split_16_low_part_size_8_output_tmp_8e0ec_3[0],
                    M31_14,
                    xor_col52,
                ];
                *lookup_data.verify_bitwise_xor_8_2 = [
                    M31_112558620,
                    split_16_low_part_size_8_output_tmp_8e0ec_3[0],
                    M31_14,
                    xor_col52,
                ];

                // Bitwise Xor Num Bits 8.

                let xor_tmp_8e0ec_10 = ((PackedUInt16::from_m31(ms_8_bits_col49)) ^ (UInt16_81));
                let xor_col53 = xor_tmp_8e0ec_10.as_m31();
                *row[53] = xor_col53;
                *sub_component_inputs.verify_bitwise_xor_8[3] =
                    [ms_8_bits_col49, M31_81, xor_col53];
                *lookup_data.verify_bitwise_xor_8_3 =
                    [M31_112558620, ms_8_bits_col49, M31_81, xor_col53];

                let create_blake_round_input_output_tmp_8e0ec_12 = [
                    blake_gate_input.0[0][0],
                    blake_gate_input.0[0][1],
                    blake_gate_input.0[0][2],
                    blake_gate_input.0[0][3],
                    blake_gate_input.0[0][4],
                    blake_gate_input.0[0][5],
                    blake_gate_input.0[0][6],
                    blake_gate_input.0[0][7],
                    UInt32_1779033703,
                    UInt32_3144134277,
                    UInt32_1013904242,
                    UInt32_2773480762,
                    PackedUInt32::from_limbs([
                        ((xor_col50) + ((xor_col51) * (M31_256))),
                        ((xor_col52) + ((xor_col53) * (M31_256))),
                    ]),
                    UInt32_2600822924,
                    PackedUInt32::from_limbs([
                        (((finalize_flag) * (M31_9812))
                            + (((M31_1) - (finalize_flag)) * (M31_55723))),
                        (((finalize_flag) * (M31_57468))
                            + (((M31_1) - (finalize_flag)) * (M31_8067))),
                    ]),
                    UInt32_1541459225,
                ];

                // Qm 31 Into U 32.

                let limb0_tmp_8e0ec_13 = PackedUInt32::from_m31(input_message_limb0_col32);
                let limb0_limb_0_col54 = input_message_limb0_col32;
                *row[54] = limb0_limb_0_col54;
                let limb0_limb_1_col55 = limb0_tmp_8e0ec_13.high().as_m31();
                *row[55] = limb0_limb_1_col55;
                *sub_component_inputs.range_check_16[0] = [limb0_limb_0_col54];
                *lookup_data.range_check_16_0 = [M31_1008385708, limb0_limb_0_col54];
                *sub_component_inputs.range_check_15[0] = [limb0_limb_1_col55];
                *lookup_data.range_check_15_0 = [M31_1058718565, limb0_limb_1_col55];
                *lookup_data.blake_message_0 = [
                    M31_1492981981,
                    seq,
                    M31_0,
                    limb0_limb_0_col54,
                    limb0_limb_1_col55,
                ];
                let limb1_tmp_8e0ec_14 = PackedUInt32::from_m31(input_message_limb1_col33);
                let limb1_limb_0_col56 = input_message_limb1_col33;
                *row[56] = limb1_limb_0_col56;
                let limb1_limb_1_col57 = limb1_tmp_8e0ec_14.high().as_m31();
                *row[57] = limb1_limb_1_col57;
                *sub_component_inputs.range_check_16[1] = [limb1_limb_0_col56];
                *lookup_data.range_check_16_1 = [M31_1008385708, limb1_limb_0_col56];
                *sub_component_inputs.range_check_15[1] = [limb1_limb_1_col57];
                *lookup_data.range_check_15_1 = [M31_1058718565, limb1_limb_1_col57];
                *lookup_data.blake_message_1 = [
                    M31_1492981981,
                    seq,
                    M31_1,
                    limb1_limb_0_col56,
                    limb1_limb_1_col57,
                ];
                let limb2_tmp_8e0ec_15 = PackedUInt32::from_m31(input_message_limb2_col34);
                let limb2_limb_0_col58 = input_message_limb2_col34;
                *row[58] = limb2_limb_0_col58;
                let limb2_limb_1_col59 = limb2_tmp_8e0ec_15.high().as_m31();
                *row[59] = limb2_limb_1_col59;
                *sub_component_inputs.range_check_16[2] = [limb2_limb_0_col58];
                *lookup_data.range_check_16_2 = [M31_1008385708, limb2_limb_0_col58];
                *sub_component_inputs.range_check_15[2] = [limb2_limb_1_col59];
                *lookup_data.range_check_15_2 = [M31_1058718565, limb2_limb_1_col59];
                *lookup_data.blake_message_2 = [
                    M31_1492981981,
                    seq,
                    M31_2,
                    limb2_limb_0_col58,
                    limb2_limb_1_col59,
                ];
                let limb3_tmp_8e0ec_16 = PackedUInt32::from_m31(input_message_limb3_col35);
                let limb3_limb_0_col60 = input_message_limb3_col35;
                *row[60] = limb3_limb_0_col60;
                let limb3_limb_1_col61 = limb3_tmp_8e0ec_16.high().as_m31();
                *row[61] = limb3_limb_1_col61;
                *sub_component_inputs.range_check_16[3] = [limb3_limb_0_col60];
                *lookup_data.range_check_16_3 = [M31_1008385708, limb3_limb_0_col60];
                *sub_component_inputs.range_check_15[3] = [limb3_limb_1_col61];
                *lookup_data.range_check_15_3 = [M31_1058718565, limb3_limb_1_col61];
                *lookup_data.blake_message_3 = [
                    M31_1492981981,
                    seq,
                    M31_3,
                    limb3_limb_0_col60,
                    limb3_limb_1_col61,
                ];
                let limb4_tmp_8e0ec_17 = PackedUInt32::from_m31(input_message_limb4_col36);
                let limb4_limb_0_col62 = input_message_limb4_col36;
                *row[62] = limb4_limb_0_col62;
                let limb4_limb_1_col63 = limb4_tmp_8e0ec_17.high().as_m31();
                *row[63] = limb4_limb_1_col63;
                *sub_component_inputs.range_check_16[4] = [limb4_limb_0_col62];
                *lookup_data.range_check_16_4 = [M31_1008385708, limb4_limb_0_col62];
                *sub_component_inputs.range_check_15[4] = [limb4_limb_1_col63];
                *lookup_data.range_check_15_4 = [M31_1058718565, limb4_limb_1_col63];
                *lookup_data.blake_message_4 = [
                    M31_1492981981,
                    seq,
                    M31_4,
                    limb4_limb_0_col62,
                    limb4_limb_1_col63,
                ];
                let limb5_tmp_8e0ec_18 = PackedUInt32::from_m31(input_message_limb5_col37);
                let limb5_limb_0_col64 = input_message_limb5_col37;
                *row[64] = limb5_limb_0_col64;
                let limb5_limb_1_col65 = limb5_tmp_8e0ec_18.high().as_m31();
                *row[65] = limb5_limb_1_col65;
                *sub_component_inputs.range_check_16[5] = [limb5_limb_0_col64];
                *lookup_data.range_check_16_5 = [M31_1008385708, limb5_limb_0_col64];
                *sub_component_inputs.range_check_15[5] = [limb5_limb_1_col65];
                *lookup_data.range_check_15_5 = [M31_1058718565, limb5_limb_1_col65];
                *lookup_data.blake_message_5 = [
                    M31_1492981981,
                    seq,
                    M31_5,
                    limb5_limb_0_col64,
                    limb5_limb_1_col65,
                ];
                let limb6_tmp_8e0ec_19 = PackedUInt32::from_m31(input_message_limb6_col38);
                let limb6_limb_0_col66 = input_message_limb6_col38;
                *row[66] = limb6_limb_0_col66;
                let limb6_limb_1_col67 = limb6_tmp_8e0ec_19.high().as_m31();
                *row[67] = limb6_limb_1_col67;
                *sub_component_inputs.range_check_16[6] = [limb6_limb_0_col66];
                *lookup_data.range_check_16_6 = [M31_1008385708, limb6_limb_0_col66];
                *sub_component_inputs.range_check_15[6] = [limb6_limb_1_col67];
                *lookup_data.range_check_15_6 = [M31_1058718565, limb6_limb_1_col67];
                *lookup_data.blake_message_6 = [
                    M31_1492981981,
                    seq,
                    M31_6,
                    limb6_limb_0_col66,
                    limb6_limb_1_col67,
                ];
                let limb7_tmp_8e0ec_20 = PackedUInt32::from_m31(input_message_limb7_col39);
                let limb7_limb_0_col68 = input_message_limb7_col39;
                *row[68] = limb7_limb_0_col68;
                let limb7_limb_1_col69 = limb7_tmp_8e0ec_20.high().as_m31();
                *row[69] = limb7_limb_1_col69;
                *sub_component_inputs.range_check_16[7] = [limb7_limb_0_col68];
                *lookup_data.range_check_16_7 = [M31_1008385708, limb7_limb_0_col68];
                *sub_component_inputs.range_check_15[7] = [limb7_limb_1_col69];
                *lookup_data.range_check_15_7 = [M31_1058718565, limb7_limb_1_col69];
                *lookup_data.blake_message_7 = [
                    M31_1492981981,
                    seq,
                    M31_7,
                    limb7_limb_0_col68,
                    limb7_limb_1_col69,
                ];
                let limb8_tmp_8e0ec_21 = PackedUInt32::from_m31(input_message_limb8_col40);
                let limb8_limb_0_col70 = input_message_limb8_col40;
                *row[70] = limb8_limb_0_col70;
                let limb8_limb_1_col71 = limb8_tmp_8e0ec_21.high().as_m31();
                *row[71] = limb8_limb_1_col71;
                *sub_component_inputs.range_check_16[8] = [limb8_limb_0_col70];
                *lookup_data.range_check_16_8 = [M31_1008385708, limb8_limb_0_col70];
                *sub_component_inputs.range_check_15[8] = [limb8_limb_1_col71];
                *lookup_data.range_check_15_8 = [M31_1058718565, limb8_limb_1_col71];
                *lookup_data.blake_message_8 = [
                    M31_1492981981,
                    seq,
                    M31_8,
                    limb8_limb_0_col70,
                    limb8_limb_1_col71,
                ];
                let limb9_tmp_8e0ec_22 = PackedUInt32::from_m31(input_message_limb9_col41);
                let limb9_limb_0_col72 = input_message_limb9_col41;
                *row[72] = limb9_limb_0_col72;
                let limb9_limb_1_col73 = limb9_tmp_8e0ec_22.high().as_m31();
                *row[73] = limb9_limb_1_col73;
                *sub_component_inputs.range_check_16[9] = [limb9_limb_0_col72];
                *lookup_data.range_check_16_9 = [M31_1008385708, limb9_limb_0_col72];
                *sub_component_inputs.range_check_15[9] = [limb9_limb_1_col73];
                *lookup_data.range_check_15_9 = [M31_1058718565, limb9_limb_1_col73];
                *lookup_data.blake_message_9 = [
                    M31_1492981981,
                    seq,
                    M31_9,
                    limb9_limb_0_col72,
                    limb9_limb_1_col73,
                ];
                let limb10_tmp_8e0ec_23 = PackedUInt32::from_m31(input_message_limb10_col42);
                let limb10_limb_0_col74 = input_message_limb10_col42;
                *row[74] = limb10_limb_0_col74;
                let limb10_limb_1_col75 = limb10_tmp_8e0ec_23.high().as_m31();
                *row[75] = limb10_limb_1_col75;
                *sub_component_inputs.range_check_16[10] = [limb10_limb_0_col74];
                *lookup_data.range_check_16_10 = [M31_1008385708, limb10_limb_0_col74];
                *sub_component_inputs.range_check_15[10] = [limb10_limb_1_col75];
                *lookup_data.range_check_15_10 = [M31_1058718565, limb10_limb_1_col75];
                *lookup_data.blake_message_10 = [
                    M31_1492981981,
                    seq,
                    M31_10,
                    limb10_limb_0_col74,
                    limb10_limb_1_col75,
                ];
                let limb11_tmp_8e0ec_24 = PackedUInt32::from_m31(input_message_limb11_col43);
                let limb11_limb_0_col76 = input_message_limb11_col43;
                *row[76] = limb11_limb_0_col76;
                let limb11_limb_1_col77 = limb11_tmp_8e0ec_24.high().as_m31();
                *row[77] = limb11_limb_1_col77;
                *sub_component_inputs.range_check_16[11] = [limb11_limb_0_col76];
                *lookup_data.range_check_16_11 = [M31_1008385708, limb11_limb_0_col76];
                *sub_component_inputs.range_check_15[11] = [limb11_limb_1_col77];
                *lookup_data.range_check_15_11 = [M31_1058718565, limb11_limb_1_col77];
                *lookup_data.blake_message_11 = [
                    M31_1492981981,
                    seq,
                    M31_11,
                    limb11_limb_0_col76,
                    limb11_limb_1_col77,
                ];
                let limb12_tmp_8e0ec_25 = PackedUInt32::from_m31(input_message_limb12_col44);
                let limb12_limb_0_col78 = input_message_limb12_col44;
                *row[78] = limb12_limb_0_col78;
                let limb12_limb_1_col79 = limb12_tmp_8e0ec_25.high().as_m31();
                *row[79] = limb12_limb_1_col79;
                *sub_component_inputs.range_check_16[12] = [limb12_limb_0_col78];
                *lookup_data.range_check_16_12 = [M31_1008385708, limb12_limb_0_col78];
                *sub_component_inputs.range_check_15[12] = [limb12_limb_1_col79];
                *lookup_data.range_check_15_12 = [M31_1058718565, limb12_limb_1_col79];
                *lookup_data.blake_message_12 = [
                    M31_1492981981,
                    seq,
                    M31_12,
                    limb12_limb_0_col78,
                    limb12_limb_1_col79,
                ];
                let limb13_tmp_8e0ec_26 = PackedUInt32::from_m31(input_message_limb13_col45);
                let limb13_limb_0_col80 = input_message_limb13_col45;
                *row[80] = limb13_limb_0_col80;
                let limb13_limb_1_col81 = limb13_tmp_8e0ec_26.high().as_m31();
                *row[81] = limb13_limb_1_col81;
                *sub_component_inputs.range_check_16[13] = [limb13_limb_0_col80];
                *lookup_data.range_check_16_13 = [M31_1008385708, limb13_limb_0_col80];
                *sub_component_inputs.range_check_15[13] = [limb13_limb_1_col81];
                *lookup_data.range_check_15_13 = [M31_1058718565, limb13_limb_1_col81];
                *lookup_data.blake_message_13 = [
                    M31_1492981981,
                    seq,
                    M31_13,
                    limb13_limb_0_col80,
                    limb13_limb_1_col81,
                ];
                let limb14_tmp_8e0ec_27 = PackedUInt32::from_m31(input_message_limb14_col46);
                let limb14_limb_0_col82 = input_message_limb14_col46;
                *row[82] = limb14_limb_0_col82;
                let limb14_limb_1_col83 = limb14_tmp_8e0ec_27.high().as_m31();
                *row[83] = limb14_limb_1_col83;
                *sub_component_inputs.range_check_16[14] = [limb14_limb_0_col82];
                *lookup_data.range_check_16_14 = [M31_1008385708, limb14_limb_0_col82];
                *sub_component_inputs.range_check_15[14] = [limb14_limb_1_col83];
                *lookup_data.range_check_15_14 = [M31_1058718565, limb14_limb_1_col83];
                *lookup_data.blake_message_14 = [
                    M31_1492981981,
                    seq,
                    M31_14,
                    limb14_limb_0_col82,
                    limb14_limb_1_col83,
                ];
                let limb15_tmp_8e0ec_28 = PackedUInt32::from_m31(input_message_limb15_col47);
                let limb15_limb_0_col84 = input_message_limb15_col47;
                *row[84] = limb15_limb_0_col84;
                let limb15_limb_1_col85 = limb15_tmp_8e0ec_28.high().as_m31();
                *row[85] = limb15_limb_1_col85;
                *sub_component_inputs.range_check_16[15] = [limb15_limb_0_col84];
                *lookup_data.range_check_16_15 = [M31_1008385708, limb15_limb_0_col84];
                *sub_component_inputs.range_check_15[15] = [limb15_limb_1_col85];
                *lookup_data.range_check_15_15 = [M31_1058718565, limb15_limb_1_col85];
                *lookup_data.blake_message_15 = [
                    M31_1492981981,
                    seq,
                    M31_15,
                    limb15_limb_0_col84,
                    limb15_limb_1_col85,
                ];
                let qm_31_into_u_32_output_tmp_8e0ec_29 = [
                    limb0_tmp_8e0ec_13,
                    limb1_tmp_8e0ec_14,
                    limb2_tmp_8e0ec_15,
                    limb3_tmp_8e0ec_16,
                    limb4_tmp_8e0ec_17,
                    limb5_tmp_8e0ec_18,
                    limb6_tmp_8e0ec_19,
                    limb7_tmp_8e0ec_20,
                    limb8_tmp_8e0ec_21,
                    limb9_tmp_8e0ec_22,
                    limb10_tmp_8e0ec_23,
                    limb11_tmp_8e0ec_24,
                    limb12_tmp_8e0ec_25,
                    limb13_tmp_8e0ec_26,
                    limb14_tmp_8e0ec_27,
                    limb15_tmp_8e0ec_28,
                ];

                *lookup_data.blake_round_0 = [
                    M31_40528774,
                    seq,
                    M31_0,
                    input_state_before_limb0_limb_0_col0,
                    input_state_before_limb0_limb_1_col1,
                    input_state_before_limb1_limb_0_col2,
                    input_state_before_limb1_limb_1_col3,
                    input_state_before_limb2_limb_0_col4,
                    input_state_before_limb2_limb_1_col5,
                    input_state_before_limb3_limb_0_col6,
                    input_state_before_limb3_limb_1_col7,
                    input_state_before_limb4_limb_0_col8,
                    input_state_before_limb4_limb_1_col9,
                    input_state_before_limb5_limb_0_col10,
                    input_state_before_limb5_limb_1_col11,
                    input_state_before_limb6_limb_0_col12,
                    input_state_before_limb6_limb_1_col13,
                    input_state_before_limb7_limb_0_col14,
                    input_state_before_limb7_limb_1_col15,
                    M31_58983,
                    M31_27145,
                    M31_44677,
                    M31_47975,
                    M31_62322,
                    M31_15470,
                    M31_62778,
                    M31_42319,
                    create_blake_round_input_output_tmp_8e0ec_12[12]
                        .low()
                        .as_m31(),
                    create_blake_round_input_output_tmp_8e0ec_12[12]
                        .high()
                        .as_m31(),
                    M31_26764,
                    M31_39685,
                    create_blake_round_input_output_tmp_8e0ec_12[14]
                        .low()
                        .as_m31(),
                    create_blake_round_input_output_tmp_8e0ec_12[14]
                        .high()
                        .as_m31(),
                    M31_52505,
                    M31_23520,
                    seq,
                ];
                *sub_component_inputs.blake_round[0] = (
                    seq,
                    M31_0,
                    (
                        [
                            create_blake_round_input_output_tmp_8e0ec_12[0],
                            create_blake_round_input_output_tmp_8e0ec_12[1],
                            create_blake_round_input_output_tmp_8e0ec_12[2],
                            create_blake_round_input_output_tmp_8e0ec_12[3],
                            create_blake_round_input_output_tmp_8e0ec_12[4],
                            create_blake_round_input_output_tmp_8e0ec_12[5],
                            create_blake_round_input_output_tmp_8e0ec_12[6],
                            create_blake_round_input_output_tmp_8e0ec_12[7],
                            UInt32_1779033703,
                            UInt32_3144134277,
                            UInt32_1013904242,
                            UInt32_2773480762,
                            create_blake_round_input_output_tmp_8e0ec_12[12],
                            UInt32_2600822924,
                            create_blake_round_input_output_tmp_8e0ec_12[14],
                            UInt32_1541459225,
                        ],
                        seq,
                    ),
                );
                let blake_round_output_round_0_tmp_8e0ec_31 = blake_round_state.deduce_output((
                    seq,
                    M31_0,
                    (
                        [
                            create_blake_round_input_output_tmp_8e0ec_12[0],
                            create_blake_round_input_output_tmp_8e0ec_12[1],
                            create_blake_round_input_output_tmp_8e0ec_12[2],
                            create_blake_round_input_output_tmp_8e0ec_12[3],
                            create_blake_round_input_output_tmp_8e0ec_12[4],
                            create_blake_round_input_output_tmp_8e0ec_12[5],
                            create_blake_round_input_output_tmp_8e0ec_12[6],
                            create_blake_round_input_output_tmp_8e0ec_12[7],
                            UInt32_1779033703,
                            UInt32_3144134277,
                            UInt32_1013904242,
                            UInt32_2773480762,
                            create_blake_round_input_output_tmp_8e0ec_12[12],
                            UInt32_2600822924,
                            create_blake_round_input_output_tmp_8e0ec_12[14],
                            UInt32_1541459225,
                        ],
                        seq,
                    ),
                ));
                *sub_component_inputs.blake_round[1] = (
                    seq,
                    M31_1,
                    (
                        [
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[0],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[1],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[2],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[3],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[4],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[5],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[6],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[7],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[8],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[9],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[10],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[11],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[12],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[13],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[14],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[15],
                        ],
                        blake_round_output_round_0_tmp_8e0ec_31.2 .1,
                    ),
                );
                let blake_round_output_round_1_tmp_8e0ec_32 = blake_round_state.deduce_output((
                    seq,
                    M31_1,
                    (
                        [
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[0],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[1],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[2],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[3],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[4],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[5],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[6],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[7],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[8],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[9],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[10],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[11],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[12],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[13],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[14],
                            blake_round_output_round_0_tmp_8e0ec_31.2 .0[15],
                        ],
                        blake_round_output_round_0_tmp_8e0ec_31.2 .1,
                    ),
                ));
                *sub_component_inputs.blake_round[2] = (
                    seq,
                    M31_2,
                    (
                        [
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[0],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[1],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[2],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[3],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[4],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[5],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[6],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[7],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[8],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[9],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[10],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[11],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[12],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[13],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[14],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[15],
                        ],
                        blake_round_output_round_1_tmp_8e0ec_32.2 .1,
                    ),
                );
                let blake_round_output_round_2_tmp_8e0ec_33 = blake_round_state.deduce_output((
                    seq,
                    M31_2,
                    (
                        [
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[0],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[1],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[2],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[3],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[4],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[5],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[6],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[7],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[8],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[9],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[10],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[11],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[12],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[13],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[14],
                            blake_round_output_round_1_tmp_8e0ec_32.2 .0[15],
                        ],
                        blake_round_output_round_1_tmp_8e0ec_32.2 .1,
                    ),
                ));
                *sub_component_inputs.blake_round[3] = (
                    seq,
                    M31_3,
                    (
                        [
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[0],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[1],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[2],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[3],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[4],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[5],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[6],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[7],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[8],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[9],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[10],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[11],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[12],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[13],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[14],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[15],
                        ],
                        blake_round_output_round_2_tmp_8e0ec_33.2 .1,
                    ),
                );
                let blake_round_output_round_3_tmp_8e0ec_34 = blake_round_state.deduce_output((
                    seq,
                    M31_3,
                    (
                        [
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[0],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[1],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[2],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[3],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[4],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[5],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[6],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[7],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[8],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[9],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[10],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[11],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[12],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[13],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[14],
                            blake_round_output_round_2_tmp_8e0ec_33.2 .0[15],
                        ],
                        blake_round_output_round_2_tmp_8e0ec_33.2 .1,
                    ),
                ));
                *sub_component_inputs.blake_round[4] = (
                    seq,
                    M31_4,
                    (
                        [
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[0],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[1],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[2],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[3],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[4],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[5],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[6],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[7],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[8],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[9],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[10],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[11],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[12],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[13],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[14],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[15],
                        ],
                        blake_round_output_round_3_tmp_8e0ec_34.2 .1,
                    ),
                );
                let blake_round_output_round_4_tmp_8e0ec_35 = blake_round_state.deduce_output((
                    seq,
                    M31_4,
                    (
                        [
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[0],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[1],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[2],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[3],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[4],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[5],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[6],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[7],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[8],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[9],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[10],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[11],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[12],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[13],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[14],
                            blake_round_output_round_3_tmp_8e0ec_34.2 .0[15],
                        ],
                        blake_round_output_round_3_tmp_8e0ec_34.2 .1,
                    ),
                ));
                *sub_component_inputs.blake_round[5] = (
                    seq,
                    M31_5,
                    (
                        [
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[0],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[1],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[2],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[3],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[4],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[5],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[6],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[7],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[8],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[9],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[10],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[11],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[12],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[13],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[14],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[15],
                        ],
                        blake_round_output_round_4_tmp_8e0ec_35.2 .1,
                    ),
                );
                let blake_round_output_round_5_tmp_8e0ec_36 = blake_round_state.deduce_output((
                    seq,
                    M31_5,
                    (
                        [
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[0],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[1],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[2],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[3],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[4],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[5],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[6],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[7],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[8],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[9],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[10],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[11],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[12],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[13],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[14],
                            blake_round_output_round_4_tmp_8e0ec_35.2 .0[15],
                        ],
                        blake_round_output_round_4_tmp_8e0ec_35.2 .1,
                    ),
                ));
                *sub_component_inputs.blake_round[6] = (
                    seq,
                    M31_6,
                    (
                        [
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[0],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[1],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[2],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[3],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[4],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[5],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[6],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[7],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[8],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[9],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[10],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[11],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[12],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[13],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[14],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[15],
                        ],
                        blake_round_output_round_5_tmp_8e0ec_36.2 .1,
                    ),
                );
                let blake_round_output_round_6_tmp_8e0ec_37 = blake_round_state.deduce_output((
                    seq,
                    M31_6,
                    (
                        [
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[0],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[1],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[2],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[3],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[4],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[5],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[6],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[7],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[8],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[9],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[10],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[11],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[12],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[13],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[14],
                            blake_round_output_round_5_tmp_8e0ec_36.2 .0[15],
                        ],
                        blake_round_output_round_5_tmp_8e0ec_36.2 .1,
                    ),
                ));
                *sub_component_inputs.blake_round[7] = (
                    seq,
                    M31_7,
                    (
                        [
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[0],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[1],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[2],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[3],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[4],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[5],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[6],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[7],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[8],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[9],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[10],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[11],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[12],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[13],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[14],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[15],
                        ],
                        blake_round_output_round_6_tmp_8e0ec_37.2 .1,
                    ),
                );
                let blake_round_output_round_7_tmp_8e0ec_38 = blake_round_state.deduce_output((
                    seq,
                    M31_7,
                    (
                        [
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[0],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[1],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[2],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[3],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[4],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[5],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[6],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[7],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[8],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[9],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[10],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[11],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[12],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[13],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[14],
                            blake_round_output_round_6_tmp_8e0ec_37.2 .0[15],
                        ],
                        blake_round_output_round_6_tmp_8e0ec_37.2 .1,
                    ),
                ));
                *sub_component_inputs.blake_round[8] = (
                    seq,
                    M31_8,
                    (
                        [
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[0],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[1],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[2],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[3],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[4],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[5],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[6],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[7],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[8],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[9],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[10],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[11],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[12],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[13],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[14],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[15],
                        ],
                        blake_round_output_round_7_tmp_8e0ec_38.2 .1,
                    ),
                );
                let blake_round_output_round_8_tmp_8e0ec_39 = blake_round_state.deduce_output((
                    seq,
                    M31_8,
                    (
                        [
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[0],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[1],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[2],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[3],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[4],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[5],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[6],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[7],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[8],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[9],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[10],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[11],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[12],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[13],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[14],
                            blake_round_output_round_7_tmp_8e0ec_38.2 .0[15],
                        ],
                        blake_round_output_round_7_tmp_8e0ec_38.2 .1,
                    ),
                ));
                *sub_component_inputs.blake_round[9] = (
                    seq,
                    M31_9,
                    (
                        [
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[0],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[1],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[2],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[3],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[4],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[5],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[6],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[7],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[8],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[9],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[10],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[11],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[12],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[13],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[14],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[15],
                        ],
                        blake_round_output_round_8_tmp_8e0ec_39.2 .1,
                    ),
                );
                let blake_round_output_round_9_tmp_8e0ec_40 = blake_round_state.deduce_output((
                    seq,
                    M31_9,
                    (
                        [
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[0],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[1],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[2],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[3],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[4],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[5],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[6],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[7],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[8],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[9],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[10],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[11],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[12],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[13],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[14],
                            blake_round_output_round_8_tmp_8e0ec_39.2 .0[15],
                        ],
                        blake_round_output_round_8_tmp_8e0ec_39.2 .1,
                    ),
                ));
                let blake_round_output_limb_0_col86 = blake_round_output_round_9_tmp_8e0ec_40.2 .0
                    [0]
                .low()
                .as_m31();
                *row[86] = blake_round_output_limb_0_col86;
                let blake_round_output_limb_1_col87 = blake_round_output_round_9_tmp_8e0ec_40.2 .0
                    [0]
                .high()
                .as_m31();
                *row[87] = blake_round_output_limb_1_col87;
                let blake_round_output_limb_2_col88 = blake_round_output_round_9_tmp_8e0ec_40.2 .0
                    [1]
                .low()
                .as_m31();
                *row[88] = blake_round_output_limb_2_col88;
                let blake_round_output_limb_3_col89 = blake_round_output_round_9_tmp_8e0ec_40.2 .0
                    [1]
                .high()
                .as_m31();
                *row[89] = blake_round_output_limb_3_col89;
                let blake_round_output_limb_4_col90 = blake_round_output_round_9_tmp_8e0ec_40.2 .0
                    [2]
                .low()
                .as_m31();
                *row[90] = blake_round_output_limb_4_col90;
                let blake_round_output_limb_5_col91 = blake_round_output_round_9_tmp_8e0ec_40.2 .0
                    [2]
                .high()
                .as_m31();
                *row[91] = blake_round_output_limb_5_col91;
                let blake_round_output_limb_6_col92 = blake_round_output_round_9_tmp_8e0ec_40.2 .0
                    [3]
                .low()
                .as_m31();
                *row[92] = blake_round_output_limb_6_col92;
                let blake_round_output_limb_7_col93 = blake_round_output_round_9_tmp_8e0ec_40.2 .0
                    [3]
                .high()
                .as_m31();
                *row[93] = blake_round_output_limb_7_col93;
                let blake_round_output_limb_8_col94 = blake_round_output_round_9_tmp_8e0ec_40.2 .0
                    [4]
                .low()
                .as_m31();
                *row[94] = blake_round_output_limb_8_col94;
                let blake_round_output_limb_9_col95 = blake_round_output_round_9_tmp_8e0ec_40.2 .0
                    [4]
                .high()
                .as_m31();
                *row[95] = blake_round_output_limb_9_col95;
                let blake_round_output_limb_10_col96 = blake_round_output_round_9_tmp_8e0ec_40.2 .0
                    [5]
                .low()
                .as_m31();
                *row[96] = blake_round_output_limb_10_col96;
                let blake_round_output_limb_11_col97 = blake_round_output_round_9_tmp_8e0ec_40.2 .0
                    [5]
                .high()
                .as_m31();
                *row[97] = blake_round_output_limb_11_col97;
                let blake_round_output_limb_12_col98 = blake_round_output_round_9_tmp_8e0ec_40.2 .0
                    [6]
                .low()
                .as_m31();
                *row[98] = blake_round_output_limb_12_col98;
                let blake_round_output_limb_13_col99 = blake_round_output_round_9_tmp_8e0ec_40.2 .0
                    [6]
                .high()
                .as_m31();
                *row[99] = blake_round_output_limb_13_col99;
                let blake_round_output_limb_14_col100 =
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[7]
                        .low()
                        .as_m31();
                *row[100] = blake_round_output_limb_14_col100;
                let blake_round_output_limb_15_col101 =
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[7]
                        .high()
                        .as_m31();
                *row[101] = blake_round_output_limb_15_col101;
                let blake_round_output_limb_16_col102 =
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[8]
                        .low()
                        .as_m31();
                *row[102] = blake_round_output_limb_16_col102;
                let blake_round_output_limb_17_col103 =
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[8]
                        .high()
                        .as_m31();
                *row[103] = blake_round_output_limb_17_col103;
                let blake_round_output_limb_18_col104 =
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[9]
                        .low()
                        .as_m31();
                *row[104] = blake_round_output_limb_18_col104;
                let blake_round_output_limb_19_col105 =
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[9]
                        .high()
                        .as_m31();
                *row[105] = blake_round_output_limb_19_col105;
                let blake_round_output_limb_20_col106 =
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[10]
                        .low()
                        .as_m31();
                *row[106] = blake_round_output_limb_20_col106;
                let blake_round_output_limb_21_col107 =
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[10]
                        .high()
                        .as_m31();
                *row[107] = blake_round_output_limb_21_col107;
                let blake_round_output_limb_22_col108 =
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[11]
                        .low()
                        .as_m31();
                *row[108] = blake_round_output_limb_22_col108;
                let blake_round_output_limb_23_col109 =
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[11]
                        .high()
                        .as_m31();
                *row[109] = blake_round_output_limb_23_col109;
                let blake_round_output_limb_24_col110 =
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[12]
                        .low()
                        .as_m31();
                *row[110] = blake_round_output_limb_24_col110;
                let blake_round_output_limb_25_col111 =
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[12]
                        .high()
                        .as_m31();
                *row[111] = blake_round_output_limb_25_col111;
                let blake_round_output_limb_26_col112 =
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[13]
                        .low()
                        .as_m31();
                *row[112] = blake_round_output_limb_26_col112;
                let blake_round_output_limb_27_col113 =
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[13]
                        .high()
                        .as_m31();
                *row[113] = blake_round_output_limb_27_col113;
                let blake_round_output_limb_28_col114 =
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[14]
                        .low()
                        .as_m31();
                *row[114] = blake_round_output_limb_28_col114;
                let blake_round_output_limb_29_col115 =
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[14]
                        .high()
                        .as_m31();
                *row[115] = blake_round_output_limb_29_col115;
                let blake_round_output_limb_30_col116 =
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[15]
                        .low()
                        .as_m31();
                *row[116] = blake_round_output_limb_30_col116;
                let blake_round_output_limb_31_col117 =
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[15]
                        .high()
                        .as_m31();
                *row[117] = blake_round_output_limb_31_col117;
                let blake_round_output_limb_32_col118 =
                    blake_round_output_round_9_tmp_8e0ec_40.2 .1;
                *row[118] = blake_round_output_limb_32_col118;
                *lookup_data.blake_round_1 = [
                    M31_40528774,
                    seq,
                    M31_10,
                    blake_round_output_limb_0_col86,
                    blake_round_output_limb_1_col87,
                    blake_round_output_limb_2_col88,
                    blake_round_output_limb_3_col89,
                    blake_round_output_limb_4_col90,
                    blake_round_output_limb_5_col91,
                    blake_round_output_limb_6_col92,
                    blake_round_output_limb_7_col93,
                    blake_round_output_limb_8_col94,
                    blake_round_output_limb_9_col95,
                    blake_round_output_limb_10_col96,
                    blake_round_output_limb_11_col97,
                    blake_round_output_limb_12_col98,
                    blake_round_output_limb_13_col99,
                    blake_round_output_limb_14_col100,
                    blake_round_output_limb_15_col101,
                    blake_round_output_limb_16_col102,
                    blake_round_output_limb_17_col103,
                    blake_round_output_limb_18_col104,
                    blake_round_output_limb_19_col105,
                    blake_round_output_limb_20_col106,
                    blake_round_output_limb_21_col107,
                    blake_round_output_limb_22_col108,
                    blake_round_output_limb_23_col109,
                    blake_round_output_limb_24_col110,
                    blake_round_output_limb_25_col111,
                    blake_round_output_limb_26_col112,
                    blake_round_output_limb_27_col113,
                    blake_round_output_limb_28_col114,
                    blake_round_output_limb_29_col115,
                    blake_round_output_limb_30_col116,
                    blake_round_output_limb_31_col117,
                    blake_round_output_limb_32_col118,
                ];

                // Create Blake Output.

                *sub_component_inputs.triple_xor_32[0] = [
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[0],
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[8],
                    blake_gate_input.0[0][0],
                ];
                let triple_xor_32_output_tmp_8e0ec_41 = PackedTripleXor32::deduce_output([
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[0],
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[8],
                    blake_gate_input.0[0][0],
                ]);
                let triple_xor_32_output_limb_0_col119 =
                    triple_xor_32_output_tmp_8e0ec_41.low().as_m31();
                *row[119] = triple_xor_32_output_limb_0_col119;
                let triple_xor_32_output_limb_1_col120 =
                    triple_xor_32_output_tmp_8e0ec_41.high().as_m31();
                *row[120] = triple_xor_32_output_limb_1_col120;
                *lookup_data.triple_xor_32_0 = [
                    M31_990559919,
                    blake_round_output_limb_0_col86,
                    blake_round_output_limb_1_col87,
                    blake_round_output_limb_16_col102,
                    blake_round_output_limb_17_col103,
                    input_state_before_limb0_limb_0_col0,
                    input_state_before_limb0_limb_1_col1,
                    triple_xor_32_output_limb_0_col119,
                    triple_xor_32_output_limb_1_col120,
                ];
                *sub_component_inputs.triple_xor_32[1] = [
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[1],
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[9],
                    blake_gate_input.0[0][1],
                ];
                let triple_xor_32_output_tmp_8e0ec_42 = PackedTripleXor32::deduce_output([
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[1],
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[9],
                    blake_gate_input.0[0][1],
                ]);
                let triple_xor_32_output_limb_0_col121 =
                    triple_xor_32_output_tmp_8e0ec_42.low().as_m31();
                *row[121] = triple_xor_32_output_limb_0_col121;
                let triple_xor_32_output_limb_1_col122 =
                    triple_xor_32_output_tmp_8e0ec_42.high().as_m31();
                *row[122] = triple_xor_32_output_limb_1_col122;
                *lookup_data.triple_xor_32_1 = [
                    M31_990559919,
                    blake_round_output_limb_2_col88,
                    blake_round_output_limb_3_col89,
                    blake_round_output_limb_18_col104,
                    blake_round_output_limb_19_col105,
                    input_state_before_limb1_limb_0_col2,
                    input_state_before_limb1_limb_1_col3,
                    triple_xor_32_output_limb_0_col121,
                    triple_xor_32_output_limb_1_col122,
                ];
                *sub_component_inputs.triple_xor_32[2] = [
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[2],
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[10],
                    blake_gate_input.0[0][2],
                ];
                let triple_xor_32_output_tmp_8e0ec_43 = PackedTripleXor32::deduce_output([
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[2],
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[10],
                    blake_gate_input.0[0][2],
                ]);
                let triple_xor_32_output_limb_0_col123 =
                    triple_xor_32_output_tmp_8e0ec_43.low().as_m31();
                *row[123] = triple_xor_32_output_limb_0_col123;
                let triple_xor_32_output_limb_1_col124 =
                    triple_xor_32_output_tmp_8e0ec_43.high().as_m31();
                *row[124] = triple_xor_32_output_limb_1_col124;
                *lookup_data.triple_xor_32_2 = [
                    M31_990559919,
                    blake_round_output_limb_4_col90,
                    blake_round_output_limb_5_col91,
                    blake_round_output_limb_20_col106,
                    blake_round_output_limb_21_col107,
                    input_state_before_limb2_limb_0_col4,
                    input_state_before_limb2_limb_1_col5,
                    triple_xor_32_output_limb_0_col123,
                    triple_xor_32_output_limb_1_col124,
                ];
                *sub_component_inputs.triple_xor_32[3] = [
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[3],
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[11],
                    blake_gate_input.0[0][3],
                ];
                let triple_xor_32_output_tmp_8e0ec_44 = PackedTripleXor32::deduce_output([
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[3],
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[11],
                    blake_gate_input.0[0][3],
                ]);
                let triple_xor_32_output_limb_0_col125 =
                    triple_xor_32_output_tmp_8e0ec_44.low().as_m31();
                *row[125] = triple_xor_32_output_limb_0_col125;
                let triple_xor_32_output_limb_1_col126 =
                    triple_xor_32_output_tmp_8e0ec_44.high().as_m31();
                *row[126] = triple_xor_32_output_limb_1_col126;
                *lookup_data.triple_xor_32_3 = [
                    M31_990559919,
                    blake_round_output_limb_6_col92,
                    blake_round_output_limb_7_col93,
                    blake_round_output_limb_22_col108,
                    blake_round_output_limb_23_col109,
                    input_state_before_limb3_limb_0_col6,
                    input_state_before_limb3_limb_1_col7,
                    triple_xor_32_output_limb_0_col125,
                    triple_xor_32_output_limb_1_col126,
                ];
                *sub_component_inputs.triple_xor_32[4] = [
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[4],
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[12],
                    blake_gate_input.0[0][4],
                ];
                let triple_xor_32_output_tmp_8e0ec_45 = PackedTripleXor32::deduce_output([
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[4],
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[12],
                    blake_gate_input.0[0][4],
                ]);
                let triple_xor_32_output_limb_0_col127 =
                    triple_xor_32_output_tmp_8e0ec_45.low().as_m31();
                *row[127] = triple_xor_32_output_limb_0_col127;
                let triple_xor_32_output_limb_1_col128 =
                    triple_xor_32_output_tmp_8e0ec_45.high().as_m31();
                *row[128] = triple_xor_32_output_limb_1_col128;
                *lookup_data.triple_xor_32_4 = [
                    M31_990559919,
                    blake_round_output_limb_8_col94,
                    blake_round_output_limb_9_col95,
                    blake_round_output_limb_24_col110,
                    blake_round_output_limb_25_col111,
                    input_state_before_limb4_limb_0_col8,
                    input_state_before_limb4_limb_1_col9,
                    triple_xor_32_output_limb_0_col127,
                    triple_xor_32_output_limb_1_col128,
                ];
                *sub_component_inputs.triple_xor_32[5] = [
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[5],
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[13],
                    blake_gate_input.0[0][5],
                ];
                let triple_xor_32_output_tmp_8e0ec_46 = PackedTripleXor32::deduce_output([
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[5],
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[13],
                    blake_gate_input.0[0][5],
                ]);
                let triple_xor_32_output_limb_0_col129 =
                    triple_xor_32_output_tmp_8e0ec_46.low().as_m31();
                *row[129] = triple_xor_32_output_limb_0_col129;
                let triple_xor_32_output_limb_1_col130 =
                    triple_xor_32_output_tmp_8e0ec_46.high().as_m31();
                *row[130] = triple_xor_32_output_limb_1_col130;
                *lookup_data.triple_xor_32_5 = [
                    M31_990559919,
                    blake_round_output_limb_10_col96,
                    blake_round_output_limb_11_col97,
                    blake_round_output_limb_26_col112,
                    blake_round_output_limb_27_col113,
                    input_state_before_limb5_limb_0_col10,
                    input_state_before_limb5_limb_1_col11,
                    triple_xor_32_output_limb_0_col129,
                    triple_xor_32_output_limb_1_col130,
                ];
                *sub_component_inputs.triple_xor_32[6] = [
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[6],
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[14],
                    blake_gate_input.0[0][6],
                ];
                let triple_xor_32_output_tmp_8e0ec_47 = PackedTripleXor32::deduce_output([
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[6],
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[14],
                    blake_gate_input.0[0][6],
                ]);
                let triple_xor_32_output_limb_0_col131 =
                    triple_xor_32_output_tmp_8e0ec_47.low().as_m31();
                *row[131] = triple_xor_32_output_limb_0_col131;
                let triple_xor_32_output_limb_1_col132 =
                    triple_xor_32_output_tmp_8e0ec_47.high().as_m31();
                *row[132] = triple_xor_32_output_limb_1_col132;
                *lookup_data.triple_xor_32_6 = [
                    M31_990559919,
                    blake_round_output_limb_12_col98,
                    blake_round_output_limb_13_col99,
                    blake_round_output_limb_28_col114,
                    blake_round_output_limb_29_col115,
                    input_state_before_limb6_limb_0_col12,
                    input_state_before_limb6_limb_1_col13,
                    triple_xor_32_output_limb_0_col131,
                    triple_xor_32_output_limb_1_col132,
                ];
                *sub_component_inputs.triple_xor_32[7] = [
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[7],
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[15],
                    blake_gate_input.0[0][7],
                ];
                let triple_xor_32_output_tmp_8e0ec_48 = PackedTripleXor32::deduce_output([
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[7],
                    blake_round_output_round_9_tmp_8e0ec_40.2 .0[15],
                    blake_gate_input.0[0][7],
                ]);
                let triple_xor_32_output_limb_0_col133 =
                    triple_xor_32_output_tmp_8e0ec_48.low().as_m31();
                *row[133] = triple_xor_32_output_limb_0_col133;
                let triple_xor_32_output_limb_1_col134 =
                    triple_xor_32_output_tmp_8e0ec_48.high().as_m31();
                *row[134] = triple_xor_32_output_limb_1_col134;
                *lookup_data.triple_xor_32_7 = [
                    M31_990559919,
                    blake_round_output_limb_14_col100,
                    blake_round_output_limb_15_col101,
                    blake_round_output_limb_30_col116,
                    blake_round_output_limb_31_col117,
                    input_state_before_limb7_limb_0_col14,
                    input_state_before_limb7_limb_1_col15,
                    triple_xor_32_output_limb_0_col133,
                    triple_xor_32_output_limb_1_col134,
                ];
                let create_blake_output_output_tmp_8e0ec_49 = [
                    triple_xor_32_output_tmp_8e0ec_41,
                    triple_xor_32_output_tmp_8e0ec_42,
                    triple_xor_32_output_tmp_8e0ec_43,
                    triple_xor_32_output_tmp_8e0ec_44,
                    triple_xor_32_output_tmp_8e0ec_45,
                    triple_xor_32_output_tmp_8e0ec_46,
                    triple_xor_32_output_tmp_8e0ec_47,
                    triple_xor_32_output_tmp_8e0ec_48,
                ];

                *lookup_data.blake_output_0 = [
                    M31_1061955672,
                    state_before_addr,
                    input_state_before_limb0_limb_0_col0,
                    input_state_before_limb0_limb_1_col1,
                    input_state_before_limb1_limb_0_col2,
                    input_state_before_limb1_limb_1_col3,
                    input_state_before_limb2_limb_0_col4,
                    input_state_before_limb2_limb_1_col5,
                    input_state_before_limb3_limb_0_col6,
                    input_state_before_limb3_limb_1_col7,
                    input_state_before_limb4_limb_0_col8,
                    input_state_before_limb4_limb_1_col9,
                    input_state_before_limb5_limb_0_col10,
                    input_state_before_limb5_limb_1_col11,
                    input_state_before_limb6_limb_0_col12,
                    input_state_before_limb6_limb_1_col13,
                    input_state_before_limb7_limb_0_col14,
                    input_state_before_limb7_limb_1_col15,
                ];
                *lookup_data.blake_output_1 = [
                    M31_1061955672,
                    state_after_addr,
                    input_state_after_limb0_limb_0_col16,
                    input_state_after_limb0_limb_1_col17,
                    input_state_after_limb1_limb_0_col18,
                    input_state_after_limb1_limb_1_col19,
                    input_state_after_limb2_limb_0_col20,
                    input_state_after_limb2_limb_1_col21,
                    input_state_after_limb3_limb_0_col22,
                    input_state_after_limb3_limb_1_col23,
                    input_state_after_limb4_limb_0_col24,
                    input_state_after_limb4_limb_1_col25,
                    input_state_after_limb5_limb_0_col26,
                    input_state_after_limb5_limb_1_col27,
                    input_state_after_limb6_limb_0_col28,
                    input_state_after_limb6_limb_1_col29,
                    input_state_after_limb7_limb_0_col30,
                    input_state_after_limb7_limb_1_col31,
                ];
                *lookup_data.gate_0 = [
                    M31_378353459,
                    message0_addr,
                    input_message_limb0_col32,
                    input_message_limb1_col33,
                    input_message_limb2_col34,
                    input_message_limb3_col35,
                ];
                *lookup_data.gate_1 = [
                    M31_378353459,
                    message1_addr,
                    input_message_limb4_col36,
                    input_message_limb5_col37,
                    input_message_limb6_col38,
                    input_message_limb7_col39,
                ];
                *lookup_data.gate_2 = [
                    M31_378353459,
                    message2_addr,
                    input_message_limb8_col40,
                    input_message_limb9_col41,
                    input_message_limb10_col42,
                    input_message_limb11_col43,
                ];
                *lookup_data.gate_3 = [
                    M31_378353459,
                    message3_addr,
                    input_message_limb12_col44,
                    input_message_limb13_col45,
                    input_message_limb14_col46,
                    input_message_limb15_col47,
                ];
                let mult_at_row = *mults.get(row_index).unwrap_or(&PackedM31::zero());
                *lookup_data.mults = mult_at_row;
            },
        );

    (trace, lookup_data, sub_component_inputs)
}

#[derive(Uninitialized, IterMut, ParIterMut)]
struct LookupData {
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
    blake_output_0: Vec<[PackedM31; 18]>,
    blake_output_1: Vec<[PackedM31; 18]>,
    blake_round_0: Vec<[PackedM31; 36]>,
    blake_round_1: Vec<[PackedM31; 36]>,
    gate_0: Vec<[PackedM31; 6]>,
    gate_1: Vec<[PackedM31; 6]>,
    gate_2: Vec<[PackedM31; 6]>,
    gate_3: Vec<[PackedM31; 6]>,
    range_check_15_0: Vec<[PackedM31; 2]>,
    range_check_15_1: Vec<[PackedM31; 2]>,
    range_check_15_2: Vec<[PackedM31; 2]>,
    range_check_15_3: Vec<[PackedM31; 2]>,
    range_check_15_4: Vec<[PackedM31; 2]>,
    range_check_15_5: Vec<[PackedM31; 2]>,
    range_check_15_6: Vec<[PackedM31; 2]>,
    range_check_15_7: Vec<[PackedM31; 2]>,
    range_check_15_8: Vec<[PackedM31; 2]>,
    range_check_15_9: Vec<[PackedM31; 2]>,
    range_check_15_10: Vec<[PackedM31; 2]>,
    range_check_15_11: Vec<[PackedM31; 2]>,
    range_check_15_12: Vec<[PackedM31; 2]>,
    range_check_15_13: Vec<[PackedM31; 2]>,
    range_check_15_14: Vec<[PackedM31; 2]>,
    range_check_15_15: Vec<[PackedM31; 2]>,
    range_check_16_0: Vec<[PackedM31; 2]>,
    range_check_16_1: Vec<[PackedM31; 2]>,
    range_check_16_2: Vec<[PackedM31; 2]>,
    range_check_16_3: Vec<[PackedM31; 2]>,
    range_check_16_4: Vec<[PackedM31; 2]>,
    range_check_16_5: Vec<[PackedM31; 2]>,
    range_check_16_6: Vec<[PackedM31; 2]>,
    range_check_16_7: Vec<[PackedM31; 2]>,
    range_check_16_8: Vec<[PackedM31; 2]>,
    range_check_16_9: Vec<[PackedM31; 2]>,
    range_check_16_10: Vec<[PackedM31; 2]>,
    range_check_16_11: Vec<[PackedM31; 2]>,
    range_check_16_12: Vec<[PackedM31; 2]>,
    range_check_16_13: Vec<[PackedM31; 2]>,
    range_check_16_14: Vec<[PackedM31; 2]>,
    range_check_16_15: Vec<[PackedM31; 2]>,
    triple_xor_32_0: Vec<[PackedM31; 9]>,
    triple_xor_32_1: Vec<[PackedM31; 9]>,
    triple_xor_32_2: Vec<[PackedM31; 9]>,
    triple_xor_32_3: Vec<[PackedM31; 9]>,
    triple_xor_32_4: Vec<[PackedM31; 9]>,
    triple_xor_32_5: Vec<[PackedM31; 9]>,
    triple_xor_32_6: Vec<[PackedM31; 9]>,
    triple_xor_32_7: Vec<[PackedM31; 9]>,
    verify_bitwise_xor_8_0: Vec<[PackedM31; 4]>,
    verify_bitwise_xor_8_1: Vec<[PackedM31; 4]>,
    verify_bitwise_xor_8_2: Vec<[PackedM31; 4]>,
    verify_bitwise_xor_8_3: Vec<[PackedM31; 4]>,
    mults_0: Vec<PackedM31>,
}

pub struct InteractionClaimGenerator {
    log_size: u32,
    lookup_data: LookupData,
}
impl InteractionClaimGenerator {
    pub fn write_interaction_trace(
        self,
        tree_builder: &mut impl TreeBuilder<SimdBackend>,
        common_lookup_elements: &relations::CommonLookupElements,
    ) -> InteractionClaim {
        let mut logup_gen = LogupTraceGenerator::new(self.log_size);

        // Sum logup terms in pairs.
        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.verify_bitwise_xor_8_0,
            &self.lookup_data.verify_bitwise_xor_8_1,
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
            &self.lookup_data.verify_bitwise_xor_8_2,
            &self.lookup_data.verify_bitwise_xor_8_3,
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
            &self.lookup_data.range_check_16_0,
            &self.lookup_data.range_check_15_0,
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
            &self.lookup_data.blake_message_0,
            &self.lookup_data.range_check_16_1,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 - denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.range_check_15_1,
            &self.lookup_data.blake_message_1,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom1 - denom0, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.range_check_16_2,
            &self.lookup_data.range_check_15_2,
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
            &self.lookup_data.blake_message_2,
            &self.lookup_data.range_check_16_3,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 - denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.range_check_15_3,
            &self.lookup_data.blake_message_3,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom1 - denom0, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.range_check_16_4,
            &self.lookup_data.range_check_15_4,
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
            &self.lookup_data.blake_message_4,
            &self.lookup_data.range_check_16_5,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 - denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.range_check_15_5,
            &self.lookup_data.blake_message_5,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom1 - denom0, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.range_check_16_6,
            &self.lookup_data.range_check_15_6,
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
            &self.lookup_data.blake_message_6,
            &self.lookup_data.range_check_16_7,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 - denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.range_check_15_7,
            &self.lookup_data.blake_message_7,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom1 - denom0, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.range_check_16_8,
            &self.lookup_data.range_check_15_8,
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
            &self.lookup_data.blake_message_8,
            &self.lookup_data.range_check_16_9,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 - denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.range_check_15_9,
            &self.lookup_data.blake_message_9,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom1 - denom0, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.range_check_16_10,
            &self.lookup_data.range_check_15_10,
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
            &self.lookup_data.blake_message_10,
            &self.lookup_data.range_check_16_11,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 - denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.range_check_15_11,
            &self.lookup_data.blake_message_11,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom1 - denom0, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.range_check_16_12,
            &self.lookup_data.range_check_15_12,
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
            &self.lookup_data.blake_message_12,
            &self.lookup_data.range_check_16_13,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 - denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.range_check_15_13,
            &self.lookup_data.blake_message_13,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom1 - denom0, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.range_check_16_14,
            &self.lookup_data.range_check_15_14,
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
            &self.lookup_data.blake_message_14,
            &self.lookup_data.range_check_16_15,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 - denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.range_check_15_15,
            &self.lookup_data.blake_message_15,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom1 - denom0, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.blake_round_0,
            &self.lookup_data.blake_round_1,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 - denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.triple_xor_32_0,
            &self.lookup_data.triple_xor_32_1,
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
            &self.lookup_data.triple_xor_32_2,
            &self.lookup_data.triple_xor_32_3,
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
            &self.lookup_data.triple_xor_32_4,
            &self.lookup_data.triple_xor_32_5,
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
            &self.lookup_data.triple_xor_32_6,
            &self.lookup_data.triple_xor_32_7,
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
            &self.lookup_data.blake_output_0,
            &self.lookup_data.blake_output_1,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom1 - denom0, denom0 * denom1);
            });
        col_gen.finalize_col();

        let mut col_gen = logup_gen.new_col();
        (
            col_gen.par_iter_mut(),
            &self.lookup_data.gate_0,
            &self.lookup_data.gate_1,
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
            &self.lookup_data.gate_2,
            &self.lookup_data.gate_3,
        )
            .into_par_iter()
            .for_each(|(writer, values0, values1)| {
                let denom0: PackedQM31 = common_lookup_elements.combine(values0);
                let denom1: PackedQM31 = common_lookup_elements.combine(values1);
                writer.write_frac(denom0 + denom1, denom0 * denom1);
            });
        col_gen.finalize_col();

        let (trace, claimed_sum) = logup_gen.finalize_last();
        tree_builder.extend_evals(trace);

        InteractionClaim { claimed_sum }
    }
}
