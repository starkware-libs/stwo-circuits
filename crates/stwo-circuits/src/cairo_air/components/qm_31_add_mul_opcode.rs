// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 73;
pub const N_INTERACTION_COLUMNS: usize = 24;

pub const RELATION_USES_PER_ROW: [RelationUse; 5] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 3 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 3 },
    RelationUse { relation_id: "Opcodes", uses: 1 },
    RelationUse { relation_id: "RangeCheck_4_4_4_4", uses: 3 },
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
        res_add_col10,
        ap_update_add_1_col11,
        mem_dst_base_col12,
        mem0_base_col13,
        mem1_base_col14,
        dst_id_col15,
        dst_limb_0_col16,
        dst_limb_1_col17,
        dst_limb_2_col18,
        dst_limb_3_col19,
        dst_limb_4_col20,
        dst_limb_5_col21,
        dst_limb_6_col22,
        dst_limb_7_col23,
        dst_limb_8_col24,
        dst_limb_9_col25,
        dst_limb_10_col26,
        dst_limb_11_col27,
        dst_limb_12_col28,
        dst_limb_13_col29,
        dst_limb_14_col30,
        dst_limb_15_col31,
        dst_delta_ab_inv_col32,
        dst_delta_cd_inv_col33,
        op0_id_col34,
        op0_limb_0_col35,
        op0_limb_1_col36,
        op0_limb_2_col37,
        op0_limb_3_col38,
        op0_limb_4_col39,
        op0_limb_5_col40,
        op0_limb_6_col41,
        op0_limb_7_col42,
        op0_limb_8_col43,
        op0_limb_9_col44,
        op0_limb_10_col45,
        op0_limb_11_col46,
        op0_limb_12_col47,
        op0_limb_13_col48,
        op0_limb_14_col49,
        op0_limb_15_col50,
        op0_delta_ab_inv_col51,
        op0_delta_cd_inv_col52,
        op1_id_col53,
        op1_limb_0_col54,
        op1_limb_1_col55,
        op1_limb_2_col56,
        op1_limb_3_col57,
        op1_limb_4_col58,
        op1_limb_5_col59,
        op1_limb_6_col60,
        op1_limb_7_col61,
        op1_limb_8_col62,
        op1_limb_9_col63,
        op1_limb_10_col64,
        op1_limb_11_col65,
        op1_limb_12_col66,
        op1_limb_13_col67,
        op1_limb_14_col68,
        op1_limb_15_col69,
        op1_delta_ab_inv_col70,
        op1_delta_cd_inv_col71,
        enabler,
    ] = input.try_into().unwrap();
    let enabler_constraint_value = eval!(context, ((enabler) * (enabler)) - (enabler));
    acc.add_constraint(context, enabler_constraint_value);

    let [
        decode_instruction_3802d_output_tmp_fa85a_12_offset0,
        decode_instruction_3802d_output_tmp_fa85a_12_offset1,
        decode_instruction_3802d_output_tmp_fa85a_12_offset2,
        decode_instruction_3802d_output_tmp_fa85a_12_op1_base_ap,
        decode_instruction_3802d_output_tmp_fa85a_12_res_mul,
    ] = decode_instruction_3802d::accumulate_constraints(
        &[
            eval!(context, input_pc_col0),
            eval!(context, offset0_col3),
            eval!(context, offset1_col4),
            eval!(context, offset2_col5),
            eval!(context, dst_base_fp_col6),
            eval!(context, op0_base_fp_col7),
            eval!(context, op1_imm_col8),
            eval!(context, op1_base_fp_col9),
            eval!(context, res_add_col10),
            eval!(context, ap_update_add_1_col11),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    //Either flag op1_imm is off or offset2 is equal to 1.
    let constraint_1_value = eval!(
        context,
        (op1_imm_col8) * ((decode_instruction_3802d_output_tmp_fa85a_12_offset2) - (1))
    );
    acc.add_constraint(context, constraint_1_value);

    //mem_dst_base.
    let constraint_2_value = eval!(
        context,
        (mem_dst_base_col12)
            - (((dst_base_fp_col6) * (input_fp_col2))
                + (((1) - (dst_base_fp_col6)) * (input_ap_col1)))
    );
    acc.add_constraint(context, constraint_2_value);

    //mem0_base.
    let constraint_3_value = eval!(
        context,
        (mem0_base_col13)
            - (((op0_base_fp_col7) * (input_fp_col2))
                + (((1) - (op0_base_fp_col7)) * (input_ap_col1)))
    );
    acc.add_constraint(context, constraint_3_value);

    //mem1_base.
    let constraint_4_value = eval!(
        context,
        (mem1_base_col14)
            - ((((op1_base_fp_col9) * (input_fp_col2))
                + ((decode_instruction_3802d_output_tmp_fa85a_12_op1_base_ap) * (input_ap_col1)))
                + ((op1_imm_col8) * (input_pc_col0)))
    );
    acc.add_constraint(context, constraint_4_value);

    let [
        qm_31_read_reduced_output_tmp_fa85a_18_limb_0,
        qm_31_read_reduced_output_tmp_fa85a_18_limb_1,
        qm_31_read_reduced_output_tmp_fa85a_18_limb_2,
        qm_31_read_reduced_output_tmp_fa85a_18_limb_3,
    ] = qm_31_read_reduced::accumulate_constraints(
        &[
            eval!(
                context,
                (mem_dst_base_col12) + (decode_instruction_3802d_output_tmp_fa85a_12_offset0)
            ),
            eval!(context, dst_id_col15),
            eval!(context, dst_limb_0_col16),
            eval!(context, dst_limb_1_col17),
            eval!(context, dst_limb_2_col18),
            eval!(context, dst_limb_3_col19),
            eval!(context, dst_limb_4_col20),
            eval!(context, dst_limb_5_col21),
            eval!(context, dst_limb_6_col22),
            eval!(context, dst_limb_7_col23),
            eval!(context, dst_limb_8_col24),
            eval!(context, dst_limb_9_col25),
            eval!(context, dst_limb_10_col26),
            eval!(context, dst_limb_11_col27),
            eval!(context, dst_limb_12_col28),
            eval!(context, dst_limb_13_col29),
            eval!(context, dst_limb_14_col30),
            eval!(context, dst_limb_15_col31),
            eval!(context, dst_delta_ab_inv_col32),
            eval!(context, dst_delta_cd_inv_col33),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    let [
        qm_31_read_reduced_output_tmp_fa85a_24_limb_0,
        qm_31_read_reduced_output_tmp_fa85a_24_limb_1,
        qm_31_read_reduced_output_tmp_fa85a_24_limb_2,
        qm_31_read_reduced_output_tmp_fa85a_24_limb_3,
    ] = qm_31_read_reduced::accumulate_constraints(
        &[
            eval!(
                context,
                (mem0_base_col13) + (decode_instruction_3802d_output_tmp_fa85a_12_offset1)
            ),
            eval!(context, op0_id_col34),
            eval!(context, op0_limb_0_col35),
            eval!(context, op0_limb_1_col36),
            eval!(context, op0_limb_2_col37),
            eval!(context, op0_limb_3_col38),
            eval!(context, op0_limb_4_col39),
            eval!(context, op0_limb_5_col40),
            eval!(context, op0_limb_6_col41),
            eval!(context, op0_limb_7_col42),
            eval!(context, op0_limb_8_col43),
            eval!(context, op0_limb_9_col44),
            eval!(context, op0_limb_10_col45),
            eval!(context, op0_limb_11_col46),
            eval!(context, op0_limb_12_col47),
            eval!(context, op0_limb_13_col48),
            eval!(context, op0_limb_14_col49),
            eval!(context, op0_limb_15_col50),
            eval!(context, op0_delta_ab_inv_col51),
            eval!(context, op0_delta_cd_inv_col52),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    let [
        qm_31_read_reduced_output_tmp_fa85a_30_limb_0,
        qm_31_read_reduced_output_tmp_fa85a_30_limb_1,
        qm_31_read_reduced_output_tmp_fa85a_30_limb_2,
        qm_31_read_reduced_output_tmp_fa85a_30_limb_3,
    ] = qm_31_read_reduced::accumulate_constraints(
        &[
            eval!(
                context,
                (mem1_base_col14) + (decode_instruction_3802d_output_tmp_fa85a_12_offset2)
            ),
            eval!(context, op1_id_col53),
            eval!(context, op1_limb_0_col54),
            eval!(context, op1_limb_1_col55),
            eval!(context, op1_limb_2_col56),
            eval!(context, op1_limb_3_col57),
            eval!(context, op1_limb_4_col58),
            eval!(context, op1_limb_5_col59),
            eval!(context, op1_limb_6_col60),
            eval!(context, op1_limb_7_col61),
            eval!(context, op1_limb_8_col62),
            eval!(context, op1_limb_9_col63),
            eval!(context, op1_limb_10_col64),
            eval!(context, op1_limb_11_col65),
            eval!(context, op1_limb_12_col66),
            eval!(context, op1_limb_13_col67),
            eval!(context, op1_limb_14_col68),
            eval!(context, op1_limb_15_col69),
            eval!(context, op1_delta_ab_inv_col70),
            eval!(context, op1_delta_cd_inv_col71),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    //dst equals (op0 * op1)*flag_res_mul + (op0 + op1)*(1-flag_res_mul).
    let constraint_8_value = eval!(
        context,
        ((qm_31_read_reduced_output_tmp_fa85a_18_limb_0)
            - (((((((qm_31_read_reduced_output_tmp_fa85a_24_limb_0)
                * (qm_31_read_reduced_output_tmp_fa85a_30_limb_0))
                - ((qm_31_read_reduced_output_tmp_fa85a_24_limb_1)
                    * (qm_31_read_reduced_output_tmp_fa85a_30_limb_1)))
                + ((2)
                    * (((qm_31_read_reduced_output_tmp_fa85a_24_limb_2)
                        * (qm_31_read_reduced_output_tmp_fa85a_30_limb_2))
                        - ((qm_31_read_reduced_output_tmp_fa85a_24_limb_3)
                            * (qm_31_read_reduced_output_tmp_fa85a_30_limb_3)))))
                - ((qm_31_read_reduced_output_tmp_fa85a_24_limb_2)
                    * (qm_31_read_reduced_output_tmp_fa85a_30_limb_3)))
                - ((qm_31_read_reduced_output_tmp_fa85a_24_limb_3)
                    * (qm_31_read_reduced_output_tmp_fa85a_30_limb_2)))
                * (decode_instruction_3802d_output_tmp_fa85a_12_res_mul)))
            - (((qm_31_read_reduced_output_tmp_fa85a_24_limb_0)
                + (qm_31_read_reduced_output_tmp_fa85a_30_limb_0))
                * (res_add_col10))
    );
    acc.add_constraint(context, constraint_8_value);

    //dst equals (op0 * op1)*flag_res_mul + (op0 + op1)*(1-flag_res_mul).
    let constraint_9_value = eval!(
        context,
        ((qm_31_read_reduced_output_tmp_fa85a_18_limb_1)
            - (((((((qm_31_read_reduced_output_tmp_fa85a_24_limb_0)
                * (qm_31_read_reduced_output_tmp_fa85a_30_limb_1))
                + ((qm_31_read_reduced_output_tmp_fa85a_24_limb_1)
                    * (qm_31_read_reduced_output_tmp_fa85a_30_limb_0)))
                + ((2)
                    * (((qm_31_read_reduced_output_tmp_fa85a_24_limb_2)
                        * (qm_31_read_reduced_output_tmp_fa85a_30_limb_3))
                        + ((qm_31_read_reduced_output_tmp_fa85a_24_limb_3)
                            * (qm_31_read_reduced_output_tmp_fa85a_30_limb_2)))))
                + ((qm_31_read_reduced_output_tmp_fa85a_24_limb_2)
                    * (qm_31_read_reduced_output_tmp_fa85a_30_limb_2)))
                - ((qm_31_read_reduced_output_tmp_fa85a_24_limb_3)
                    * (qm_31_read_reduced_output_tmp_fa85a_30_limb_3)))
                * (decode_instruction_3802d_output_tmp_fa85a_12_res_mul)))
            - (((qm_31_read_reduced_output_tmp_fa85a_24_limb_1)
                + (qm_31_read_reduced_output_tmp_fa85a_30_limb_1))
                * (res_add_col10))
    );
    acc.add_constraint(context, constraint_9_value);

    //dst equals (op0 * op1)*flag_res_mul + (op0 + op1)*(1-flag_res_mul).
    let constraint_10_value = eval!(
        context,
        ((qm_31_read_reduced_output_tmp_fa85a_18_limb_2)
            - ((((((qm_31_read_reduced_output_tmp_fa85a_24_limb_0)
                * (qm_31_read_reduced_output_tmp_fa85a_30_limb_2))
                - ((qm_31_read_reduced_output_tmp_fa85a_24_limb_1)
                    * (qm_31_read_reduced_output_tmp_fa85a_30_limb_3)))
                + ((qm_31_read_reduced_output_tmp_fa85a_24_limb_2)
                    * (qm_31_read_reduced_output_tmp_fa85a_30_limb_0)))
                - ((qm_31_read_reduced_output_tmp_fa85a_24_limb_3)
                    * (qm_31_read_reduced_output_tmp_fa85a_30_limb_1)))
                * (decode_instruction_3802d_output_tmp_fa85a_12_res_mul)))
            - (((qm_31_read_reduced_output_tmp_fa85a_24_limb_2)
                + (qm_31_read_reduced_output_tmp_fa85a_30_limb_2))
                * (res_add_col10))
    );
    acc.add_constraint(context, constraint_10_value);

    //dst equals (op0 * op1)*flag_res_mul + (op0 + op1)*(1-flag_res_mul).
    let constraint_11_value = eval!(
        context,
        ((qm_31_read_reduced_output_tmp_fa85a_18_limb_3)
            - ((((((qm_31_read_reduced_output_tmp_fa85a_24_limb_0)
                * (qm_31_read_reduced_output_tmp_fa85a_30_limb_3))
                + ((qm_31_read_reduced_output_tmp_fa85a_24_limb_1)
                    * (qm_31_read_reduced_output_tmp_fa85a_30_limb_2)))
                + ((qm_31_read_reduced_output_tmp_fa85a_24_limb_2)
                    * (qm_31_read_reduced_output_tmp_fa85a_30_limb_1)))
                + ((qm_31_read_reduced_output_tmp_fa85a_24_limb_3)
                    * (qm_31_read_reduced_output_tmp_fa85a_30_limb_0)))
                * (decode_instruction_3802d_output_tmp_fa85a_12_res_mul)))
            - (((qm_31_read_reduced_output_tmp_fa85a_24_limb_3)
                + (qm_31_read_reduced_output_tmp_fa85a_30_limb_3))
                * (res_add_col10))
    );
    acc.add_constraint(context, constraint_11_value);

    // Use Opcodes.
    let tuple_12 = &[
        eval!(context, 428564188),
        eval!(context, input_pc_col0),
        eval!(context, input_ap_col1),
        eval!(context, input_fp_col2),
    ];
    let numerator_12 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_12, tuple_12);

    // Yield Opcodes.
    let tuple_13 = &[
        eval!(context, 428564188),
        eval!(context, ((input_pc_col0) + (1)) + (op1_imm_col8)),
        eval!(context, (input_ap_col1) + (ap_update_add_1_col11)),
        eval!(context, input_fp_col2),
    ];
    let numerator_13 = eval!(context, -(enabler));
    acc.add_to_relation(context, numerator_13, tuple_13);
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
