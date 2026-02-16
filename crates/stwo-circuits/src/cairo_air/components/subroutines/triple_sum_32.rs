// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 0] = [];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        triple_sum_32_input_a_limb_0,
        triple_sum_32_input_a_limb_1,
        triple_sum_32_input_b_limb_0,
        triple_sum_32_input_b_limb_1,
        triple_sum_32_input_c_limb_0,
        triple_sum_32_input_c_limb_1,
        triple_sum32_res_limb_0_col0,
        triple_sum32_res_limb_1_col1,
    ] = input.try_into().unwrap();

    let carry_low_tmp_541fa_1 = eval!(
        context,
        ((((triple_sum_32_input_a_limb_0) + (triple_sum_32_input_b_limb_0))
            + (triple_sum_32_input_c_limb_0))
            - (triple_sum32_res_limb_0_col0))
            * (32768)
    );

    //carry low is 0 or 1 or 2.
    let constraint_1_value = eval!(
        context,
        ((carry_low_tmp_541fa_1) * ((carry_low_tmp_541fa_1) - (1)))
            * ((carry_low_tmp_541fa_1) - (2))
    );
    acc.add_constraint(context, constraint_1_value);

    let carry_high_tmp_541fa_2 = eval!(
        context,
        (((((triple_sum_32_input_a_limb_1) + (triple_sum_32_input_b_limb_1))
            + (triple_sum_32_input_c_limb_1))
            + (carry_low_tmp_541fa_1))
            - (triple_sum32_res_limb_1_col1))
            * (32768)
    );

    //carry high is 0 or 1 or 2.
    let constraint_3_value = eval!(
        context,
        ((carry_high_tmp_541fa_2) * ((carry_high_tmp_541fa_2) - (1)))
            * ((carry_high_tmp_541fa_2) - (2))
    );
    acc.add_constraint(context, constraint_3_value);
    vec![]
}
