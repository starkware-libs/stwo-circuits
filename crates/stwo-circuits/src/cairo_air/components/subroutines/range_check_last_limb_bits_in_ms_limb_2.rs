// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let _ = component_data;
    let [range_check_last_limb_bits_in_ms_limb_2_input, partial_limb_msb_col0] =
        input.try_into().unwrap();

    cond_range_check_2::accumulate_constraints(
        &[
            eval!(context, range_check_last_limb_bits_in_ms_limb_2_input),
            eval!(context, 1),
            eval!(context, partial_limb_msb_col0),
        ],
        context,
        component_data,
        acc,
    );
    vec![]
}
