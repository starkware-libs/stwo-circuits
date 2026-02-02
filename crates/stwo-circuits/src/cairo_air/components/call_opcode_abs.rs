// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 25;
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
    let [
        input_pc_col0,
        input_ap_col1,
        input_fp_col2,
        offset2_col3,
        op1_base_fp_col4,
        stored_fp_id_col5,
        stored_fp_limb_0_col6,
        stored_fp_limb_1_col7,
        stored_fp_limb_2_col8,
        stored_fp_limb_3_col9,
        partial_limb_msb_col10,
        stored_ret_pc_id_col11,
        stored_ret_pc_limb_0_col12,
        stored_ret_pc_limb_1_col13,
        stored_ret_pc_limb_2_col14,
        stored_ret_pc_limb_3_col15,
        partial_limb_msb_col16,
        mem1_base_col17,
        next_pc_id_col18,
        next_pc_limb_0_col19,
        next_pc_limb_1_col20,
        next_pc_limb_2_col21,
        next_pc_limb_3_col22,
        partial_limb_msb_col23,
        enabler,
    ] = input.try_into().unwrap();
    let enabler_constraint_value = eval!(context, ((enabler) * (enabler)) - (enabler));
    acc.add_constraint(context, enabler_constraint_value);

    let [
        decode_instruction_f1edd_output_tmp_32b66_4_offset2,
        decode_instruction_f1edd_output_tmp_32b66_4_op1_base_ap,
    ] = decode_instruction_f1edd::accumulate_constraints(
        &[
            eval!(context, input_pc_col0),
            eval!(context, offset2_col3),
            eval!(context, op1_base_fp_col4),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    read_positive_num_bits_29::accumulate_constraints(
        &[
            eval!(context, input_ap_col1),
            eval!(context, stored_fp_id_col5),
            eval!(context, stored_fp_limb_0_col6),
            eval!(context, stored_fp_limb_1_col7),
            eval!(context, stored_fp_limb_2_col8),
            eval!(context, stored_fp_limb_3_col9),
            eval!(context, partial_limb_msb_col10),
        ],
        context,
        component_data,
        acc,
    );

    //[ap] = fp.
    let constraint_2_value = eval!(
        context,
        ((((stored_fp_limb_0_col6) + ((stored_fp_limb_1_col7) * (512)))
            + ((stored_fp_limb_2_col8) * (262144)))
            + ((stored_fp_limb_3_col9) * (134217728)))
            - (input_fp_col2)
    );
    acc.add_constraint(context, constraint_2_value);

    read_positive_num_bits_29::accumulate_constraints(
        &[
            eval!(context, (input_ap_col1) + (1)),
            eval!(context, stored_ret_pc_id_col11),
            eval!(context, stored_ret_pc_limb_0_col12),
            eval!(context, stored_ret_pc_limb_1_col13),
            eval!(context, stored_ret_pc_limb_2_col14),
            eval!(context, stored_ret_pc_limb_3_col15),
            eval!(context, partial_limb_msb_col16),
        ],
        context,
        component_data,
        acc,
    );

    //[ap+1] = return_pc.
    let constraint_4_value = eval!(
        context,
        ((((stored_ret_pc_limb_0_col12) + ((stored_ret_pc_limb_1_col13) * (512)))
            + ((stored_ret_pc_limb_2_col14) * (262144)))
            + ((stored_ret_pc_limb_3_col15) * (134217728)))
            - ((input_pc_col0) + (1))
    );
    acc.add_constraint(context, constraint_4_value);

    //mem1_base.
    let constraint_5_value = eval!(
        context,
        (mem1_base_col17)
            - (((op1_base_fp_col4) * (input_fp_col2))
                + ((decode_instruction_f1edd_output_tmp_32b66_4_op1_base_ap) * (input_ap_col1)))
    );
    acc.add_constraint(context, constraint_5_value);

    read_positive_num_bits_29::accumulate_constraints(
        &[
            eval!(
                context,
                (mem1_base_col17) + (decode_instruction_f1edd_output_tmp_32b66_4_offset2)
            ),
            eval!(context, next_pc_id_col18),
            eval!(context, next_pc_limb_0_col19),
            eval!(context, next_pc_limb_1_col20),
            eval!(context, next_pc_limb_2_col21),
            eval!(context, next_pc_limb_3_col22),
            eval!(context, partial_limb_msb_col23),
        ],
        context,
        component_data,
        acc,
    );

    // Use Opcodes.
    let tuple_7 = &[
        eval!(context, 428564188),
        eval!(context, input_pc_col0),
        eval!(context, input_ap_col1),
        eval!(context, input_fp_col2),
    ];
    let numerator_7 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_7, tuple_7);

    // Yield Opcodes.
    let tuple_8 = &[
        eval!(context, 428564188),
        eval!(
            context,
            (((next_pc_limb_0_col19) + ((next_pc_limb_1_col20) * (512)))
                + ((next_pc_limb_2_col21) * (262144)))
                + ((next_pc_limb_3_col22) * (134217728))
        ),
        eval!(context, (input_ap_col1) + (2)),
        eval!(context, (input_ap_col1) + (2)),
    ];
    let numerator_8 = eval!(context, -(enabler));
    acc.add_to_relation(context, numerator_8, tuple_8);
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
