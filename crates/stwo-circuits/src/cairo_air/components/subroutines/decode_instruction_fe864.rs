// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] =
    [RelationUse { relation_id: "VerifyInstruction", uses: 1 }];

pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let _ = component_data;
    let [
        decode_instruction_fe864_input_pc,
        offset0_col0,
        offset2_col1,
        dst_base_fp_col2,
        op1_base_fp_col3,
        ap_update_add_1_col4,
    ] = input.try_into().unwrap();

    //Flag dst_base_fp is a bit.
    let constraint_0_value = eval!(context, (dst_base_fp_col2) * ((1) - (dst_base_fp_col2)));
    acc.add_constraint(context, constraint_0_value);

    //Flag op1_base_fp is a bit.
    let constraint_1_value = eval!(context, (op1_base_fp_col3) * ((1) - (op1_base_fp_col3)));
    acc.add_constraint(context, constraint_1_value);

    //Flag ap_update_add_1 is a bit.
    let constraint_2_value =
        eval!(context, (ap_update_add_1_col4) * ((1) - (ap_update_add_1_col4)));
    acc.add_constraint(context, constraint_2_value);

    // Use VerifyInstruction.
    let tuple_3 = &[
        eval!(context, 1719106205),
        eval!(context, decode_instruction_fe864_input_pc),
        eval!(context, offset0_col0),
        eval!(context, 32767),
        eval!(context, offset2_col1),
        eval!(
            context,
            ((((dst_base_fp_col2) * (8)) + (16)) + ((op1_base_fp_col3) * (64)))
                + (((1) - (op1_base_fp_col3)) * (128))
        ),
        eval!(context, ((ap_update_add_1_col4) * (32)) + (256)),
    ];
    let numerator_3 = eval!(context, 1);
    acc.add_to_relation(context, numerator_3, tuple_3);
    vec![
        eval!(context, (offset0_col0) - (32768)),
        eval!(context, (offset2_col1) - (32768)),
        eval!(context, (1) - (op1_base_fp_col3)),
    ]
}
