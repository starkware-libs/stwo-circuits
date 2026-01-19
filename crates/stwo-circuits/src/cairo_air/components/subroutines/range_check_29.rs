// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let _ = component_data;
    let [range_check_29_input, range_check_29_bot11bits_col0] = input.try_into().unwrap();

    // Use RangeCheck_18.
    let tuple_0 = &[
        eval!(context, 1109051422),
        eval!(context, ((range_check_29_input) - (range_check_29_bot11bits_col0)) * (1048576)),
    ];
    let numerator_0 = eval!(context, 1);
    acc.add_to_relation(context, numerator_0, tuple_0);

    // Use RangeCheck_11.
    let tuple_1 = &[eval!(context, 991608089), eval!(context, range_check_29_bot11bits_col0)];
    let numerator_1 = eval!(context, 1);
    acc.add_to_relation(context, numerator_1, tuple_1);
    vec![]
}
