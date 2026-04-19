use crate::components::prelude::*;
use crate::components::subroutines::bitwise_xor_num_bits_8::BitwiseXorNumBits8;
use crate::components::subroutines::split_16_low_part_size_8::Split16LowPartSize8;

pub const N_TRACE_COLUMNS: usize = 20;
pub const RELATION_USES_PER_ROW: [RelationUse; 2] = [
    RelationUse { relation_id: "Gate", uses: 3 },
    RelationUse { relation_id: "VerifyBitwiseXor_8", uses: 8 },
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
        let interaction_log_sizes = vec![self.log_size; SECURE_EXTENSION_DEGREE * 6];
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
        let M31_1 = E::F::from(M31::from(1));
        let M31_112558620 = E::F::from(M31::from(112558620));
        let M31_378353459 = E::F::from(M31::from(378353459));
        let triple_xor_input_addr_0 = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "triple_xor_input_addr_0".to_owned(),
        });
        let triple_xor_input_addr_1 = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "triple_xor_input_addr_1".to_owned(),
        });
        let triple_xor_input_addr_2 = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "triple_xor_input_addr_2".to_owned(),
        });
        let triple_xor_output_addr = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "triple_xor_output_addr".to_owned(),
        });
        let triple_xor_multiplicity = eval.get_preprocessed_column(PreProcessedColumnId {
            id: "triple_xor_multiplicity".to_owned(),
        });
        let input_a_limb_0_col0 = eval.next_trace_mask();
        let input_a_limb_1_col1 = eval.next_trace_mask();
        let input_b_limb_0_col2 = eval.next_trace_mask();
        let input_b_limb_1_col3 = eval.next_trace_mask();
        let input_c_limb_0_col4 = eval.next_trace_mask();
        let input_c_limb_1_col5 = eval.next_trace_mask();
        let input_a_xor_b_xor_c_limb_0_col6 = eval.next_trace_mask();
        let input_a_xor_b_xor_c_limb_1_col7 = eval.next_trace_mask();
        let ms_8_bits_col8 = eval.next_trace_mask();
        let ms_8_bits_col9 = eval.next_trace_mask();
        let ms_8_bits_col10 = eval.next_trace_mask();
        let ms_8_bits_col11 = eval.next_trace_mask();
        let ms_8_bits_col12 = eval.next_trace_mask();
        let ms_8_bits_col13 = eval.next_trace_mask();
        let ms_8_bits_col14 = eval.next_trace_mask();
        let ms_8_bits_col15 = eval.next_trace_mask();
        let xor_col16 = eval.next_trace_mask();
        let xor_col17 = eval.next_trace_mask();
        let xor_col18 = eval.next_trace_mask();
        let xor_col19 = eval.next_trace_mask();

        #[allow(clippy::unused_unit)]
        #[allow(unused_variables)]
        let [split_16_low_part_size_8_output_tmp_568ba_1_limb_0] = Split16LowPartSize8::evaluate(
            [input_a_limb_0_col0.clone()],
            ms_8_bits_col8.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        #[allow(clippy::unused_unit)]
        #[allow(unused_variables)]
        let [split_16_low_part_size_8_output_tmp_568ba_3_limb_0] = Split16LowPartSize8::evaluate(
            [input_a_limb_1_col1.clone()],
            ms_8_bits_col9.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        #[allow(clippy::unused_unit)]
        #[allow(unused_variables)]
        let [split_16_low_part_size_8_output_tmp_568ba_5_limb_0] = Split16LowPartSize8::evaluate(
            [input_b_limb_0_col2.clone()],
            ms_8_bits_col10.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        #[allow(clippy::unused_unit)]
        #[allow(unused_variables)]
        let [split_16_low_part_size_8_output_tmp_568ba_7_limb_0] = Split16LowPartSize8::evaluate(
            [input_b_limb_1_col3.clone()],
            ms_8_bits_col11.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        #[allow(clippy::unused_unit)]
        #[allow(unused_variables)]
        let [split_16_low_part_size_8_output_tmp_568ba_9_limb_0] = Split16LowPartSize8::evaluate(
            [input_c_limb_0_col4.clone()],
            ms_8_bits_col12.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        #[allow(clippy::unused_unit)]
        #[allow(unused_variables)]
        let [split_16_low_part_size_8_output_tmp_568ba_11_limb_0] = Split16LowPartSize8::evaluate(
            [input_c_limb_1_col5.clone()],
            ms_8_bits_col13.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        #[allow(clippy::unused_unit)]
        #[allow(unused_variables)]
        let [split_16_low_part_size_8_output_tmp_568ba_13_limb_0] = Split16LowPartSize8::evaluate(
            [input_a_xor_b_xor_c_limb_0_col6.clone()],
            ms_8_bits_col14.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        #[allow(clippy::unused_unit)]
        #[allow(unused_variables)]
        let [split_16_low_part_size_8_output_tmp_568ba_15_limb_0] = Split16LowPartSize8::evaluate(
            [input_a_xor_b_xor_c_limb_1_col7.clone()],
            ms_8_bits_col15.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        BitwiseXorNumBits8::evaluate(
            [
                split_16_low_part_size_8_output_tmp_568ba_1_limb_0.clone(),
                split_16_low_part_size_8_output_tmp_568ba_5_limb_0.clone(),
            ],
            xor_col16.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        BitwiseXorNumBits8::evaluate(
            [ms_8_bits_col8.clone(), ms_8_bits_col10.clone()],
            xor_col17.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        BitwiseXorNumBits8::evaluate(
            [
                split_16_low_part_size_8_output_tmp_568ba_3_limb_0.clone(),
                split_16_low_part_size_8_output_tmp_568ba_7_limb_0.clone(),
            ],
            xor_col18.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        BitwiseXorNumBits8::evaluate(
            [ms_8_bits_col9.clone(), ms_8_bits_col11.clone()],
            xor_col19.clone(),
            &self.common_lookup_elements,
            &mut eval,
        );
        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[
                M31_112558620.clone(),
                xor_col16.clone(),
                split_16_low_part_size_8_output_tmp_568ba_9_limb_0.clone(),
                split_16_low_part_size_8_output_tmp_568ba_13_limb_0.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[
                M31_112558620.clone(),
                xor_col17.clone(),
                ms_8_bits_col12.clone(),
                ms_8_bits_col14.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[
                M31_112558620.clone(),
                xor_col18.clone(),
                split_16_low_part_size_8_output_tmp_568ba_11_limb_0.clone(),
                split_16_low_part_size_8_output_tmp_568ba_15_limb_0.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[
                M31_112558620.clone(),
                xor_col19.clone(),
                ms_8_bits_col13.clone(),
                ms_8_bits_col15.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[
                M31_378353459.clone(),
                triple_xor_input_addr_0.clone(),
                input_a_limb_0_col0.clone(),
                input_a_limb_1_col1.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[
                M31_378353459.clone(),
                triple_xor_input_addr_1.clone(),
                input_b_limb_0_col2.clone(),
                input_b_limb_1_col3.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[
                M31_378353459.clone(),
                triple_xor_input_addr_2.clone(),
                input_c_limb_0_col4.clone(),
                input_c_limb_1_col5.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.common_lookup_elements,
            -E::EF::from(triple_xor_multiplicity.clone()),
            &[
                M31_378353459.clone(),
                triple_xor_output_addr.clone(),
                input_a_xor_b_xor_c_limb_0_col6.clone(),
                input_a_xor_b_xor_c_limb_1_col7.clone(),
            ],
        ));

        eval.finalize_logup_in_pairs();
        eval
    }
}
