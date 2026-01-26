// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 24;
pub const N_INTERACTION_COLUMNS: usize = 20;

pub const RELATION_USES_PER_ROW: [RelationUse; 4] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 3 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 3 },
    RelationUse { relation_id: "Opcodes", uses: 1 },
    RelationUse { relation_id: "VerifyInstruction", uses: 1 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let _ = component_data;
    let _ = acc;
    let [
        input_pc_col0,
        input_ap_col1,
        input_fp_col2,
        stored_fp_id_col3,
        stored_fp_limb_0_col4,
        stored_fp_limb_1_col5,
        stored_fp_limb_2_col6,
        stored_fp_limb_3_col7,
        partial_limb_msb_col8,
        stored_ret_pc_id_col9,
        stored_ret_pc_limb_0_col10,
        stored_ret_pc_limb_1_col11,
        stored_ret_pc_limb_2_col12,
        stored_ret_pc_limb_3_col13,
        partial_limb_msb_col14,
        distance_to_next_pc_id_col15,
        msb_col16,
        mid_limbs_set_col17,
        distance_to_next_pc_limb_0_col18,
        distance_to_next_pc_limb_1_col19,
        distance_to_next_pc_limb_2_col20,
        remainder_bits_col21,
        partial_limb_msb_col22,
        enabler,
    ] = input.try_into().unwrap();
    let enabler_constraint_value = eval!(context, ((enabler) * (enabler)) - (enabler));
    acc.add_constraint(context, enabler_constraint_value);

    decode_instruction_2a7a2::accumulate_constraints(
        &[eval!(context, input_pc_col0)],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_29::accumulate_constraints(
        &[
            eval!(context, input_ap_col1),
            eval!(context, stored_fp_id_col3),
            eval!(context, stored_fp_limb_0_col4),
            eval!(context, stored_fp_limb_1_col5),
            eval!(context, stored_fp_limb_2_col6),
            eval!(context, stored_fp_limb_3_col7),
            eval!(context, partial_limb_msb_col8),
        ],
        context,
        component_data,
        acc,
    );

    //[ap] = fp.
    let constraint_2_value = eval!(
        context,
        ((((stored_fp_limb_0_col4) + ((stored_fp_limb_1_col5) * (512)))
            + ((stored_fp_limb_2_col6) * (262144)))
            + ((stored_fp_limb_3_col7) * (134217728)))
            - (input_fp_col2)
    );
    acc.add_constraint(context, constraint_2_value);

    read_positive_num_bits_29::accumulate_constraints(
        &[
            eval!(context, (input_ap_col1) + (1)),
            eval!(context, stored_ret_pc_id_col9),
            eval!(context, stored_ret_pc_limb_0_col10),
            eval!(context, stored_ret_pc_limb_1_col11),
            eval!(context, stored_ret_pc_limb_2_col12),
            eval!(context, stored_ret_pc_limb_3_col13),
            eval!(context, partial_limb_msb_col14),
        ],
        context,
        component_data,
        acc,
    );

    //[ap+1] = return_pc.
    let constraint_4_value = eval!(
        context,
        ((((stored_ret_pc_limb_0_col10) + ((stored_ret_pc_limb_1_col11) * (512)))
            + ((stored_ret_pc_limb_2_col12) * (262144)))
            + ((stored_ret_pc_limb_3_col13) * (134217728)))
            - ((input_pc_col0) + (2))
    );
    acc.add_constraint(context, constraint_4_value);

    let [read_small_output_tmp_9db06_26_limb_0] = read_small::accumulate_constraints(
        &[
            eval!(context, (input_pc_col0) + (1)),
            eval!(context, distance_to_next_pc_id_col15),
            eval!(context, msb_col16),
            eval!(context, mid_limbs_set_col17),
            eval!(context, distance_to_next_pc_limb_0_col18),
            eval!(context, distance_to_next_pc_limb_1_col19),
            eval!(context, distance_to_next_pc_limb_2_col20),
            eval!(context, remainder_bits_col21),
            eval!(context, partial_limb_msb_col22),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    // Use Opcodes.
    let tuple_6 = &[
        eval!(context, 428564188),
        eval!(context, input_pc_col0),
        eval!(context, input_ap_col1),
        eval!(context, input_fp_col2),
    ];
    let numerator_6 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_6, tuple_6);

    // Yield Opcodes.
    let tuple_7 = &[
        eval!(context, 428564188),
        eval!(context, (input_pc_col0) + (read_small_output_tmp_9db06_26_limb_0)),
        eval!(context, (input_ap_col1) + (2)),
        eval!(context, (input_ap_col1) + (2)),
    ];
    let numerator_7 = eval!(context, -(enabler));
    acc.add_to_relation(context, numerator_7, tuple_7);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &ComponentData<'_>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        accumulate_constraints(component_data.trace_columns, context, component_data, acc);
    }

    fn trace_columns(&self) -> usize {
        N_TRACE_COLUMNS
    }

    fn interaction_columns(&self) -> usize {
        N_INTERACTION_COLUMNS
    }

    fn relation_uses_per_row(&self) -> &[RelationUse] {
        &RELATION_USES_PER_ROW
    }
}
