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
    let [decode_instruction_15a61_input_pc] = input.try_into().unwrap();

    // Use VerifyInstruction.
    let tuple_0 = &[
        eval!(context, 1719106205),
        eval!(context, decode_instruction_15a61_input_pc),
        eval!(context, 32766),
        eval!(context, 32767),
        eval!(context, 32767),
        eval!(context, 88),
        eval!(context, 130),
    ];
    let numerator_0 = eval!(context, 1);
    acc.add_to_relation(context, numerator_0, tuple_0);
    vec![]
}
