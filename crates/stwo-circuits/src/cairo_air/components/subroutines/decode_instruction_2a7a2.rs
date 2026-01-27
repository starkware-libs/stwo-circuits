// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

#[allow(unused_variables)]
pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let _ = component_data;
    let _ = acc;
    let [decode_instruction_2a7a2_input_pc] = input.try_into().unwrap();

    // Use VerifyInstruction.
    let tuple_0 = &[
        eval!(context, 1719106205),
        eval!(context, decode_instruction_2a7a2_input_pc),
        eval!(context, 32768),
        eval!(context, 32769),
        eval!(context, 32769),
        eval!(context, 32),
        eval!(context, 68),
    ];
    let numerator_0 = eval!(context, 1);
    acc.add_to_relation(context, numerator_0, tuple_0);
    vec![]
}
