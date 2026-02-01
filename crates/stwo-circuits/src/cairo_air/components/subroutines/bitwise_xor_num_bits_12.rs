// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] =
    [RelationUse { relation_id: "VerifyBitwiseXor_12", uses: 1 }];

#[allow(unused_variables)]
pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let _ = component_data;
    let _ = acc;
    let [bitwise_xor_num_bits_12_input_limb_0, bitwise_xor_num_bits_12_input_limb_1, xor_col0] =
        input.try_into().unwrap();

    // Use VerifyBitwiseXor_12.
    let tuple_0 = &[
        eval!(context, 648362599),
        eval!(context, bitwise_xor_num_bits_12_input_limb_0),
        eval!(context, bitwise_xor_num_bits_12_input_limb_1),
        eval!(context, xor_col0),
    ];
    let numerator_0 = eval!(context, 1);
    acc.add_to_relation(context, numerator_0, tuple_0);
    vec![]
}
