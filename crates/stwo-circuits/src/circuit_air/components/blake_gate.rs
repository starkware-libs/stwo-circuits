// This file was created by the AIR team.

use crate::circuit_air::components::prelude::*;
use crate::circuit_air::components::subroutines::create_blake_output::CreateBlakeOutput;
use crate::circuit_air::components::subroutines::create_blake_round_input::CreateBlakeRoundInput;
use crate::circuit_air::components::subroutines::qm_31_into_u_32::Qm31IntoU32;

pub const N_TRACE_COLUMNS: usize = 136;
pub const RELATION_USES_PER_ROW: [RelationUse; 7] = [
    RelationUse {
        relation_id: "BlakeOutput",
        uses: 1,
    },
    RelationUse {
        relation_id: "BlakeRound",
        uses: 1,
    },
    RelationUse {
        relation_id: "Gate",
        uses: 4,
    },
    RelationUse {
        relation_id: "RangeCheck_15",
        uses: 16,
    },
    RelationUse {
        relation_id: "RangeCheck_16",
        uses: 16,
    },
    RelationUse {
        relation_id: "TripleXor32",
        uses: 8,
    },
    RelationUse {
        relation_id: "VerifyBitwiseXor_8",
        uses: 4,
    },
];

pub struct Eval {
    pub claim: Claim,
    pub common_lookup_elements: relations::CommonLookupElements,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub log_size: u32,
}
impl Claim {
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let trace_log_sizes = vec![self.log_size; N_TRACE_COLUMNS];
        let interaction_log_sizes = vec![self.log_size; SECURE_EXTENSION_DEGREE * 34];
        TreeVec::new(vec![vec![], trace_log_sizes, interaction_log_sizes])
    }

    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_u64(self.log_size as u64);
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct InteractionClaim {
    pub claimed_sum: SecureField,
}
impl InteractionClaim {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_felts(&[self.claimed_sum]);
    }
}

pub type Component = FrameworkComponent<Eval>;

impl FrameworkEval for Eval {
    fn log_size(&self) -> u32 {
        self.claim.log_size
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        self.log_size() + 1
    }

    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        let M31_0 = E::F::from(M31::from(0));
        let M31_10 = E::F::from(M31::from(10));
        let M31_1061955672 = E::F::from(M31::from(1061955672));
        let M31_15470 = E::F::from(M31::from(15470));
        let M31_23520 = E::F::from(M31::from(23520));
        let M31_26764 = E::F::from(M31::from(26764));
        let M31_27145 = E::F::from(M31::from(27145));
        let M31_378353459 = E::F::from(M31::from(378353459));
        let M31_39685 = E::F::from(M31::from(39685));
        let M31_40528774 = E::F::from(M31::from(40528774));
        let M31_42319 = E::F::from(M31::from(42319));
        let M31_44677 = E::F::from(M31::from(44677));
        let M31_47975 = E::F::from(M31::from(47975));
        let M31_52505 = E::F::from(M31::from(52505));
        let M31_58983 = E::F::from(M31::from(58983));
        let M31_62322 = E::F::from(M31::from(62322));
        let M31_62778 = E::F::from(M31::from(62778));
        let t0 = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "t0".to_owned(),
        });
        let t1 = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "t1".to_owned(),
        });
        let finalize_flag = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "finalize_flag".to_owned(),
        });
        let seq = eval.get_preprocessed_column(Seq::new(self.log_size()).id());
        let state_before_addr = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "state_before_addr".to_owned(),
        });
        let state_after_addr = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "state_after_addr".to_owned(),
        });
        let message0_addr = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "message0_addr".to_owned(),
        });
        let message1_addr = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "message1_addr".to_owned(),
        });
        let message2_addr = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "message2_addr".to_owned(),
        });
        let message3_addr = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "message3_addr".to_owned(),
        });
        let input_state_before_limb0_limb_0_col0 = eval.next_trace_mask();
        let input_state_before_limb0_limb_1_col1 = eval.next_trace_mask();
        let input_state_before_limb1_limb_0_col2 = eval.next_trace_mask();
        let input_state_before_limb1_limb_1_col3 = eval.next_trace_mask();
        let input_state_before_limb2_limb_0_col4 = eval.next_trace_mask();
        let input_state_before_limb2_limb_1_col5 = eval.next_trace_mask();
        let input_state_before_limb3_limb_0_col6 = eval.next_trace_mask();
        let input_state_before_limb3_limb_1_col7 = eval.next_trace_mask();
        let input_state_before_limb4_limb_0_col8 = eval.next_trace_mask();
        let input_state_before_limb4_limb_1_col9 = eval.next_trace_mask();
        let input_state_before_limb5_limb_0_col10 = eval.next_trace_mask();
        let input_state_before_limb5_limb_1_col11 = eval.next_trace_mask();
        let input_state_before_limb6_limb_0_col12 = eval.next_trace_mask();
        let input_state_before_limb6_limb_1_col13 = eval.next_trace_mask();
        let input_state_before_limb7_limb_0_col14 = eval.next_trace_mask();
        let input_state_before_limb7_limb_1_col15 = eval.next_trace_mask();
        let input_state_after_limb0_limb_0_col16 = eval.next_trace_mask();
        let input_state_after_limb0_limb_1_col17 = eval.next_trace_mask();
        let input_state_after_limb1_limb_0_col18 = eval.next_trace_mask();
        let input_state_after_limb1_limb_1_col19 = eval.next_trace_mask();
        let input_state_after_limb2_limb_0_col20 = eval.next_trace_mask();
        let input_state_after_limb2_limb_1_col21 = eval.next_trace_mask();
        let input_state_after_limb3_limb_0_col22 = eval.next_trace_mask();
        let input_state_after_limb3_limb_1_col23 = eval.next_trace_mask();
        let input_state_after_limb4_limb_0_col24 = eval.next_trace_mask();
        let input_state_after_limb4_limb_1_col25 = eval.next_trace_mask();
        let input_state_after_limb5_limb_0_col26 = eval.next_trace_mask();
        let input_state_after_limb5_limb_1_col27 = eval.next_trace_mask();
        let input_state_after_limb6_limb_0_col28 = eval.next_trace_mask();
        let input_state_after_limb6_limb_1_col29 = eval.next_trace_mask();
        let input_state_after_limb7_limb_0_col30 = eval.next_trace_mask();
        let input_state_after_limb7_limb_1_col31 = eval.next_trace_mask();
        let input_message_limb0_col32 = eval.next_trace_mask();
        let input_message_limb1_col33 = eval.next_trace_mask();
        let input_message_limb2_col34 = eval.next_trace_mask();
        let input_message_limb3_col35 = eval.next_trace_mask();
        let input_message_limb4_col36 = eval.next_trace_mask();
        let input_message_limb5_col37 = eval.next_trace_mask();
        let input_message_limb6_col38 = eval.next_trace_mask();
        let input_message_limb7_col39 = eval.next_trace_mask();
        let input_message_limb8_col40 = eval.next_trace_mask();
        let input_message_limb9_col41 = eval.next_trace_mask();
        let input_message_limb10_col42 = eval.next_trace_mask();
        let input_message_limb11_col43 = eval.next_trace_mask();
        let input_message_limb12_col44 = eval.next_trace_mask();
        let input_message_limb13_col45 = eval.next_trace_mask();
        let input_message_limb14_col46 = eval.next_trace_mask();
        let input_message_limb15_col47 = eval.next_trace_mask();
        let ms_8_bits_col48 = eval.next_trace_mask();
        let ms_8_bits_col49 = eval.next_trace_mask();
        let xor_col50 = eval.next_trace_mask();
        let xor_col51 = eval.next_trace_mask();
        let xor_col52 = eval.next_trace_mask();
        let xor_col53 = eval.next_trace_mask();
        let limbi_low_col54 = eval.next_trace_mask();
        let limbi_high_col55 = eval.next_trace_mask();
        let limbi_low_col56 = eval.next_trace_mask();
        let limbi_high_col57 = eval.next_trace_mask();
        let limbi_low_col58 = eval.next_trace_mask();
        let limbi_high_col59 = eval.next_trace_mask();
        let limbi_low_col60 = eval.next_trace_mask();
        let limbi_high_col61 = eval.next_trace_mask();
        let limbi_low_col62 = eval.next_trace_mask();
        let limbi_high_col63 = eval.next_trace_mask();
        let limbi_low_col64 = eval.next_trace_mask();
        let limbi_high_col65 = eval.next_trace_mask();
        let limbi_low_col66 = eval.next_trace_mask();
        let limbi_high_col67 = eval.next_trace_mask();
        let limbi_low_col68 = eval.next_trace_mask();
        let limbi_high_col69 = eval.next_trace_mask();
        let limbi_low_col70 = eval.next_trace_mask();
        let limbi_high_col71 = eval.next_trace_mask();
        let limbi_low_col72 = eval.next_trace_mask();
        let limbi_high_col73 = eval.next_trace_mask();
        let limbi_low_col74 = eval.next_trace_mask();
        let limbi_high_col75 = eval.next_trace_mask();
        let limbi_low_col76 = eval.next_trace_mask();
        let limbi_high_col77 = eval.next_trace_mask();
        let limbi_low_col78 = eval.next_trace_mask();
        let limbi_high_col79 = eval.next_trace_mask();
        let limbi_low_col80 = eval.next_trace_mask();
        let limbi_high_col81 = eval.next_trace_mask();
        let limbi_low_col82 = eval.next_trace_mask();
        let limbi_high_col83 = eval.next_trace_mask();
        let limbi_low_col84 = eval.next_trace_mask();
        let limbi_high_col85 = eval.next_trace_mask();
        let blake_round_output_limb_0_col86 = eval.next_trace_mask();
        let blake_round_output_limb_1_col87 = eval.next_trace_mask();
        let blake_round_output_limb_2_col88 = eval.next_trace_mask();
        let blake_round_output_limb_3_col89 = eval.next_trace_mask();
        let blake_round_output_limb_4_col90 = eval.next_trace_mask();
        let blake_round_output_limb_5_col91 = eval.next_trace_mask();
        let blake_round_output_limb_6_col92 = eval.next_trace_mask();
        let blake_round_output_limb_7_col93 = eval.next_trace_mask();
        let blake_round_output_limb_8_col94 = eval.next_trace_mask();
        let blake_round_output_limb_9_col95 = eval.next_trace_mask();
        let blake_round_output_limb_10_col96 = eval.next_trace_mask();
        let blake_round_output_limb_11_col97 = eval.next_trace_mask();
        let blake_round_output_limb_12_col98 = eval.next_trace_mask();
        let blake_round_output_limb_13_col99 = eval.next_trace_mask();
        let blake_round_output_limb_14_col100 = eval.next_trace_mask();
        let blake_round_output_limb_15_col101 = eval.next_trace_mask();
        let blake_round_output_limb_16_col102 = eval.next_trace_mask();
        let blake_round_output_limb_17_col103 = eval.next_trace_mask();
        let blake_round_output_limb_18_col104 = eval.next_trace_mask();
        let blake_round_output_limb_19_col105 = eval.next_trace_mask();
        let blake_round_output_limb_20_col106 = eval.next_trace_mask();
        let blake_round_output_limb_21_col107 = eval.next_trace_mask();
        let blake_round_output_limb_22_col108 = eval.next_trace_mask();
        let blake_round_output_limb_23_col109 = eval.next_trace_mask();
        let blake_round_output_limb_24_col110 = eval.next_trace_mask();
        let blake_round_output_limb_25_col111 = eval.next_trace_mask();
        let blake_round_output_limb_26_col112 = eval.next_trace_mask();
        let blake_round_output_limb_27_col113 = eval.next_trace_mask();
        let blake_round_output_limb_28_col114 = eval.next_trace_mask();
        let blake_round_output_limb_29_col115 = eval.next_trace_mask();
        let blake_round_output_limb_30_col116 = eval.next_trace_mask();
        let blake_round_output_limb_31_col117 = eval.next_trace_mask();
        let blake_round_output_limb_32_col118 = eval.next_trace_mask();
        let triple_xor_32_output_limb_0_col119 = eval.next_trace_mask();
        let triple_xor_32_output_limb_1_col120 = eval.next_trace_mask();
        let triple_xor_32_output_limb_0_col121 = eval.next_trace_mask();
        let triple_xor_32_output_limb_1_col122 = eval.next_trace_mask();
        let triple_xor_32_output_limb_0_col123 = eval.next_trace_mask();
        let triple_xor_32_output_limb_1_col124 = eval.next_trace_mask();
        let triple_xor_32_output_limb_0_col125 = eval.next_trace_mask();
        let triple_xor_32_output_limb_1_col126 = eval.next_trace_mask();
        let triple_xor_32_output_limb_0_col127 = eval.next_trace_mask();
        let triple_xor_32_output_limb_1_col128 = eval.next_trace_mask();
        let triple_xor_32_output_limb_0_col129 = eval.next_trace_mask();
        let triple_xor_32_output_limb_1_col130 = eval.next_trace_mask();
        let triple_xor_32_output_limb_0_col131 = eval.next_trace_mask();
        let triple_xor_32_output_limb_1_col132 = eval.next_trace_mask();
        let triple_xor_32_output_limb_0_col133 = eval.next_trace_mask();
        let triple_xor_32_output_limb_1_col134 = eval.next_trace_mask();
        let enabler = eval.next_trace_mask();

        eval.add_constraint(enabler.clone() * enabler.clone() - enabler.clone());

        #[allow(clippy::unused_unit)]
        #[allow(unused_variables)]
        let [create_blake_round_input_output_tmp_8e0ec_12_limb_0, create_blake_round_input_output_tmp_8e0ec_12_limb_1, create_blake_round_input_output_tmp_8e0ec_12_limb_2, create_blake_round_input_output_tmp_8e0ec_12_limb_3, create_blake_round_input_output_tmp_8e0ec_12_limb_4, create_blake_round_input_output_tmp_8e0ec_12_limb_5, create_blake_round_input_output_tmp_8e0ec_12_limb_6, create_blake_round_input_output_tmp_8e0ec_12_limb_7, create_blake_round_input_output_tmp_8e0ec_12_limb_8, create_blake_round_input_output_tmp_8e0ec_12_limb_9, create_blake_round_input_output_tmp_8e0ec_12_limb_10, create_blake_round_input_output_tmp_8e0ec_12_limb_11, create_blake_round_input_output_tmp_8e0ec_12_limb_12, create_blake_round_input_output_tmp_8e0ec_12_limb_13, create_blake_round_input_output_tmp_8e0ec_12_limb_14, create_blake_round_input_output_tmp_8e0ec_12_limb_15, create_blake_round_input_output_tmp_8e0ec_12_limb_24, create_blake_round_input_output_tmp_8e0ec_12_limb_25, create_blake_round_input_output_tmp_8e0ec_12_limb_28, create_blake_round_input_output_tmp_8e0ec_12_limb_29] =
            CreateBlakeRoundInput::evaluate(
                [
                    input_state_before_limb0_limb_0_col0.clone(),
                    input_state_before_limb0_limb_1_col1.clone(),
                    input_state_before_limb1_limb_0_col2.clone(),
                    input_state_before_limb1_limb_1_col3.clone(),
                    input_state_before_limb2_limb_0_col4.clone(),
                    input_state_before_limb2_limb_1_col5.clone(),
                    input_state_before_limb3_limb_0_col6.clone(),
                    input_state_before_limb3_limb_1_col7.clone(),
                    input_state_before_limb4_limb_0_col8.clone(),
                    input_state_before_limb4_limb_1_col9.clone(),
                    input_state_before_limb5_limb_0_col10.clone(),
                    input_state_before_limb5_limb_1_col11.clone(),
                    input_state_before_limb6_limb_0_col12.clone(),
                    input_state_before_limb6_limb_1_col13.clone(),
                    input_state_before_limb7_limb_0_col14.clone(),
                    input_state_before_limb7_limb_1_col15.clone(),
                    finalize_flag.clone(),
                ],
                ms_8_bits_col48.clone(),
                ms_8_bits_col49.clone(),
                xor_col50.clone(),
                xor_col51.clone(),
                xor_col52.clone(),
                xor_col53.clone(),
                &self.common_lookup_elements,
                t0.clone(),
                t1.clone(),
                &mut eval,
            );
        Qm31IntoU32::evaluate(
            [
                input_message_limb0_col32.clone(),
                input_message_limb1_col33.clone(),
                input_message_limb2_col34.clone(),
                input_message_limb3_col35.clone(),
                input_message_limb4_col36.clone(),
                input_message_limb5_col37.clone(),
                input_message_limb6_col38.clone(),
                input_message_limb7_col39.clone(),
                input_message_limb8_col40.clone(),
                input_message_limb9_col41.clone(),
                input_message_limb10_col42.clone(),
                input_message_limb11_col43.clone(),
                input_message_limb12_col44.clone(),
                input_message_limb13_col45.clone(),
                input_message_limb14_col46.clone(),
                input_message_limb15_col47.clone(),
                seq.clone(),
            ],
            limbi_low_col54.clone(),
            limbi_high_col55.clone(),
            limbi_low_col56.clone(),
            limbi_high_col57.clone(),
            limbi_low_col58.clone(),
            limbi_high_col59.clone(),
            limbi_low_col60.clone(),
            limbi_high_col61.clone(),
            limbi_low_col62.clone(),
            limbi_high_col63.clone(),
            limbi_low_col64.clone(),
            limbi_high_col65.clone(),
            limbi_low_col66.clone(),
            limbi_high_col67.clone(),
            limbi_low_col68.clone(),
            limbi_high_col69.clone(),
            limbi_low_col70.clone(),
            limbi_high_col71.clone(),
            limbi_low_col72.clone(),
            limbi_high_col73.clone(),
            limbi_low_col74.clone(),
            limbi_high_col75.clone(),
            limbi_low_col76.clone(),
            limbi_high_col77.clone(),
            limbi_low_col78.clone(),
            limbi_high_col79.clone(),
            limbi_low_col80.clone(),
            limbi_high_col81.clone(),
            limbi_low_col82.clone(),
            limbi_high_col83.clone(),
            limbi_low_col84.clone(),
            limbi_high_col85.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::one(),
            &[
                M31_40528774.clone(),
                seq.clone(),
                M31_0.clone(),
                input_state_before_limb0_limb_0_col0.clone(),
                input_state_before_limb0_limb_1_col1.clone(),
                input_state_before_limb1_limb_0_col2.clone(),
                input_state_before_limb1_limb_1_col3.clone(),
                input_state_before_limb2_limb_0_col4.clone(),
                input_state_before_limb2_limb_1_col5.clone(),
                input_state_before_limb3_limb_0_col6.clone(),
                input_state_before_limb3_limb_1_col7.clone(),
                input_state_before_limb4_limb_0_col8.clone(),
                input_state_before_limb4_limb_1_col9.clone(),
                input_state_before_limb5_limb_0_col10.clone(),
                input_state_before_limb5_limb_1_col11.clone(),
                input_state_before_limb6_limb_0_col12.clone(),
                input_state_before_limb6_limb_1_col13.clone(),
                input_state_before_limb7_limb_0_col14.clone(),
                input_state_before_limb7_limb_1_col15.clone(),
                M31_58983.clone(),
                M31_27145.clone(),
                M31_44677.clone(),
                M31_47975.clone(),
                M31_62322.clone(),
                M31_15470.clone(),
                M31_62778.clone(),
                M31_42319.clone(),
                create_blake_round_input_output_tmp_8e0ec_12_limb_24.clone(),
                create_blake_round_input_output_tmp_8e0ec_12_limb_25.clone(),
                M31_26764.clone(),
                M31_39685.clone(),
                create_blake_round_input_output_tmp_8e0ec_12_limb_28.clone(),
                create_blake_round_input_output_tmp_8e0ec_12_limb_29.clone(),
                M31_52505.clone(),
                M31_23520.clone(),
                seq.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::one(),
            &[
                M31_40528774.clone(),
                seq.clone(),
                M31_10.clone(),
                blake_round_output_limb_0_col86.clone(),
                blake_round_output_limb_1_col87.clone(),
                blake_round_output_limb_2_col88.clone(),
                blake_round_output_limb_3_col89.clone(),
                blake_round_output_limb_4_col90.clone(),
                blake_round_output_limb_5_col91.clone(),
                blake_round_output_limb_6_col92.clone(),
                blake_round_output_limb_7_col93.clone(),
                blake_round_output_limb_8_col94.clone(),
                blake_round_output_limb_9_col95.clone(),
                blake_round_output_limb_10_col96.clone(),
                blake_round_output_limb_11_col97.clone(),
                blake_round_output_limb_12_col98.clone(),
                blake_round_output_limb_13_col99.clone(),
                blake_round_output_limb_14_col100.clone(),
                blake_round_output_limb_15_col101.clone(),
                blake_round_output_limb_16_col102.clone(),
                blake_round_output_limb_17_col103.clone(),
                blake_round_output_limb_18_col104.clone(),
                blake_round_output_limb_19_col105.clone(),
                blake_round_output_limb_20_col106.clone(),
                blake_round_output_limb_21_col107.clone(),
                blake_round_output_limb_22_col108.clone(),
                blake_round_output_limb_23_col109.clone(),
                blake_round_output_limb_24_col110.clone(),
                blake_round_output_limb_25_col111.clone(),
                blake_round_output_limb_26_col112.clone(),
                blake_round_output_limb_27_col113.clone(),
                blake_round_output_limb_28_col114.clone(),
                blake_round_output_limb_29_col115.clone(),
                blake_round_output_limb_30_col116.clone(),
                blake_round_output_limb_31_col117.clone(),
                blake_round_output_limb_32_col118.clone(),
            ],
        ));

        CreateBlakeOutput::evaluate(
            [
                input_state_before_limb0_limb_0_col0.clone(),
                input_state_before_limb0_limb_1_col1.clone(),
                input_state_before_limb1_limb_0_col2.clone(),
                input_state_before_limb1_limb_1_col3.clone(),
                input_state_before_limb2_limb_0_col4.clone(),
                input_state_before_limb2_limb_1_col5.clone(),
                input_state_before_limb3_limb_0_col6.clone(),
                input_state_before_limb3_limb_1_col7.clone(),
                input_state_before_limb4_limb_0_col8.clone(),
                input_state_before_limb4_limb_1_col9.clone(),
                input_state_before_limb5_limb_0_col10.clone(),
                input_state_before_limb5_limb_1_col11.clone(),
                input_state_before_limb6_limb_0_col12.clone(),
                input_state_before_limb6_limb_1_col13.clone(),
                input_state_before_limb7_limb_0_col14.clone(),
                input_state_before_limb7_limb_1_col15.clone(),
                blake_round_output_limb_0_col86.clone(),
                blake_round_output_limb_1_col87.clone(),
                blake_round_output_limb_2_col88.clone(),
                blake_round_output_limb_3_col89.clone(),
                blake_round_output_limb_4_col90.clone(),
                blake_round_output_limb_5_col91.clone(),
                blake_round_output_limb_6_col92.clone(),
                blake_round_output_limb_7_col93.clone(),
                blake_round_output_limb_8_col94.clone(),
                blake_round_output_limb_9_col95.clone(),
                blake_round_output_limb_10_col96.clone(),
                blake_round_output_limb_11_col97.clone(),
                blake_round_output_limb_12_col98.clone(),
                blake_round_output_limb_13_col99.clone(),
                blake_round_output_limb_14_col100.clone(),
                blake_round_output_limb_15_col101.clone(),
                blake_round_output_limb_16_col102.clone(),
                blake_round_output_limb_17_col103.clone(),
                blake_round_output_limb_18_col104.clone(),
                blake_round_output_limb_19_col105.clone(),
                blake_round_output_limb_20_col106.clone(),
                blake_round_output_limb_21_col107.clone(),
                blake_round_output_limb_22_col108.clone(),
                blake_round_output_limb_23_col109.clone(),
                blake_round_output_limb_24_col110.clone(),
                blake_round_output_limb_25_col111.clone(),
                blake_round_output_limb_26_col112.clone(),
                blake_round_output_limb_27_col113.clone(),
                blake_round_output_limb_28_col114.clone(),
                blake_round_output_limb_29_col115.clone(),
                blake_round_output_limb_30_col116.clone(),
                blake_round_output_limb_31_col117.clone(),
            ],
            triple_xor_32_output_limb_0_col119.clone(),
            triple_xor_32_output_limb_1_col120.clone(),
            triple_xor_32_output_limb_0_col121.clone(),
            triple_xor_32_output_limb_1_col122.clone(),
            triple_xor_32_output_limb_0_col123.clone(),
            triple_xor_32_output_limb_1_col124.clone(),
            triple_xor_32_output_limb_0_col125.clone(),
            triple_xor_32_output_limb_1_col126.clone(),
            triple_xor_32_output_limb_0_col127.clone(),
            triple_xor_32_output_limb_1_col128.clone(),
            triple_xor_32_output_limb_0_col129.clone(),
            triple_xor_32_output_limb_1_col130.clone(),
            triple_xor_32_output_limb_0_col131.clone(),
            triple_xor_32_output_limb_1_col132.clone(),
            triple_xor_32_output_limb_0_col133.clone(),
            triple_xor_32_output_limb_1_col134.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        // Blake output h[0].low() matches expected.
        eval.add_constraint(
            (triple_xor_32_output_limb_0_col119.clone()
                - input_state_after_limb0_limb_0_col16.clone()),
        );
        // Blake output h[0].high() matches expected.
        eval.add_constraint(
            (triple_xor_32_output_limb_1_col120.clone()
                - input_state_after_limb0_limb_1_col17.clone()),
        );
        // Blake output h[1].low() matches expected.
        eval.add_constraint(
            (triple_xor_32_output_limb_0_col121.clone()
                - input_state_after_limb1_limb_0_col18.clone()),
        );
        // Blake output h[1].high() matches expected.
        eval.add_constraint(
            (triple_xor_32_output_limb_1_col122.clone()
                - input_state_after_limb1_limb_1_col19.clone()),
        );
        // Blake output h[2].low() matches expected.
        eval.add_constraint(
            (triple_xor_32_output_limb_0_col123.clone()
                - input_state_after_limb2_limb_0_col20.clone()),
        );
        // Blake output h[2].high() matches expected.
        eval.add_constraint(
            (triple_xor_32_output_limb_1_col124.clone()
                - input_state_after_limb2_limb_1_col21.clone()),
        );
        // Blake output h[3].low() matches expected.
        eval.add_constraint(
            (triple_xor_32_output_limb_0_col125.clone()
                - input_state_after_limb3_limb_0_col22.clone()),
        );
        // Blake output h[3].high() matches expected.
        eval.add_constraint(
            (triple_xor_32_output_limb_1_col126.clone()
                - input_state_after_limb3_limb_1_col23.clone()),
        );
        // Blake output h[4].low() matches expected.
        eval.add_constraint(
            (triple_xor_32_output_limb_0_col127.clone()
                - input_state_after_limb4_limb_0_col24.clone()),
        );
        // Blake output h[4].high() matches expected.
        eval.add_constraint(
            (triple_xor_32_output_limb_1_col128.clone()
                - input_state_after_limb4_limb_1_col25.clone()),
        );
        // Blake output h[5].low() matches expected.
        eval.add_constraint(
            (triple_xor_32_output_limb_0_col129.clone()
                - input_state_after_limb5_limb_0_col26.clone()),
        );
        // Blake output h[5].high() matches expected.
        eval.add_constraint(
            (triple_xor_32_output_limb_1_col130.clone()
                - input_state_after_limb5_limb_1_col27.clone()),
        );
        // Blake output h[6].low() matches expected.
        eval.add_constraint(
            (triple_xor_32_output_limb_0_col131.clone()
                - input_state_after_limb6_limb_0_col28.clone()),
        );
        // Blake output h[6].high() matches expected.
        eval.add_constraint(
            (triple_xor_32_output_limb_1_col132.clone()
                - input_state_after_limb6_limb_1_col29.clone()),
        );
        // Blake output h[7].low() matches expected.
        eval.add_constraint(
            (triple_xor_32_output_limb_0_col133.clone()
                - input_state_after_limb7_limb_0_col30.clone()),
        );
        // Blake output h[7].high() matches expected.
        eval.add_constraint(
            (triple_xor_32_output_limb_1_col134.clone()
                - input_state_after_limb7_limb_1_col31.clone()),
        );
        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::one(),
            &[
                M31_1061955672.clone(),
                state_before_addr.clone(),
                input_state_before_limb0_limb_0_col0.clone(),
                input_state_before_limb0_limb_1_col1.clone(),
                input_state_before_limb1_limb_0_col2.clone(),
                input_state_before_limb1_limb_1_col3.clone(),
                input_state_before_limb2_limb_0_col4.clone(),
                input_state_before_limb2_limb_1_col5.clone(),
                input_state_before_limb3_limb_0_col6.clone(),
                input_state_before_limb3_limb_1_col7.clone(),
                input_state_before_limb4_limb_0_col8.clone(),
                input_state_before_limb4_limb_1_col9.clone(),
                input_state_before_limb5_limb_0_col10.clone(),
                input_state_before_limb5_limb_1_col11.clone(),
                input_state_before_limb6_limb_0_col12.clone(),
                input_state_before_limb6_limb_1_col13.clone(),
                input_state_before_limb7_limb_0_col14.clone(),
                input_state_before_limb7_limb_1_col15.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1061955672.clone(),
                state_after_addr.clone(),
                input_state_after_limb0_limb_0_col16.clone(),
                input_state_after_limb0_limb_1_col17.clone(),
                input_state_after_limb1_limb_0_col18.clone(),
                input_state_after_limb1_limb_1_col19.clone(),
                input_state_after_limb2_limb_0_col20.clone(),
                input_state_after_limb2_limb_1_col21.clone(),
                input_state_after_limb3_limb_0_col22.clone(),
                input_state_after_limb3_limb_1_col23.clone(),
                input_state_after_limb4_limb_0_col24.clone(),
                input_state_after_limb4_limb_1_col25.clone(),
                input_state_after_limb5_limb_0_col26.clone(),
                input_state_after_limb5_limb_1_col27.clone(),
                input_state_after_limb6_limb_0_col28.clone(),
                input_state_after_limb6_limb_1_col29.clone(),
                input_state_after_limb7_limb_0_col30.clone(),
                input_state_after_limb7_limb_1_col31.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_378353459.clone(),
                message0_addr.clone(),
                input_message_limb0_col32.clone(),
                input_message_limb1_col33.clone(),
                input_message_limb2_col34.clone(),
                input_message_limb3_col35.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_378353459.clone(),
                message1_addr.clone(),
                input_message_limb4_col36.clone(),
                input_message_limb5_col37.clone(),
                input_message_limb6_col38.clone(),
                input_message_limb7_col39.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_378353459.clone(),
                message2_addr.clone(),
                input_message_limb8_col40.clone(),
                input_message_limb9_col41.clone(),
                input_message_limb10_col42.clone(),
                input_message_limb11_col43.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(enabler.clone()),
            &[
                M31_378353459.clone(),
                message3_addr.clone(),
                input_message_limb12_col44.clone(),
                input_message_limb13_col45.clone(),
                input_message_limb14_col46.clone(),
                input_message_limb15_col47.clone(),
            ],
        ));

        eval.finalize_logup_in_pairs();
        eval
    }
}

// #[cfg(test)]
// mod tests {
//     use num_traits::Zero;
//     use rand::rngs::SmallRng;
//     use rand::{Rng, SeedableRng};
//     use stwo::core::fields::qm31::QM31;
//     use stwo_constraint_framework::expr::ExprEvaluator;

//     use super::*;
//     use crate::circuit_air::components::constraints_regression_test_values::BLAKE_GATE;

//     #[test]
//     fn blake_gate_constraints_regression() {
//         let mut rng = SmallRng::seed_from_u64(0);
//         let eval = Eval {
//             claim: Claim { log_size: 4 },
//             common_lookup_elements: relations::CommonLookupElements::dummy(),
//         };
//         let expr_eval = eval.evaluate(ExprEvaluator::new());
//         let assignment = expr_eval.random_assignment();

//         let mut sum = QM31::zero();
//         for c in expr_eval.constraints {
//             sum += c.assign(&assignment) * rng.gen_range(0..QM31::MODULUS);
//         }

//         BLAKE_GATE.assert_debug_eq(&sum);
//     }
// }
