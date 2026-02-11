// This file was created by the AIR team.

use crate::components::prelude::*;
use crate::components::subroutines::bitwise_xor_num_bits_8::BitwiseXorNumBits8;
use crate::components::subroutines::split_16_low_part_size_8::Split16LowPartSize8;

#[derive(Copy, Clone, Serialize, Deserialize, CairoSerialize)]
pub struct CreateBlakeRoundInput {}

impl CreateBlakeRoundInput {
    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    #[allow(clippy::unused_unit)]
    #[allow(unused_variables)]
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate<E: EvalAtRow>(
        [create_blake_round_input_input_limb_0, create_blake_round_input_input_limb_1, create_blake_round_input_input_limb_2, create_blake_round_input_input_limb_3, create_blake_round_input_input_limb_4, create_blake_round_input_input_limb_5, create_blake_round_input_input_limb_6, create_blake_round_input_input_limb_7, create_blake_round_input_input_limb_8, create_blake_round_input_input_limb_9, create_blake_round_input_input_limb_10, create_blake_round_input_input_limb_11, create_blake_round_input_input_limb_12, create_blake_round_input_input_limb_13, create_blake_round_input_input_limb_14, create_blake_round_input_input_limb_15, create_blake_round_input_input_limb_16]: [E::F; 17],
        ms_8_bits_col0: E::F,
        ms_8_bits_col1: E::F,
        xor_col2: E::F,
        xor_col3: E::F,
        xor_col4: E::F,
        xor_col5: E::F,
        common_lookup_elements: &relations::CommonLookupElements,
        t0: E::F,
        t1: E::F,
        eval: &mut E,
    ) -> [E::F; 20] {
        let M31_1 = E::F::from(M31::from(1));
        let M31_127 = E::F::from(M31::from(127));
        let M31_14 = E::F::from(M31::from(14));
        let M31_256 = E::F::from(M31::from(256));
        let M31_55723 = E::F::from(M31::from(55723));
        let M31_57468 = E::F::from(M31::from(57468));
        let M31_8067 = E::F::from(M31::from(8067));
        let M31_81 = E::F::from(M31::from(81));
        let M31_82 = E::F::from(M31::from(82));
        let M31_9812 = E::F::from(M31::from(9812));

        let [split_16_low_part_size_8_output_tmp_4d188_1_limb_0] = Split16LowPartSize8::evaluate(
            [t0.clone()],
            ms_8_bits_col0.clone(),
            common_lookup_elements,
            eval,
        );
        let [split_16_low_part_size_8_output_tmp_4d188_3_limb_0] = Split16LowPartSize8::evaluate(
            [t1.clone()],
            ms_8_bits_col1.clone(),
            common_lookup_elements,
            eval,
        );
        BitwiseXorNumBits8::evaluate(
            [
                split_16_low_part_size_8_output_tmp_4d188_1_limb_0.clone(),
                M31_127.clone(),
            ],
            xor_col2.clone(),
            common_lookup_elements,
            eval,
        );
        BitwiseXorNumBits8::evaluate(
            [ms_8_bits_col0.clone(), M31_82.clone()],
            xor_col3.clone(),
            common_lookup_elements,
            eval,
        );
        BitwiseXorNumBits8::evaluate(
            [
                split_16_low_part_size_8_output_tmp_4d188_3_limb_0.clone(),
                M31_14.clone(),
            ],
            xor_col4.clone(),
            common_lookup_elements,
            eval,
        );
        BitwiseXorNumBits8::evaluate(
            [ms_8_bits_col1.clone(), M31_81.clone()],
            xor_col5.clone(),
            common_lookup_elements,
            eval,
        );
        [
            create_blake_round_input_input_limb_0.clone(),
            create_blake_round_input_input_limb_1.clone(),
            create_blake_round_input_input_limb_2.clone(),
            create_blake_round_input_input_limb_3.clone(),
            create_blake_round_input_input_limb_4.clone(),
            create_blake_round_input_input_limb_5.clone(),
            create_blake_round_input_input_limb_6.clone(),
            create_blake_round_input_input_limb_7.clone(),
            create_blake_round_input_input_limb_8.clone(),
            create_blake_round_input_input_limb_9.clone(),
            create_blake_round_input_input_limb_10.clone(),
            create_blake_round_input_input_limb_11.clone(),
            create_blake_round_input_input_limb_12.clone(),
            create_blake_round_input_input_limb_13.clone(),
            create_blake_round_input_input_limb_14.clone(),
            create_blake_round_input_input_limb_15.clone(),
            (xor_col2.clone() + (xor_col3.clone() * M31_256.clone())),
            (xor_col4.clone() + (xor_col5.clone() * M31_256.clone())),
            ((create_blake_round_input_input_limb_16.clone() * M31_9812.clone())
                + ((M31_1.clone() - create_blake_round_input_input_limb_16.clone())
                    * M31_55723.clone())),
            ((create_blake_round_input_input_limb_16.clone() * M31_57468.clone())
                + ((M31_1.clone() - create_blake_round_input_input_limb_16.clone())
                    * M31_8067.clone())),
        ]
    }
}
