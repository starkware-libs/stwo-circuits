// This file was created by the AIR team.

use crate::circuit_air::components::prelude::*;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Qm31IntoU32 {}

impl Qm31IntoU32 {
    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    #[allow(clippy::unused_unit)]
    #[allow(unused_variables)]
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate<E: EvalAtRow>(
        [qm_31_into_u_32_input_limb_0, qm_31_into_u_32_input_limb_1, qm_31_into_u_32_input_limb_2, qm_31_into_u_32_input_limb_3, qm_31_into_u_32_input_limb_4, qm_31_into_u_32_input_limb_5, qm_31_into_u_32_input_limb_6, qm_31_into_u_32_input_limb_7, qm_31_into_u_32_input_limb_8, qm_31_into_u_32_input_limb_9, qm_31_into_u_32_input_limb_10, qm_31_into_u_32_input_limb_11, qm_31_into_u_32_input_limb_12, qm_31_into_u_32_input_limb_13, qm_31_into_u_32_input_limb_14, qm_31_into_u_32_input_limb_15, qm_31_into_u_32_input_limb_16]: [E::F; 17],
        limb0_limb_0_col0: E::F,
        limb0_limb_1_col1: E::F,
        limb1_limb_0_col2: E::F,
        limb1_limb_1_col3: E::F,
        limb2_limb_0_col4: E::F,
        limb2_limb_1_col5: E::F,
        limb3_limb_0_col6: E::F,
        limb3_limb_1_col7: E::F,
        limb4_limb_0_col8: E::F,
        limb4_limb_1_col9: E::F,
        limb5_limb_0_col10: E::F,
        limb5_limb_1_col11: E::F,
        limb6_limb_0_col12: E::F,
        limb6_limb_1_col13: E::F,
        limb7_limb_0_col14: E::F,
        limb7_limb_1_col15: E::F,
        limb8_limb_0_col16: E::F,
        limb8_limb_1_col17: E::F,
        limb9_limb_0_col18: E::F,
        limb9_limb_1_col19: E::F,
        limb10_limb_0_col20: E::F,
        limb10_limb_1_col21: E::F,
        limb11_limb_0_col22: E::F,
        limb11_limb_1_col23: E::F,
        limb12_limb_0_col24: E::F,
        limb12_limb_1_col25: E::F,
        limb13_limb_0_col26: E::F,
        limb13_limb_1_col27: E::F,
        limb14_limb_0_col28: E::F,
        limb14_limb_1_col29: E::F,
        limb15_limb_0_col30: E::F,
        limb15_limb_1_col31: E::F,
        common_lookup_elements: &relations::CommonLookupElements,
        eval: &mut E,
    ) -> [E::F; 0] {
        let M31_0 = E::F::from(M31::from(0));
        let M31_1 = E::F::from(M31::from(1));
        let M31_10 = E::F::from(M31::from(10));
        let M31_1008385708 = E::F::from(M31::from(1008385708));
        let M31_1058718565 = E::F::from(M31::from(1058718565));
        let M31_11 = E::F::from(M31::from(11));
        let M31_12 = E::F::from(M31::from(12));
        let M31_13 = E::F::from(M31::from(13));
        let M31_14 = E::F::from(M31::from(14));
        let M31_1492981981 = E::F::from(M31::from(1492981981));
        let M31_15 = E::F::from(M31::from(15));
        let M31_2 = E::F::from(M31::from(2));
        let M31_3 = E::F::from(M31::from(3));
        let M31_4 = E::F::from(M31::from(4));
        let M31_5 = E::F::from(M31::from(5));
        let M31_6 = E::F::from(M31::from(6));
        let M31_65536 = E::F::from(M31::from(65536));
        let M31_7 = E::F::from(M31::from(7));
        let M31_8 = E::F::from(M31::from(8));
        let M31_9 = E::F::from(M31::from(9));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limb0_limb_0_col0.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limb0_limb_1_col1.clone()],
        ));

        // limb 0 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_0.clone()
                - (limb0_limb_0_col0.clone() + (limb0_limb_1_col1.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_0.clone(),
                limb0_limb_0_col0.clone(),
                limb0_limb_1_col1.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limb1_limb_0_col2.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limb1_limb_1_col3.clone()],
        ));

        // limb 1 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_1.clone()
                - (limb1_limb_0_col2.clone() + (limb1_limb_1_col3.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_1.clone(),
                limb1_limb_0_col2.clone(),
                limb1_limb_1_col3.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limb2_limb_0_col4.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limb2_limb_1_col5.clone()],
        ));

        // limb 2 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_2.clone()
                - (limb2_limb_0_col4.clone() + (limb2_limb_1_col5.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_2.clone(),
                limb2_limb_0_col4.clone(),
                limb2_limb_1_col5.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limb3_limb_0_col6.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limb3_limb_1_col7.clone()],
        ));

        // limb 3 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_3.clone()
                - (limb3_limb_0_col6.clone() + (limb3_limb_1_col7.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_3.clone(),
                limb3_limb_0_col6.clone(),
                limb3_limb_1_col7.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limb4_limb_0_col8.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limb4_limb_1_col9.clone()],
        ));

        // limb 4 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_4.clone()
                - (limb4_limb_0_col8.clone() + (limb4_limb_1_col9.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_4.clone(),
                limb4_limb_0_col8.clone(),
                limb4_limb_1_col9.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limb5_limb_0_col10.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limb5_limb_1_col11.clone()],
        ));

        // limb 5 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_5.clone()
                - (limb5_limb_0_col10.clone() + (limb5_limb_1_col11.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_5.clone(),
                limb5_limb_0_col10.clone(),
                limb5_limb_1_col11.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limb6_limb_0_col12.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limb6_limb_1_col13.clone()],
        ));

        // limb 6 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_6.clone()
                - (limb6_limb_0_col12.clone() + (limb6_limb_1_col13.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_6.clone(),
                limb6_limb_0_col12.clone(),
                limb6_limb_1_col13.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limb7_limb_0_col14.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limb7_limb_1_col15.clone()],
        ));

        // limb 7 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_7.clone()
                - (limb7_limb_0_col14.clone() + (limb7_limb_1_col15.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_7.clone(),
                limb7_limb_0_col14.clone(),
                limb7_limb_1_col15.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limb8_limb_0_col16.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limb8_limb_1_col17.clone()],
        ));

        // limb 8 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_8.clone()
                - (limb8_limb_0_col16.clone() + (limb8_limb_1_col17.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_8.clone(),
                limb8_limb_0_col16.clone(),
                limb8_limb_1_col17.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limb9_limb_0_col18.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limb9_limb_1_col19.clone()],
        ));

        // limb 9 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_9.clone()
                - (limb9_limb_0_col18.clone() + (limb9_limb_1_col19.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_9.clone(),
                limb9_limb_0_col18.clone(),
                limb9_limb_1_col19.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limb10_limb_0_col20.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limb10_limb_1_col21.clone()],
        ));

        // limb 10 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_10.clone()
                - (limb10_limb_0_col20.clone()
                    + (limb10_limb_1_col21.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_10.clone(),
                limb10_limb_0_col20.clone(),
                limb10_limb_1_col21.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limb11_limb_0_col22.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limb11_limb_1_col23.clone()],
        ));

        // limb 11 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_11.clone()
                - (limb11_limb_0_col22.clone()
                    + (limb11_limb_1_col23.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_11.clone(),
                limb11_limb_0_col22.clone(),
                limb11_limb_1_col23.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limb12_limb_0_col24.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limb12_limb_1_col25.clone()],
        ));

        // limb 12 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_12.clone()
                - (limb12_limb_0_col24.clone()
                    + (limb12_limb_1_col25.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_12.clone(),
                limb12_limb_0_col24.clone(),
                limb12_limb_1_col25.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limb13_limb_0_col26.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limb13_limb_1_col27.clone()],
        ));

        // limb 13 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_13.clone()
                - (limb13_limb_0_col26.clone()
                    + (limb13_limb_1_col27.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_13.clone(),
                limb13_limb_0_col26.clone(),
                limb13_limb_1_col27.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limb14_limb_0_col28.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limb14_limb_1_col29.clone()],
        ));

        // limb 14 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_14.clone()
                - (limb14_limb_0_col28.clone()
                    + (limb14_limb_1_col29.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_14.clone(),
                limb14_limb_0_col28.clone(),
                limb14_limb_1_col29.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limb15_limb_0_col30.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limb15_limb_1_col31.clone()],
        ));

        // limb 15 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_15.clone()
                - (limb15_limb_0_col30.clone()
                    + (limb15_limb_1_col31.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_15.clone(),
                limb15_limb_0_col30.clone(),
                limb15_limb_1_col31.clone(),
            ],
        ));

        []
    }
}
