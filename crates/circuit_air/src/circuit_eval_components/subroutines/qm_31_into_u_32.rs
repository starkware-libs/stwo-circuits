// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 2] = [
    RelationUse { relation_id: "RangeCheck_15", uses: 16 },
    RelationUse { relation_id: "RangeCheck_16", uses: 16 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
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
        limbi_low_col0,
        limbi_high_col1,
        limbi_low_col2,
        limbi_high_col3,
        limbi_low_col4,
        limbi_high_col5,
        limbi_low_col6,
        limbi_high_col7,
        limbi_low_col8,
        limbi_high_col9,
        limbi_low_col10,
        limbi_high_col11,
        limbi_low_col12,
        limbi_high_col13,
        limbi_low_col14,
        limbi_high_col15,
        limbi_low_col16,
        limbi_high_col17,
        limbi_low_col18,
        limbi_high_col19,
        limbi_low_col20,
        limbi_high_col21,
        limbi_low_col22,
        limbi_high_col23,
        limbi_low_col24,
        limbi_high_col25,
        limbi_low_col26,
        limbi_high_col27,
        limbi_low_col28,
        limbi_high_col29,
        limbi_low_col30,
        limbi_high_col31,
    ] = input.try_into().unwrap();

    // Use RangeCheck_16.
    let tuple_0 = &[eval!(context, 1008385708), eval!(context, limbi_low_col0)];
    let numerator_0 = eval!(context, 1);
    acc.add_to_relation(context, numerator_0, tuple_0);

    // Use RangeCheck_15.
    let tuple_1 = &[eval!(context, 1058718565), eval!(context, limbi_high_col1)];
    let numerator_1 = eval!(context, 1);
    acc.add_to_relation(context, numerator_1, tuple_1);

    //limb 0 reconstruction.
    let constraint_2_value = eval!(
        context,
        (qm_31_into_u_32_input_limb_0) - ((limbi_low_col0) + ((limbi_high_col1) * (65536)))
    );
    acc.add_constraint(context, constraint_2_value);

    // Yield BlakeMessage.
    let tuple_3 = &[
        eval!(context, 1492981981),
        eval!(context, qm_31_into_u_32_input_limb_16),
        eval!(context, 0),
        eval!(context, limbi_low_col0),
        eval!(context, limbi_high_col1),
    ];
    let numerator_3 = eval!(context, -(10));
    acc.add_to_relation(context, numerator_3, tuple_3);

    // Use RangeCheck_16.
    let tuple_4 = &[eval!(context, 1008385708), eval!(context, limbi_low_col2)];
    let numerator_4 = eval!(context, 1);
    acc.add_to_relation(context, numerator_4, tuple_4);

    // Use RangeCheck_15.
    let tuple_5 = &[eval!(context, 1058718565), eval!(context, limbi_high_col3)];
    let numerator_5 = eval!(context, 1);
    acc.add_to_relation(context, numerator_5, tuple_5);

    //limb 1 reconstruction.
    let constraint_6_value = eval!(
        context,
        (qm_31_into_u_32_input_limb_1) - ((limbi_low_col2) + ((limbi_high_col3) * (65536)))
    );
    acc.add_constraint(context, constraint_6_value);

    // Yield BlakeMessage.
    let tuple_7 = &[
        eval!(context, 1492981981),
        eval!(context, qm_31_into_u_32_input_limb_16),
        eval!(context, 1),
        eval!(context, limbi_low_col2),
        eval!(context, limbi_high_col3),
    ];
    let numerator_7 = eval!(context, -(10));
    acc.add_to_relation(context, numerator_7, tuple_7);

    // Use RangeCheck_16.
    let tuple_8 = &[eval!(context, 1008385708), eval!(context, limbi_low_col4)];
    let numerator_8 = eval!(context, 1);
    acc.add_to_relation(context, numerator_8, tuple_8);

    // Use RangeCheck_15.
    let tuple_9 = &[eval!(context, 1058718565), eval!(context, limbi_high_col5)];
    let numerator_9 = eval!(context, 1);
    acc.add_to_relation(context, numerator_9, tuple_9);

    //limb 2 reconstruction.
    let constraint_10_value = eval!(
        context,
        (qm_31_into_u_32_input_limb_2) - ((limbi_low_col4) + ((limbi_high_col5) * (65536)))
    );
    acc.add_constraint(context, constraint_10_value);

    // Yield BlakeMessage.
    let tuple_11 = &[
        eval!(context, 1492981981),
        eval!(context, qm_31_into_u_32_input_limb_16),
        eval!(context, 2),
        eval!(context, limbi_low_col4),
        eval!(context, limbi_high_col5),
    ];
    let numerator_11 = eval!(context, -(10));
    acc.add_to_relation(context, numerator_11, tuple_11);

    // Use RangeCheck_16.
    let tuple_12 = &[eval!(context, 1008385708), eval!(context, limbi_low_col6)];
    let numerator_12 = eval!(context, 1);
    acc.add_to_relation(context, numerator_12, tuple_12);

    // Use RangeCheck_15.
    let tuple_13 = &[eval!(context, 1058718565), eval!(context, limbi_high_col7)];
    let numerator_13 = eval!(context, 1);
    acc.add_to_relation(context, numerator_13, tuple_13);

    //limb 3 reconstruction.
    let constraint_14_value = eval!(
        context,
        (qm_31_into_u_32_input_limb_3) - ((limbi_low_col6) + ((limbi_high_col7) * (65536)))
    );
    acc.add_constraint(context, constraint_14_value);

    // Yield BlakeMessage.
    let tuple_15 = &[
        eval!(context, 1492981981),
        eval!(context, qm_31_into_u_32_input_limb_16),
        eval!(context, 3),
        eval!(context, limbi_low_col6),
        eval!(context, limbi_high_col7),
    ];
    let numerator_15 = eval!(context, -(10));
    acc.add_to_relation(context, numerator_15, tuple_15);

    // Use RangeCheck_16.
    let tuple_16 = &[eval!(context, 1008385708), eval!(context, limbi_low_col8)];
    let numerator_16 = eval!(context, 1);
    acc.add_to_relation(context, numerator_16, tuple_16);

    // Use RangeCheck_15.
    let tuple_17 = &[eval!(context, 1058718565), eval!(context, limbi_high_col9)];
    let numerator_17 = eval!(context, 1);
    acc.add_to_relation(context, numerator_17, tuple_17);

    //limb 4 reconstruction.
    let constraint_18_value = eval!(
        context,
        (qm_31_into_u_32_input_limb_4) - ((limbi_low_col8) + ((limbi_high_col9) * (65536)))
    );
    acc.add_constraint(context, constraint_18_value);

    // Yield BlakeMessage.
    let tuple_19 = &[
        eval!(context, 1492981981),
        eval!(context, qm_31_into_u_32_input_limb_16),
        eval!(context, 4),
        eval!(context, limbi_low_col8),
        eval!(context, limbi_high_col9),
    ];
    let numerator_19 = eval!(context, -(10));
    acc.add_to_relation(context, numerator_19, tuple_19);

    // Use RangeCheck_16.
    let tuple_20 = &[eval!(context, 1008385708), eval!(context, limbi_low_col10)];
    let numerator_20 = eval!(context, 1);
    acc.add_to_relation(context, numerator_20, tuple_20);

    // Use RangeCheck_15.
    let tuple_21 = &[eval!(context, 1058718565), eval!(context, limbi_high_col11)];
    let numerator_21 = eval!(context, 1);
    acc.add_to_relation(context, numerator_21, tuple_21);

    //limb 5 reconstruction.
    let constraint_22_value = eval!(
        context,
        (qm_31_into_u_32_input_limb_5) - ((limbi_low_col10) + ((limbi_high_col11) * (65536)))
    );
    acc.add_constraint(context, constraint_22_value);

    // Yield BlakeMessage.
    let tuple_23 = &[
        eval!(context, 1492981981),
        eval!(context, qm_31_into_u_32_input_limb_16),
        eval!(context, 5),
        eval!(context, limbi_low_col10),
        eval!(context, limbi_high_col11),
    ];
    let numerator_23 = eval!(context, -(10));
    acc.add_to_relation(context, numerator_23, tuple_23);

    // Use RangeCheck_16.
    let tuple_24 = &[eval!(context, 1008385708), eval!(context, limbi_low_col12)];
    let numerator_24 = eval!(context, 1);
    acc.add_to_relation(context, numerator_24, tuple_24);

    // Use RangeCheck_15.
    let tuple_25 = &[eval!(context, 1058718565), eval!(context, limbi_high_col13)];
    let numerator_25 = eval!(context, 1);
    acc.add_to_relation(context, numerator_25, tuple_25);

    //limb 6 reconstruction.
    let constraint_26_value = eval!(
        context,
        (qm_31_into_u_32_input_limb_6) - ((limbi_low_col12) + ((limbi_high_col13) * (65536)))
    );
    acc.add_constraint(context, constraint_26_value);

    // Yield BlakeMessage.
    let tuple_27 = &[
        eval!(context, 1492981981),
        eval!(context, qm_31_into_u_32_input_limb_16),
        eval!(context, 6),
        eval!(context, limbi_low_col12),
        eval!(context, limbi_high_col13),
    ];
    let numerator_27 = eval!(context, -(10));
    acc.add_to_relation(context, numerator_27, tuple_27);

    // Use RangeCheck_16.
    let tuple_28 = &[eval!(context, 1008385708), eval!(context, limbi_low_col14)];
    let numerator_28 = eval!(context, 1);
    acc.add_to_relation(context, numerator_28, tuple_28);

    // Use RangeCheck_15.
    let tuple_29 = &[eval!(context, 1058718565), eval!(context, limbi_high_col15)];
    let numerator_29 = eval!(context, 1);
    acc.add_to_relation(context, numerator_29, tuple_29);

    //limb 7 reconstruction.
    let constraint_30_value = eval!(
        context,
        (qm_31_into_u_32_input_limb_7) - ((limbi_low_col14) + ((limbi_high_col15) * (65536)))
    );
    acc.add_constraint(context, constraint_30_value);

    // Yield BlakeMessage.
    let tuple_31 = &[
        eval!(context, 1492981981),
        eval!(context, qm_31_into_u_32_input_limb_16),
        eval!(context, 7),
        eval!(context, limbi_low_col14),
        eval!(context, limbi_high_col15),
    ];
    let numerator_31 = eval!(context, -(10));
    acc.add_to_relation(context, numerator_31, tuple_31);

    // Use RangeCheck_16.
    let tuple_32 = &[eval!(context, 1008385708), eval!(context, limbi_low_col16)];
    let numerator_32 = eval!(context, 1);
    acc.add_to_relation(context, numerator_32, tuple_32);

    // Use RangeCheck_15.
    let tuple_33 = &[eval!(context, 1058718565), eval!(context, limbi_high_col17)];
    let numerator_33 = eval!(context, 1);
    acc.add_to_relation(context, numerator_33, tuple_33);

    //limb 8 reconstruction.
    let constraint_34_value = eval!(
        context,
        (qm_31_into_u_32_input_limb_8) - ((limbi_low_col16) + ((limbi_high_col17) * (65536)))
    );
    acc.add_constraint(context, constraint_34_value);

    // Yield BlakeMessage.
    let tuple_35 = &[
        eval!(context, 1492981981),
        eval!(context, qm_31_into_u_32_input_limb_16),
        eval!(context, 8),
        eval!(context, limbi_low_col16),
        eval!(context, limbi_high_col17),
    ];
    let numerator_35 = eval!(context, -(10));
    acc.add_to_relation(context, numerator_35, tuple_35);

    // Use RangeCheck_16.
    let tuple_36 = &[eval!(context, 1008385708), eval!(context, limbi_low_col18)];
    let numerator_36 = eval!(context, 1);
    acc.add_to_relation(context, numerator_36, tuple_36);

    // Use RangeCheck_15.
    let tuple_37 = &[eval!(context, 1058718565), eval!(context, limbi_high_col19)];
    let numerator_37 = eval!(context, 1);
    acc.add_to_relation(context, numerator_37, tuple_37);

    //limb 9 reconstruction.
    let constraint_38_value = eval!(
        context,
        (qm_31_into_u_32_input_limb_9) - ((limbi_low_col18) + ((limbi_high_col19) * (65536)))
    );
    acc.add_constraint(context, constraint_38_value);

    // Yield BlakeMessage.
    let tuple_39 = &[
        eval!(context, 1492981981),
        eval!(context, qm_31_into_u_32_input_limb_16),
        eval!(context, 9),
        eval!(context, limbi_low_col18),
        eval!(context, limbi_high_col19),
    ];
    let numerator_39 = eval!(context, -(10));
    acc.add_to_relation(context, numerator_39, tuple_39);

    // Use RangeCheck_16.
    let tuple_40 = &[eval!(context, 1008385708), eval!(context, limbi_low_col20)];
    let numerator_40 = eval!(context, 1);
    acc.add_to_relation(context, numerator_40, tuple_40);

    // Use RangeCheck_15.
    let tuple_41 = &[eval!(context, 1058718565), eval!(context, limbi_high_col21)];
    let numerator_41 = eval!(context, 1);
    acc.add_to_relation(context, numerator_41, tuple_41);

    //limb 10 reconstruction.
    let constraint_42_value = eval!(
        context,
        (qm_31_into_u_32_input_limb_10) - ((limbi_low_col20) + ((limbi_high_col21) * (65536)))
    );
    acc.add_constraint(context, constraint_42_value);

    // Yield BlakeMessage.
    let tuple_43 = &[
        eval!(context, 1492981981),
        eval!(context, qm_31_into_u_32_input_limb_16),
        eval!(context, 10),
        eval!(context, limbi_low_col20),
        eval!(context, limbi_high_col21),
    ];
    let numerator_43 = eval!(context, -(10));
    acc.add_to_relation(context, numerator_43, tuple_43);

    // Use RangeCheck_16.
    let tuple_44 = &[eval!(context, 1008385708), eval!(context, limbi_low_col22)];
    let numerator_44 = eval!(context, 1);
    acc.add_to_relation(context, numerator_44, tuple_44);

    // Use RangeCheck_15.
    let tuple_45 = &[eval!(context, 1058718565), eval!(context, limbi_high_col23)];
    let numerator_45 = eval!(context, 1);
    acc.add_to_relation(context, numerator_45, tuple_45);

    //limb 11 reconstruction.
    let constraint_46_value = eval!(
        context,
        (qm_31_into_u_32_input_limb_11) - ((limbi_low_col22) + ((limbi_high_col23) * (65536)))
    );
    acc.add_constraint(context, constraint_46_value);

    // Yield BlakeMessage.
    let tuple_47 = &[
        eval!(context, 1492981981),
        eval!(context, qm_31_into_u_32_input_limb_16),
        eval!(context, 11),
        eval!(context, limbi_low_col22),
        eval!(context, limbi_high_col23),
    ];
    let numerator_47 = eval!(context, -(10));
    acc.add_to_relation(context, numerator_47, tuple_47);

    // Use RangeCheck_16.
    let tuple_48 = &[eval!(context, 1008385708), eval!(context, limbi_low_col24)];
    let numerator_48 = eval!(context, 1);
    acc.add_to_relation(context, numerator_48, tuple_48);

    // Use RangeCheck_15.
    let tuple_49 = &[eval!(context, 1058718565), eval!(context, limbi_high_col25)];
    let numerator_49 = eval!(context, 1);
    acc.add_to_relation(context, numerator_49, tuple_49);

    //limb 12 reconstruction.
    let constraint_50_value = eval!(
        context,
        (qm_31_into_u_32_input_limb_12) - ((limbi_low_col24) + ((limbi_high_col25) * (65536)))
    );
    acc.add_constraint(context, constraint_50_value);

    // Yield BlakeMessage.
    let tuple_51 = &[
        eval!(context, 1492981981),
        eval!(context, qm_31_into_u_32_input_limb_16),
        eval!(context, 12),
        eval!(context, limbi_low_col24),
        eval!(context, limbi_high_col25),
    ];
    let numerator_51 = eval!(context, -(10));
    acc.add_to_relation(context, numerator_51, tuple_51);

    // Use RangeCheck_16.
    let tuple_52 = &[eval!(context, 1008385708), eval!(context, limbi_low_col26)];
    let numerator_52 = eval!(context, 1);
    acc.add_to_relation(context, numerator_52, tuple_52);

    // Use RangeCheck_15.
    let tuple_53 = &[eval!(context, 1058718565), eval!(context, limbi_high_col27)];
    let numerator_53 = eval!(context, 1);
    acc.add_to_relation(context, numerator_53, tuple_53);

    //limb 13 reconstruction.
    let constraint_54_value = eval!(
        context,
        (qm_31_into_u_32_input_limb_13) - ((limbi_low_col26) + ((limbi_high_col27) * (65536)))
    );
    acc.add_constraint(context, constraint_54_value);

    // Yield BlakeMessage.
    let tuple_55 = &[
        eval!(context, 1492981981),
        eval!(context, qm_31_into_u_32_input_limb_16),
        eval!(context, 13),
        eval!(context, limbi_low_col26),
        eval!(context, limbi_high_col27),
    ];
    let numerator_55 = eval!(context, -(10));
    acc.add_to_relation(context, numerator_55, tuple_55);

    // Use RangeCheck_16.
    let tuple_56 = &[eval!(context, 1008385708), eval!(context, limbi_low_col28)];
    let numerator_56 = eval!(context, 1);
    acc.add_to_relation(context, numerator_56, tuple_56);

    // Use RangeCheck_15.
    let tuple_57 = &[eval!(context, 1058718565), eval!(context, limbi_high_col29)];
    let numerator_57 = eval!(context, 1);
    acc.add_to_relation(context, numerator_57, tuple_57);

    //limb 14 reconstruction.
    let constraint_58_value = eval!(
        context,
        (qm_31_into_u_32_input_limb_14) - ((limbi_low_col28) + ((limbi_high_col29) * (65536)))
    );
    acc.add_constraint(context, constraint_58_value);

    // Yield BlakeMessage.
    let tuple_59 = &[
        eval!(context, 1492981981),
        eval!(context, qm_31_into_u_32_input_limb_16),
        eval!(context, 14),
        eval!(context, limbi_low_col28),
        eval!(context, limbi_high_col29),
    ];
    let numerator_59 = eval!(context, -(10));
    acc.add_to_relation(context, numerator_59, tuple_59);

    // Use RangeCheck_16.
    let tuple_60 = &[eval!(context, 1008385708), eval!(context, limbi_low_col30)];
    let numerator_60 = eval!(context, 1);
    acc.add_to_relation(context, numerator_60, tuple_60);

    // Use RangeCheck_15.
    let tuple_61 = &[eval!(context, 1058718565), eval!(context, limbi_high_col31)];
    let numerator_61 = eval!(context, 1);
    acc.add_to_relation(context, numerator_61, tuple_61);

    //limb 15 reconstruction.
    let constraint_62_value = eval!(
        context,
        (qm_31_into_u_32_input_limb_15) - ((limbi_low_col30) + ((limbi_high_col31) * (65536)))
    );
    acc.add_constraint(context, constraint_62_value);

    // Yield BlakeMessage.
    let tuple_63 = &[
        eval!(context, 1492981981),
        eval!(context, qm_31_into_u_32_input_limb_16),
        eval!(context, 15),
        eval!(context, limbi_low_col30),
        eval!(context, limbi_high_col31),
    ];
    let numerator_63 = eval!(context, -(10));
    acc.add_to_relation(context, numerator_63, tuple_63);
    vec![]
}
