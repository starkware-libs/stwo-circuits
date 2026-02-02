// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 19;
pub const N_INTERACTION_COLUMNS: usize = 16;

pub const RELATION_USES_PER_ROW: [RelationUse; 4] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 3 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 1 },
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
        offset0_col3,
        offset1_col4,
        offset2_col5,
        dst_base_fp_col6,
        op0_base_fp_col7,
        ap_update_add_1_col8,
        mem_dst_base_col9,
        mem0_base_col10,
        mem1_base_id_col11,
        mem1_base_limb_0_col12,
        mem1_base_limb_1_col13,
        mem1_base_limb_2_col14,
        mem1_base_limb_3_col15,
        partial_limb_msb_col16,
        dst_id_col17,
        enabler,
    ] = input.try_into().unwrap();
    let enabler_constraint_value = eval!(context, ((enabler) * (enabler)) - (enabler));
    acc.add_constraint(context, enabler_constraint_value);

    let [
        decode_instruction_cb32b_output_tmp_b1151_8_offset0,
        decode_instruction_cb32b_output_tmp_b1151_8_offset1,
        decode_instruction_cb32b_output_tmp_b1151_8_offset2,
    ] = decode_instruction_cb32b::accumulate_constraints(
        &[
            eval!(context, input_pc_col0),
            eval!(context, offset0_col3),
            eval!(context, offset1_col4),
            eval!(context, offset2_col5),
            eval!(context, dst_base_fp_col6),
            eval!(context, op0_base_fp_col7),
            eval!(context, ap_update_add_1_col8),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    //mem_dst_base.
    let constraint_1_value = eval!(
        context,
        (mem_dst_base_col9)
            - (((dst_base_fp_col6) * (input_fp_col2))
                + (((1) - (dst_base_fp_col6)) * (input_ap_col1)))
    );
    acc.add_constraint(context, constraint_1_value);

    //mem0_base.
    let constraint_2_value = eval!(
        context,
        (mem0_base_col10)
            - (((op0_base_fp_col7) * (input_fp_col2))
                + (((1) - (op0_base_fp_col7)) * (input_ap_col1)))
    );
    acc.add_constraint(context, constraint_2_value);

    read_positive_num_bits_29::accumulate_constraints(
        &[
            eval!(
                context,
                (mem0_base_col10) + (decode_instruction_cb32b_output_tmp_b1151_8_offset1)
            ),
            eval!(context, mem1_base_id_col11),
            eval!(context, mem1_base_limb_0_col12),
            eval!(context, mem1_base_limb_1_col13),
            eval!(context, mem1_base_limb_2_col14),
            eval!(context, mem1_base_limb_3_col15),
            eval!(context, partial_limb_msb_col16),
        ],
        context,
        component_data,
        acc,
    );

    mem_verify_equal::accumulate_constraints(
        &[
            eval!(
                context,
                (mem_dst_base_col9) + (decode_instruction_cb32b_output_tmp_b1151_8_offset0)
            ),
            eval!(
                context,
                ((((mem1_base_limb_0_col12) + ((mem1_base_limb_1_col13) * (512)))
                    + ((mem1_base_limb_2_col14) * (262144)))
                    + ((mem1_base_limb_3_col15) * (134217728)))
                    + (decode_instruction_cb32b_output_tmp_b1151_8_offset2)
            ),
            eval!(context, dst_id_col17),
        ],
        context,
        component_data,
        acc,
    );

    // Use Opcodes.
    let tuple_5 = &[
        eval!(context, 428564188),
        eval!(context, input_pc_col0),
        eval!(context, input_ap_col1),
        eval!(context, input_fp_col2),
    ];
    let numerator_5 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_5, tuple_5);

    // Yield Opcodes.
    let tuple_6 = &[
        eval!(context, 428564188),
        eval!(context, (input_pc_col0) + (1)),
        eval!(context, (input_ap_col1) + (ap_update_add_1_col8)),
        eval!(context, input_fp_col2),
    ];
    let numerator_6 = eval!(context, -(enabler));
    acc.add_to_relation(context, numerator_6, tuple_6);
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
