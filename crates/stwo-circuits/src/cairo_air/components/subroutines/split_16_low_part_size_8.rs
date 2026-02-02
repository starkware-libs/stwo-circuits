// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 0] = [];

#[allow(unused_variables)]
pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [split_16_low_part_size_8_input, ms_8_bits_col0] = input.try_into().unwrap();
    vec![eval!(context, (split_16_low_part_size_8_input) - ((ms_8_bits_col0) * (256)))]
}
