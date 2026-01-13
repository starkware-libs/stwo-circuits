// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    acc: &mut CompositionConstraintAccumulator<'_>,
) -> Vec<Var> {
    let [cond_range_check_2_input_limb_0, cond_range_check_2_input_limb_1, partial_limb_msb_col0] =
        input.try_into().unwrap();

    //msb is a bit or condition is 0.
    let constraint_0_value = eval!(
        context,
        ((partial_limb_msb_col0) * ((1) - (partial_limb_msb_col0)))
            * (cond_range_check_2_input_limb_1)
    );
    acc.add_constraint(context, constraint_0_value);

    let partial_limb_bit_before_msb_tmp_88401_1 =
        eval!(context, (cond_range_check_2_input_limb_0) - ((partial_limb_msb_col0) * (2)));

    //bit before msb is a bit or condition is 0.
    let constraint_2_value = eval!(
        context,
        ((partial_limb_bit_before_msb_tmp_88401_1)
            * ((1) - (partial_limb_bit_before_msb_tmp_88401_1)))
            * (cond_range_check_2_input_limb_1)
    );
    acc.add_constraint(context, constraint_2_value);
    vec![]
}
