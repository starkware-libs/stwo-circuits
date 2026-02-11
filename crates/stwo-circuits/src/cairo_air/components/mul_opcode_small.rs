// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 37;
pub const N_INTERACTION_COLUMNS: usize = 24;

pub const RELATION_USES_PER_ROW: [RelationUse; 5] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 3 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 3 },
    RelationUse { relation_id: "Opcodes", uses: 1 },
    RelationUse { relation_id: "RangeCheck_11", uses: 3 },
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
        offset0_col3,
        offset1_col4,
        offset2_col5,
        dst_base_fp_col6,
        op0_base_fp_col7,
        op1_imm_col8,
        op1_base_fp_col9,
        ap_update_add_1_col10,
        mem_dst_base_col11,
        mem0_base_col12,
        mem1_base_col13,
        dst_id_col14,
        dst_limb_0_col15,
        dst_limb_1_col16,
        dst_limb_2_col17,
        dst_limb_3_col18,
        dst_limb_4_col19,
        dst_limb_5_col20,
        dst_limb_6_col21,
        dst_limb_7_col22,
        op0_id_col23,
        op0_limb_0_col24,
        op0_limb_1_col25,
        op0_limb_2_col26,
        op0_limb_3_col27,
        op1_id_col28,
        op1_limb_0_col29,
        op1_limb_1_col30,
        op1_limb_2_col31,
        op1_limb_3_col32,
        carry_1_col33,
        carry_3_col34,
        carry_5_col35,
        enabler,
    ] = input.try_into().unwrap();
    let enabler_constraint_value = eval!(context, ((enabler) * (enabler)) - (enabler));
    acc.add_constraint(context, enabler_constraint_value);

    let [
        decode_instruction_4b8cf_output_tmp_9d1ad_11_offset0,
        decode_instruction_4b8cf_output_tmp_9d1ad_11_offset1,
        decode_instruction_4b8cf_output_tmp_9d1ad_11_offset2,
        decode_instruction_4b8cf_output_tmp_9d1ad_11_op1_base_ap,
    ] = decode_instruction_4b8cf::accumulate_constraints(
        &[
            eval!(context, input_pc_col0),
            eval!(context, offset0_col3),
            eval!(context, offset1_col4),
            eval!(context, offset2_col5),
            eval!(context, dst_base_fp_col6),
            eval!(context, op0_base_fp_col7),
            eval!(context, op1_imm_col8),
            eval!(context, op1_base_fp_col9),
            eval!(context, ap_update_add_1_col10),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    //if imm then offset2 is 1.
    let constraint_1_value = eval!(
        context,
        (op1_imm_col8) * ((1) - (decode_instruction_4b8cf_output_tmp_9d1ad_11_offset2))
    );
    acc.add_constraint(context, constraint_1_value);

    //mem_dst_base.
    let constraint_2_value = eval!(
        context,
        (mem_dst_base_col11)
            - (((dst_base_fp_col6) * (input_fp_col2))
                + (((1) - (dst_base_fp_col6)) * (input_ap_col1)))
    );
    acc.add_constraint(context, constraint_2_value);

    //mem0_base.
    let constraint_3_value = eval!(
        context,
        (mem0_base_col12)
            - (((op0_base_fp_col7) * (input_fp_col2))
                + (((1) - (op0_base_fp_col7)) * (input_ap_col1)))
    );
    acc.add_constraint(context, constraint_3_value);

    //mem1_base.
    let constraint_4_value = eval!(
        context,
        (mem1_base_col13)
            - ((((op1_imm_col8) * (input_pc_col0)) + ((op1_base_fp_col9) * (input_fp_col2)))
                + ((decode_instruction_4b8cf_output_tmp_9d1ad_11_op1_base_ap) * (input_ap_col1)))
    );
    acc.add_constraint(context, constraint_4_value);

    read_positive_num_bits_72::accumulate_constraints(
        &[
            eval!(
                context,
                (mem_dst_base_col11) + (decode_instruction_4b8cf_output_tmp_9d1ad_11_offset0)
            ),
            eval!(context, dst_id_col14),
            eval!(context, dst_limb_0_col15),
            eval!(context, dst_limb_1_col16),
            eval!(context, dst_limb_2_col17),
            eval!(context, dst_limb_3_col18),
            eval!(context, dst_limb_4_col19),
            eval!(context, dst_limb_5_col20),
            eval!(context, dst_limb_6_col21),
            eval!(context, dst_limb_7_col22),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_36::accumulate_constraints(
        &[
            eval!(
                context,
                (mem0_base_col12) + (decode_instruction_4b8cf_output_tmp_9d1ad_11_offset1)
            ),
            eval!(context, op0_id_col23),
            eval!(context, op0_limb_0_col24),
            eval!(context, op0_limb_1_col25),
            eval!(context, op0_limb_2_col26),
            eval!(context, op0_limb_3_col27),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_36::accumulate_constraints(
        &[
            eval!(
                context,
                (mem1_base_col13) + (decode_instruction_4b8cf_output_tmp_9d1ad_11_offset2)
            ),
            eval!(context, op1_id_col28),
            eval!(context, op1_limb_0_col29),
            eval!(context, op1_limb_1_col30),
            eval!(context, op1_limb_2_col31),
            eval!(context, op1_limb_3_col32),
        ],
        context,
        component_data,
        acc,
    );

    verify_mul_small::accumulate_constraints(
        &[
            eval!(context, op0_limb_0_col24),
            eval!(context, op0_limb_1_col25),
            eval!(context, op0_limb_2_col26),
            eval!(context, op0_limb_3_col27),
            eval!(context, op1_limb_0_col29),
            eval!(context, op1_limb_1_col30),
            eval!(context, op1_limb_2_col31),
            eval!(context, op1_limb_3_col32),
            eval!(context, dst_limb_0_col15),
            eval!(context, dst_limb_1_col16),
            eval!(context, dst_limb_2_col17),
            eval!(context, dst_limb_3_col18),
            eval!(context, dst_limb_4_col19),
            eval!(context, dst_limb_5_col20),
            eval!(context, dst_limb_6_col21),
            eval!(context, dst_limb_7_col22),
            eval!(context, carry_1_col33),
            eval!(context, carry_3_col34),
            eval!(context, carry_5_col35),
        ],
        context,
        component_data,
        acc,
    );

    // Use Opcodes.
    let tuple_9 = &[
        eval!(context, 428564188),
        eval!(context, input_pc_col0),
        eval!(context, input_ap_col1),
        eval!(context, input_fp_col2),
    ];
    let numerator_9 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_9, tuple_9);

    // Yield Opcodes.
    let tuple_10 = &[
        eval!(context, 428564188),
        eval!(context, ((input_pc_col0) + (1)) + (op1_imm_col8)),
        eval!(context, (input_ap_col1) + (ap_update_add_1_col10)),
        eval!(context, input_fp_col2),
    ];
    let numerator_10 = eval!(context, -(enabler));
    acc.add_to_relation(context, numerator_10, tuple_10);
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
