// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let _ = component_data;
    let [read_id_input, id_col0] = input.try_into().unwrap();

    // Use MemoryAddressToId.
    let tuple_0 =
        &[eval!(context, 1444891767), eval!(context, read_id_input), eval!(context, id_col0)];
    let numerator_0 = eval!(context, 1);
    acc.add_to_relation(context, numerator_0, tuple_0);
    vec![]
}
