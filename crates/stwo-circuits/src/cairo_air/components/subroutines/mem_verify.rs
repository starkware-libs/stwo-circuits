// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    acc: &mut CompositionConstraintAccumulator<'_>,
) -> Vec<Var> {
    let [
        mem_verify_input_address,
        mem_verify_input_value_limb_0,
        mem_verify_input_value_limb_1,
        mem_verify_input_value_limb_2,
        mem_verify_input_value_limb_3,
        mem_verify_input_value_limb_4,
        mem_verify_input_value_limb_5,
        mem_verify_input_value_limb_6,
        mem_verify_input_value_limb_7,
        mem_verify_input_value_limb_8,
        mem_verify_input_value_limb_9,
        mem_verify_input_value_limb_10,
        mem_verify_input_value_limb_11,
        mem_verify_input_value_limb_12,
        mem_verify_input_value_limb_13,
        mem_verify_input_value_limb_14,
        mem_verify_input_value_limb_15,
        mem_verify_input_value_limb_16,
        mem_verify_input_value_limb_17,
        mem_verify_input_value_limb_18,
        mem_verify_input_value_limb_19,
        mem_verify_input_value_limb_20,
        mem_verify_input_value_limb_21,
        mem_verify_input_value_limb_22,
        mem_verify_input_value_limb_23,
        mem_verify_input_value_limb_24,
        mem_verify_input_value_limb_25,
        mem_verify_input_value_limb_26,
        mem_verify_input_value_limb_27,
        id_col0,
    ] = input.try_into().unwrap();

    read_id::accumulate_constraints(
        &[eval!(context, mem_verify_input_address), eval!(context, id_col0)],
        context,
        acc,
    );

    // Use MemoryIdToBig.
    let tuple_1 = &[
        eval!(context, 1662111297),
        eval!(context, id_col0),
        eval!(context, mem_verify_input_value_limb_0),
        eval!(context, mem_verify_input_value_limb_1),
        eval!(context, mem_verify_input_value_limb_2),
        eval!(context, mem_verify_input_value_limb_3),
        eval!(context, mem_verify_input_value_limb_4),
        eval!(context, mem_verify_input_value_limb_5),
        eval!(context, mem_verify_input_value_limb_6),
        eval!(context, mem_verify_input_value_limb_7),
        eval!(context, mem_verify_input_value_limb_8),
        eval!(context, mem_verify_input_value_limb_9),
        eval!(context, mem_verify_input_value_limb_10),
        eval!(context, mem_verify_input_value_limb_11),
        eval!(context, mem_verify_input_value_limb_12),
        eval!(context, mem_verify_input_value_limb_13),
        eval!(context, mem_verify_input_value_limb_14),
        eval!(context, mem_verify_input_value_limb_15),
        eval!(context, mem_verify_input_value_limb_16),
        eval!(context, mem_verify_input_value_limb_17),
        eval!(context, mem_verify_input_value_limb_18),
        eval!(context, mem_verify_input_value_limb_19),
        eval!(context, mem_verify_input_value_limb_20),
        eval!(context, mem_verify_input_value_limb_21),
        eval!(context, mem_verify_input_value_limb_22),
        eval!(context, mem_verify_input_value_limb_23),
        eval!(context, mem_verify_input_value_limb_24),
        eval!(context, mem_verify_input_value_limb_25),
        eval!(context, mem_verify_input_value_limb_26),
        eval!(context, mem_verify_input_value_limb_27),
    ];
    let numerator_1 = eval!(context, 1);
    acc.add_to_relation(context, numerator_1, tuple_1);
    vec![]
}
