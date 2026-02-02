// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 39;
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
        msb_col15,
        mid_limbs_set_col16,
        dst_limb_0_col17,
        dst_limb_1_col18,
        dst_limb_2_col19,
        remainder_bits_col20,
        partial_limb_msb_col21,
        op0_id_col22,
        msb_col23,
        mid_limbs_set_col24,
        op0_limb_0_col25,
        op0_limb_1_col26,
        op0_limb_2_col27,
        remainder_bits_col28,
        partial_limb_msb_col29,
        op1_id_col30,
        msb_col31,
        mid_limbs_set_col32,
        op1_limb_0_col33,
        op1_limb_1_col34,
        op1_limb_2_col35,
        remainder_bits_col36,
        partial_limb_msb_col37,
        enabler,
    ] = input.try_into().unwrap();
    let enabler_constraint_value = eval!(context, ((enabler) * (enabler)) - (enabler));
    acc.add_constraint(context, enabler_constraint_value);

    let [
        decode_instruction_bc3cd_output_tmp_756b7_11_offset0,
        decode_instruction_bc3cd_output_tmp_756b7_11_offset1,
        decode_instruction_bc3cd_output_tmp_756b7_11_offset2,
        decode_instruction_bc3cd_output_tmp_756b7_11_op1_base_ap,
    ] = decode_instruction_bc3cd::accumulate_constraints(
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
        (op1_imm_col8) * ((1) - (decode_instruction_bc3cd_output_tmp_756b7_11_offset2))
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
                + ((decode_instruction_bc3cd_output_tmp_756b7_11_op1_base_ap) * (input_ap_col1)))
    );
    acc.add_constraint(context, constraint_4_value);

    let [read_small_output_tmp_756b7_21_limb_0] = read_small::accumulate_constraints(
        &[
            eval!(
                context,
                (mem_dst_base_col11) + (decode_instruction_bc3cd_output_tmp_756b7_11_offset0)
            ),
            eval!(context, dst_id_col14),
            eval!(context, msb_col15),
            eval!(context, mid_limbs_set_col16),
            eval!(context, dst_limb_0_col17),
            eval!(context, dst_limb_1_col18),
            eval!(context, dst_limb_2_col19),
            eval!(context, remainder_bits_col20),
            eval!(context, partial_limb_msb_col21),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    let [read_small_output_tmp_756b7_31_limb_0] = read_small::accumulate_constraints(
        &[
            eval!(
                context,
                (mem0_base_col12) + (decode_instruction_bc3cd_output_tmp_756b7_11_offset1)
            ),
            eval!(context, op0_id_col22),
            eval!(context, msb_col23),
            eval!(context, mid_limbs_set_col24),
            eval!(context, op0_limb_0_col25),
            eval!(context, op0_limb_1_col26),
            eval!(context, op0_limb_2_col27),
            eval!(context, remainder_bits_col28),
            eval!(context, partial_limb_msb_col29),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    let [read_small_output_tmp_756b7_41_limb_0] = read_small::accumulate_constraints(
        &[
            eval!(
                context,
                (mem1_base_col13) + (decode_instruction_bc3cd_output_tmp_756b7_11_offset2)
            ),
            eval!(context, op1_id_col30),
            eval!(context, msb_col31),
            eval!(context, mid_limbs_set_col32),
            eval!(context, op1_limb_0_col33),
            eval!(context, op1_limb_1_col34),
            eval!(context, op1_limb_2_col35),
            eval!(context, remainder_bits_col36),
            eval!(context, partial_limb_msb_col37),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    //dst equals op0 + op1.
    let constraint_8_value = eval!(
        context,
        (read_small_output_tmp_756b7_21_limb_0)
            - ((read_small_output_tmp_756b7_31_limb_0) + (read_small_output_tmp_756b7_41_limb_0))
    );
    acc.add_constraint(context, constraint_8_value);

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
