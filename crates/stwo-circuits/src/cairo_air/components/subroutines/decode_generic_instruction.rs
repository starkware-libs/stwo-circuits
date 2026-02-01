// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] =
    [RelationUse { relation_id: "VerifyInstruction", uses: 1 }];

#[allow(unused_variables)]
pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let _ = component_data;
    let _ = acc;
    let [
        decode_generic_instruction_input,
        offset0_col0,
        offset1_col1,
        offset2_col2,
        dst_base_fp_col3,
        op0_base_fp_col4,
        op1_imm_col5,
        op1_base_fp_col6,
        op1_base_ap_col7,
        res_add_col8,
        res_mul_col9,
        pc_update_jump_col10,
        pc_update_jump_rel_col11,
        pc_update_jnz_col12,
        ap_update_add_col13,
        ap_update_add_1_col14,
        opcode_call_col15,
        opcode_ret_col16,
        opcode_assert_eq_col17,
    ] = input.try_into().unwrap();

    let [
        decode_instruction_df7a6_output_tmp_62f3c_20_offset0,
        decode_instruction_df7a6_output_tmp_62f3c_20_offset1,
        decode_instruction_df7a6_output_tmp_62f3c_20_offset2,
    ] = decode_instruction_df7a6::accumulate_constraints(
        &[
            eval!(context, decode_generic_instruction_input),
            eval!(context, offset0_col0),
            eval!(context, offset1_col1),
            eval!(context, offset2_col2),
            eval!(context, dst_base_fp_col3),
            eval!(context, op0_base_fp_col4),
            eval!(context, op1_imm_col5),
            eval!(context, op1_base_fp_col6),
            eval!(context, op1_base_ap_col7),
            eval!(context, res_add_col8),
            eval!(context, res_mul_col9),
            eval!(context, pc_update_jump_col10),
            eval!(context, pc_update_jump_rel_col11),
            eval!(context, pc_update_jnz_col12),
            eval!(context, ap_update_add_col13),
            eval!(context, ap_update_add_1_col14),
            eval!(context, opcode_call_col15),
            eval!(context, opcode_ret_col16),
            eval!(context, opcode_assert_eq_col17),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    let op1_base_op0_tmp_62f3c_21 =
        eval!(context, (((1) - (op1_imm_col5)) - (op1_base_fp_col6)) - (op1_base_ap_col7));

    //op1_src is 0, 1, 2, or 4.
    let constraint_2_value =
        eval!(context, (op1_base_op0_tmp_62f3c_21) * ((1) - (op1_base_op0_tmp_62f3c_21)));
    acc.add_constraint(context, constraint_2_value);

    let res_op1_tmp_62f3c_22 =
        eval!(context, (((1) - (res_add_col8)) - (res_mul_col9)) - (pc_update_jnz_col12));

    //res_logic is 0, 1, or 2.
    let constraint_4_value =
        eval!(context, (res_op1_tmp_62f3c_22) * ((1) - (res_op1_tmp_62f3c_22)));
    acc.add_constraint(context, constraint_4_value);

    let pc_update_regular_tmp_62f3c_23 = eval!(
        context,
        (((1) - (pc_update_jump_col10)) - (pc_update_jump_rel_col11)) - (pc_update_jnz_col12)
    );

    //pc_update is 0, 1, 2, or 4.
    let constraint_6_value =
        eval!(context, (pc_update_regular_tmp_62f3c_23) * ((1) - (pc_update_regular_tmp_62f3c_23)));
    acc.add_constraint(context, constraint_6_value);

    let ap_update_regular_tmp_62f3c_24 = eval!(
        context,
        (((1) - (ap_update_add_col13)) - (ap_update_add_1_col14)) - (opcode_call_col15)
    );

    //ap_update is 0, 1, 2, or 4.
    let constraint_8_value =
        eval!(context, (ap_update_regular_tmp_62f3c_24) * ((1) - (ap_update_regular_tmp_62f3c_24)));
    acc.add_constraint(context, constraint_8_value);

    let fp_update_regular_tmp_62f3c_25 =
        eval!(context, ((1) - (opcode_call_col15)) - (opcode_ret_col16));

    //opcode is 0, 1, 2, or 4.
    let constraint_10_value =
        eval!(context, (fp_update_regular_tmp_62f3c_25) * ((1) - (fp_update_regular_tmp_62f3c_25)));
    acc.add_constraint(context, constraint_10_value);
    vec![
        eval!(context, op1_base_op0_tmp_62f3c_21),
        eval!(context, res_op1_tmp_62f3c_22),
        eval!(context, pc_update_regular_tmp_62f3c_23),
        eval!(context, fp_update_regular_tmp_62f3c_25),
        eval!(context, (1) + (op1_imm_col5)),
        eval!(context, decode_instruction_df7a6_output_tmp_62f3c_20_offset0),
        eval!(context, decode_instruction_df7a6_output_tmp_62f3c_20_offset1),
        eval!(context, decode_instruction_df7a6_output_tmp_62f3c_20_offset2),
    ]
}
