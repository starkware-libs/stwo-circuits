// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] =
    [RelationUse { relation_id: "VerifyInstruction", uses: 1 }];

#[allow(unused_variables)]
pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let _ = component_data;
    let _ = acc;
    let [
        decode_instruction_9bd86_input_pc,
        offset1_col0,
        offset2_col1,
        op0_base_fp_col2,
        ap_update_add_1_col3,
    ] = input.try_into().unwrap();

    //Flag op0_base_fp is a bit.
    let constraint_0_value = eval!(context, (op0_base_fp_col2) * ((1) - (op0_base_fp_col2)));
    acc.add_constraint(context, constraint_0_value);

    //Flag ap_update_add_1 is a bit.
    let constraint_1_value =
        eval!(context, (ap_update_add_1_col3) * ((1) - (ap_update_add_1_col3)));
    acc.add_constraint(context, constraint_1_value);

    // Use VerifyInstruction.
    let tuple_2 = &[
        eval!(context, 1719106205),
        eval!(context, decode_instruction_9bd86_input_pc),
        eval!(context, 32767),
        eval!(context, offset1_col0),
        eval!(context, offset2_col1),
        eval!(context, (8) + ((op0_base_fp_col2) * (16))),
        eval!(context, (2) + ((ap_update_add_1_col3) * (32))),
    ];
    let numerator_2 = eval!(context, 1);
    acc.add_to_relation(context, numerator_2, tuple_2);
    vec![eval!(context, (offset1_col0) - (32768)), eval!(context, (offset2_col1) - (32768))]
}
