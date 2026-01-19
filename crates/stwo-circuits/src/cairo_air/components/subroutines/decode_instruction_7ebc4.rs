// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let _ = component_data;
    let [decode_instruction_7ebc4_input_pc, ap_update_add_1_col0] = input.try_into().unwrap();

    //Flag ap_update_add_1 is a bit.
    let constraint_0_value =
        eval!(context, (ap_update_add_1_col0) * ((1) - (ap_update_add_1_col0)));
    acc.add_constraint(context, constraint_0_value);

    // Use VerifyInstruction.
    let tuple_1 = &[
        eval!(context, 1719106205),
        eval!(context, decode_instruction_7ebc4_input_pc),
        eval!(context, 32767),
        eval!(context, 32767),
        eval!(context, 32769),
        eval!(context, 56),
        eval!(context, (4) + ((ap_update_add_1_col0) * (32))),
    ];
    let numerator_1 = eval!(context, 1);
    acc.add_to_relation(context, numerator_1, tuple_1);
    vec![]
}
