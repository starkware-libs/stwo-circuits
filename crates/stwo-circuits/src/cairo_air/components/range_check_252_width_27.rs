// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 20;
pub const N_INTERACTION_COLUMNS: usize = 32;

pub const RELATION_USES_PER_ROW: [RelationUse; 7] = [
    RelationUse { relation_id: "RangeCheck_18", uses: 7 },
    RelationUse { relation_id: "RangeCheck_18_B", uses: 2 },
    RelationUse { relation_id: "RangeCheck_9_9", uses: 1 },
    RelationUse { relation_id: "RangeCheck_9_9_B", uses: 1 },
    RelationUse { relation_id: "RangeCheck_9_9_C", uses: 1 },
    RelationUse { relation_id: "RangeCheck_9_9_D", uses: 1 },
    RelationUse { relation_id: "RangeCheck_9_9_E", uses: 1 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [
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
        limb_0_high_part_col10,
        limb_1_low_part_col11,
        limb_2_high_part_col12,
        limb_3_low_part_col13,
        limb_4_high_part_col14,
        limb_5_low_part_col15,
        limb_6_high_part_col16,
        limb_7_low_part_col17,
        limb_8_high_part_col18,
        enabler,
    ] = input.try_into().unwrap();
    let enabler_constraint_value = eval!(context, ((enabler) * (enabler)) - (enabler));
    acc.add_constraint(context, enabler_constraint_value);

    // Use RangeCheck_9_9.
    let tuple_0 = &[
        eval!(context, 517791011),
        eval!(context, limb_0_high_part_col10),
        eval!(context, limb_1_low_part_col11),
    ];
    let numerator_0 = eval!(context, 1);
    acc.add_to_relation(context, numerator_0, tuple_0);

    // Use RangeCheck_18.
    let tuple_1 = &[
        eval!(context, 1109051422),
        eval!(context, (input_limb_0_col0) - ((limb_0_high_part_col10) * (262144))),
    ];
    let numerator_1 = eval!(context, 1);
    acc.add_to_relation(context, numerator_1, tuple_1);

    // Use RangeCheck_18.
    let tuple_2 = &[
        eval!(context, 1109051422),
        eval!(context, ((input_limb_1_col1) - (limb_1_low_part_col11)) * (4194304)),
    ];
    let numerator_2 = eval!(context, 1);
    acc.add_to_relation(context, numerator_2, tuple_2);

    // Use RangeCheck_9_9_B.
    let tuple_3 = &[
        eval!(context, 1897792095),
        eval!(context, limb_2_high_part_col12),
        eval!(context, limb_3_low_part_col13),
    ];
    let numerator_3 = eval!(context, 1);
    acc.add_to_relation(context, numerator_3, tuple_3);

    // Use RangeCheck_18_B.
    let tuple_4 = &[
        eval!(context, 1424798916),
        eval!(context, (input_limb_2_col2) - ((limb_2_high_part_col12) * (262144))),
    ];
    let numerator_4 = eval!(context, 1);
    acc.add_to_relation(context, numerator_4, tuple_4);

    // Use RangeCheck_18.
    let tuple_5 = &[
        eval!(context, 1109051422),
        eval!(context, ((input_limb_3_col3) - (limb_3_low_part_col13)) * (4194304)),
    ];
    let numerator_5 = eval!(context, 1);
    acc.add_to_relation(context, numerator_5, tuple_5);

    // Use RangeCheck_9_9_C.
    let tuple_6 = &[
        eval!(context, 1881014476),
        eval!(context, limb_4_high_part_col14),
        eval!(context, limb_5_low_part_col15),
    ];
    let numerator_6 = eval!(context, 1);
    acc.add_to_relation(context, numerator_6, tuple_6);

    // Use RangeCheck_18.
    let tuple_7 = &[
        eval!(context, 1109051422),
        eval!(context, (input_limb_4_col4) - ((limb_4_high_part_col14) * (262144))),
    ];
    let numerator_7 = eval!(context, 1);
    acc.add_to_relation(context, numerator_7, tuple_7);

    // Use RangeCheck_18.
    let tuple_8 = &[
        eval!(context, 1109051422),
        eval!(context, ((input_limb_5_col5) - (limb_5_low_part_col15)) * (4194304)),
    ];
    let numerator_8 = eval!(context, 1);
    acc.add_to_relation(context, numerator_8, tuple_8);

    // Use RangeCheck_9_9_D.
    let tuple_9 = &[
        eval!(context, 1864236857),
        eval!(context, limb_6_high_part_col16),
        eval!(context, limb_7_low_part_col17),
    ];
    let numerator_9 = eval!(context, 1);
    acc.add_to_relation(context, numerator_9, tuple_9);

    // Use RangeCheck_18_B.
    let tuple_10 = &[
        eval!(context, 1424798916),
        eval!(context, (input_limb_6_col6) - ((limb_6_high_part_col16) * (262144))),
    ];
    let numerator_10 = eval!(context, 1);
    acc.add_to_relation(context, numerator_10, tuple_10);

    // Use RangeCheck_18.
    let tuple_11 = &[
        eval!(context, 1109051422),
        eval!(context, ((input_limb_7_col7) - (limb_7_low_part_col17)) * (4194304)),
    ];
    let numerator_11 = eval!(context, 1);
    acc.add_to_relation(context, numerator_11, tuple_11);

    // Use RangeCheck_9_9_E.
    let tuple_12 = &[
        eval!(context, 1847459238),
        eval!(context, limb_8_high_part_col18),
        eval!(context, input_limb_9_col9),
    ];
    let numerator_12 = eval!(context, 1);
    acc.add_to_relation(context, numerator_12, tuple_12);

    // Use RangeCheck_18.
    let tuple_13 = &[
        eval!(context, 1109051422),
        eval!(context, (input_limb_8_col8) - ((limb_8_high_part_col18) * (262144))),
    ];
    let numerator_13 = eval!(context, 1);
    acc.add_to_relation(context, numerator_13, tuple_13);

    // Yield RangeCheck252Width27.
    let tuple_14 = &[
        eval!(context, 1090315331),
        eval!(context, input_limb_0_col0),
        eval!(context, input_limb_1_col1),
        eval!(context, input_limb_2_col2),
        eval!(context, input_limb_3_col3),
        eval!(context, input_limb_4_col4),
        eval!(context, input_limb_5_col5),
        eval!(context, input_limb_6_col6),
        eval!(context, input_limb_7_col7),
        eval!(context, input_limb_8_col8),
        eval!(context, input_limb_9_col9),
    ];
    let numerator_14 = eval!(context, -(enabler));
    acc.add_to_relation(context, numerator_14, tuple_14);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &ComponentData<'_>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        accumulate_constraints(component_data.trace_columns, context, component_data, acc);
    }

    fn trace_columns(&self) -> usize {
        N_TRACE_COLUMNS
    }

    fn interaction_columns(&self) -> usize {
        N_INTERACTION_COLUMNS
    }

    fn relation_uses_per_row(&self) -> &[RelationUse] {
        &RELATION_USES_PER_ROW
    }
}
