// This file was created by the AIR team.

use crate::circuit_air::components::prelude::*;

#[derive(Copy, Clone, Serialize)]
pub struct Qm31IntoU32 {}

impl Qm31IntoU32 {
    #[allow(unused_parens)]
    #[allow(clippy::double_parens)]
    #[allow(non_snake_case)]
    #[allow(clippy::unused_unit)]
    #[allow(unused_variables)]
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate<E: EvalAtRow>(
        [
            qm_31_into_u_32_input_limb_0,
            qm_31_into_u_32_input_limb_1,
            qm_31_into_u_32_input_limb_2,
            qm_31_into_u_32_input_limb_3,
            qm_31_into_u_32_input_limb_4,
            qm_31_into_u_32_input_limb_5,
            qm_31_into_u_32_input_limb_6,
            qm_31_into_u_32_input_limb_7,
            qm_31_into_u_32_input_limb_8,
            qm_31_into_u_32_input_limb_9,
            qm_31_into_u_32_input_limb_10,
            qm_31_into_u_32_input_limb_11,
            qm_31_into_u_32_input_limb_12,
            qm_31_into_u_32_input_limb_13,
            qm_31_into_u_32_input_limb_14,
            qm_31_into_u_32_input_limb_15,
            qm_31_into_u_32_input_limb_16,
        ]: [E::F; 17],
        limbi_low_col0: E::F,
        limbi_high_col1: E::F,
        limbi_low_col2: E::F,
        limbi_high_col3: E::F,
        limbi_low_col4: E::F,
        limbi_high_col5: E::F,
        limbi_low_col6: E::F,
        limbi_high_col7: E::F,
        limbi_low_col8: E::F,
        limbi_high_col9: E::F,
        limbi_low_col10: E::F,
        limbi_high_col11: E::F,
        limbi_low_col12: E::F,
        limbi_high_col13: E::F,
        limbi_low_col14: E::F,
        limbi_high_col15: E::F,
        limbi_low_col16: E::F,
        limbi_high_col17: E::F,
        limbi_low_col18: E::F,
        limbi_high_col19: E::F,
        limbi_low_col20: E::F,
        limbi_high_col21: E::F,
        limbi_low_col22: E::F,
        limbi_high_col23: E::F,
        limbi_low_col24: E::F,
        limbi_high_col25: E::F,
        limbi_low_col26: E::F,
        limbi_high_col27: E::F,
        limbi_low_col28: E::F,
        limbi_high_col29: E::F,
        limbi_low_col30: E::F,
        limbi_high_col31: E::F,
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
            &[M31_1008385708.clone(), limbi_low_col0.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limbi_high_col1.clone()],
        ));

        // limb 0 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_0.clone()
                - (limbi_low_col0.clone() + (limbi_high_col1.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_0.clone(),
                limbi_low_col0.clone(),
                limbi_high_col1.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limbi_low_col2.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limbi_high_col3.clone()],
        ));

        // limb 1 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_1.clone()
                - (limbi_low_col2.clone() + (limbi_high_col3.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_1.clone(),
                limbi_low_col2.clone(),
                limbi_high_col3.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limbi_low_col4.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limbi_high_col5.clone()],
        ));

        // limb 2 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_2.clone()
                - (limbi_low_col4.clone() + (limbi_high_col5.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_2.clone(),
                limbi_low_col4.clone(),
                limbi_high_col5.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limbi_low_col6.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limbi_high_col7.clone()],
        ));

        // limb 3 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_3.clone()
                - (limbi_low_col6.clone() + (limbi_high_col7.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_3.clone(),
                limbi_low_col6.clone(),
                limbi_high_col7.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limbi_low_col8.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limbi_high_col9.clone()],
        ));

        // limb 4 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_4.clone()
                - (limbi_low_col8.clone() + (limbi_high_col9.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_4.clone(),
                limbi_low_col8.clone(),
                limbi_high_col9.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limbi_low_col10.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limbi_high_col11.clone()],
        ));

        // limb 5 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_5.clone()
                - (limbi_low_col10.clone() + (limbi_high_col11.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_5.clone(),
                limbi_low_col10.clone(),
                limbi_high_col11.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limbi_low_col12.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limbi_high_col13.clone()],
        ));

        // limb 6 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_6.clone()
                - (limbi_low_col12.clone() + (limbi_high_col13.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_6.clone(),
                limbi_low_col12.clone(),
                limbi_high_col13.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limbi_low_col14.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limbi_high_col15.clone()],
        ));

        // limb 7 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_7.clone()
                - (limbi_low_col14.clone() + (limbi_high_col15.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_7.clone(),
                limbi_low_col14.clone(),
                limbi_high_col15.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limbi_low_col16.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limbi_high_col17.clone()],
        ));

        // limb 8 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_8.clone()
                - (limbi_low_col16.clone() + (limbi_high_col17.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_8.clone(),
                limbi_low_col16.clone(),
                limbi_high_col17.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limbi_low_col18.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limbi_high_col19.clone()],
        ));

        // limb 9 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_9.clone()
                - (limbi_low_col18.clone() + (limbi_high_col19.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_9.clone(),
                limbi_low_col18.clone(),
                limbi_high_col19.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limbi_low_col20.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limbi_high_col21.clone()],
        ));

        // limb 10 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_10.clone()
                - (limbi_low_col20.clone() + (limbi_high_col21.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_10.clone(),
                limbi_low_col20.clone(),
                limbi_high_col21.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limbi_low_col22.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limbi_high_col23.clone()],
        ));

        // limb 11 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_11.clone()
                - (limbi_low_col22.clone() + (limbi_high_col23.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_11.clone(),
                limbi_low_col22.clone(),
                limbi_high_col23.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limbi_low_col24.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limbi_high_col25.clone()],
        ));

        // limb 12 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_12.clone()
                - (limbi_low_col24.clone() + (limbi_high_col25.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_12.clone(),
                limbi_low_col24.clone(),
                limbi_high_col25.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limbi_low_col26.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limbi_high_col27.clone()],
        ));

        // limb 13 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_13.clone()
                - (limbi_low_col26.clone() + (limbi_high_col27.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_13.clone(),
                limbi_low_col26.clone(),
                limbi_high_col27.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limbi_low_col28.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limbi_high_col29.clone()],
        ));

        // limb 14 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_14.clone()
                - (limbi_low_col28.clone() + (limbi_high_col29.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_14.clone(),
                limbi_low_col28.clone(),
                limbi_high_col29.clone(),
            ],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1008385708.clone(), limbi_low_col30.clone()],
        ));

        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            E::EF::one(),
            &[M31_1058718565.clone(), limbi_high_col31.clone()],
        ));

        // limb 15 reconstruction.
        eval.add_constraint(
            (qm_31_into_u_32_input_limb_15.clone()
                - (limbi_low_col30.clone() + (limbi_high_col31.clone() * M31_65536.clone()))),
        );
        eval.add_to_relation(RelationEntry::new(
            common_lookup_elements,
            -E::EF::one(),
            &[
                M31_1492981981.clone(),
                qm_31_into_u_32_input_limb_16.clone(),
                M31_15.clone(),
                limbi_low_col30.clone(),
                limbi_high_col31.clone(),
            ],
        ));

        []
    }
}
