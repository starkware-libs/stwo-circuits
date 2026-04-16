// This file was created by the AIR team.

use crate::components::prelude::*;
use crate::components::subroutines::split_16_low_part_size_8::Split16LowPartSize8;

#[derive(Copy, Clone, Serialize)]
pub struct VerifyXorRot32R8 {}

impl VerifyXorRot32R8 {
    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    #[allow(clippy::unused_unit)]
    #[allow(unused_variables)]
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate<E: EvalAtRow>(
        [
            verify_xor_rot_32_r_8_input_limb_0,
            verify_xor_rot_32_r_8_input_limb_1,
            verify_xor_rot_32_r_8_input_limb_2,
            verify_xor_rot_32_r_8_input_limb_3,
            verify_xor_rot_32_r_8_input_limb_4,
            verify_xor_rot_32_r_8_input_limb_5,
        ]: [E::F; 6],
        ms_8_bits_col0: E::F,
        ms_8_bits_col1: E::F,
        ms_8_bits_col2: E::F,
        ms_8_bits_col3: E::F,
        ms_8_bits_col4: E::F,
        ms_8_bits_col5: E::F,
        common_lookup_elements: &relations::CommonLookupElements,
        eval: &mut E,
    ) -> [E::F; 0] {
        let M31_1 = E::F::from(M31::from(1));
        let M31_112558620 = E::F::from(M31::from(112558620));

        let [split_16_low_part_size_8_output_tmp_f3b45_1_limb_0] = Split16LowPartSize8::evaluate(
            [verify_xor_rot_32_r_8_input_limb_0.clone()],
            ms_8_bits_col0.clone(),
            common_lookup_elements,
            eval,
        );
        let [split_16_low_part_size_8_output_tmp_f3b45_3_limb_0] = Split16LowPartSize8::evaluate(
            [verify_xor_rot_32_r_8_input_limb_1.clone()],
            ms_8_bits_col1.clone(),
            common_lookup_elements,
            eval,
        );
        let [split_16_low_part_size_8_output_tmp_f3b45_5_limb_0] = Split16LowPartSize8::evaluate(
            [verify_xor_rot_32_r_8_input_limb_2.clone()],
            ms_8_bits_col2.clone(),
            common_lookup_elements,
            eval,
        );
        let [split_16_low_part_size_8_output_tmp_f3b45_7_limb_0] = Split16LowPartSize8::evaluate(
            [verify_xor_rot_32_r_8_input_limb_3.clone()],
            ms_8_bits_col3.clone(),
            common_lookup_elements,
            eval,
        );
        let [split_16_low_part_size_8_output_tmp_f3b45_9_limb_0] = Split16LowPartSize8::evaluate(
            [verify_xor_rot_32_r_8_input_limb_4.clone()],
            ms_8_bits_col4.clone(),
            common_lookup_elements,
            eval,
        );
        let [split_16_low_part_size_8_output_tmp_f3b45_11_limb_0] = Split16LowPartSize8::evaluate(
            [verify_xor_rot_32_r_8_input_limb_5.clone()],
            ms_8_bits_col5.clone(),
            common_lookup_elements,
            eval,
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[
                M31_112558620.clone(),
                ms_8_bits_col0.clone(),
                ms_8_bits_col2.clone(),
                split_16_low_part_size_8_output_tmp_f3b45_9_limb_0.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[
                M31_112558620.clone(),
                split_16_low_part_size_8_output_tmp_f3b45_3_limb_0.clone(),
                split_16_low_part_size_8_output_tmp_f3b45_7_limb_0.clone(),
                ms_8_bits_col4.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[
                M31_112558620.clone(),
                ms_8_bits_col1.clone(),
                ms_8_bits_col3.clone(),
                split_16_low_part_size_8_output_tmp_f3b45_11_limb_0.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::from(M31_1.clone()),
            &[
                M31_112558620.clone(),
                split_16_low_part_size_8_output_tmp_f3b45_1_limb_0.clone(),
                split_16_low_part_size_8_output_tmp_f3b45_5_limb_0.clone(),
                ms_8_bits_col5.clone(),
            ],
        ));

        []
    }
}
