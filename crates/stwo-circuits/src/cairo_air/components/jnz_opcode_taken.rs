// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 47;
pub const N_INTERACTION_COLUMNS: usize = 16;

pub const RELATION_USES_PER_ROW: [RelationUse; 4] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 2 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 2 },
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
        dst_base_fp_col4,
        ap_update_add_1_col5,
        mem_dst_base_col6,
        dst_id_col7,
        dst_limb_0_col8,
        dst_limb_1_col9,
        dst_limb_2_col10,
        dst_limb_3_col11,
        dst_limb_4_col12,
        dst_limb_5_col13,
        dst_limb_6_col14,
        dst_limb_7_col15,
        dst_limb_8_col16,
        dst_limb_9_col17,
        dst_limb_10_col18,
        dst_limb_11_col19,
        dst_limb_12_col20,
        dst_limb_13_col21,
        dst_limb_14_col22,
        dst_limb_15_col23,
        dst_limb_16_col24,
        dst_limb_17_col25,
        dst_limb_18_col26,
        dst_limb_19_col27,
        dst_limb_20_col28,
        dst_limb_21_col29,
        dst_limb_22_col30,
        dst_limb_23_col31,
        dst_limb_24_col32,
        dst_limb_25_col33,
        dst_limb_26_col34,
        dst_limb_27_col35,
        dst_sum_inv_col36,
        dst_sum_squares_inv_col37,
        next_pc_id_col38,
        msb_col39,
        mid_limbs_set_col40,
        next_pc_limb_0_col41,
        next_pc_limb_1_col42,
        next_pc_limb_2_col43,
        remainder_bits_col44,
        partial_limb_msb_col45,
        enabler,
    ] = input.try_into().unwrap();
    let enabler_constraint_value = eval!(context, ((enabler) * (enabler)) - (enabler));
    acc.add_constraint(context, enabler_constraint_value);

    let [decode_instruction_de75a_output_tmp_f51a9_5_offset0] =
        decode_instruction_de75a::accumulate_constraints(
            &[
                eval!(context, input_pc_col0),
                eval!(context, offset0_col3),
                eval!(context, dst_base_fp_col4),
                eval!(context, ap_update_add_1_col5),
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
        (mem_dst_base_col6)
            - (((dst_base_fp_col4) * (input_fp_col2))
                + (((1) - (dst_base_fp_col4)) * (input_ap_col1)))
    );
    acc.add_constraint(context, constraint_1_value);

    read_positive_num_bits_252::accumulate_constraints(
        &[
            eval!(
                context,
                (mem_dst_base_col6) + (decode_instruction_de75a_output_tmp_f51a9_5_offset0)
            ),
            eval!(context, dst_id_col7),
            eval!(context, dst_limb_0_col8),
            eval!(context, dst_limb_1_col9),
            eval!(context, dst_limb_2_col10),
            eval!(context, dst_limb_3_col11),
            eval!(context, dst_limb_4_col12),
            eval!(context, dst_limb_5_col13),
            eval!(context, dst_limb_6_col14),
            eval!(context, dst_limb_7_col15),
            eval!(context, dst_limb_8_col16),
            eval!(context, dst_limb_9_col17),
            eval!(context, dst_limb_10_col18),
            eval!(context, dst_limb_11_col19),
            eval!(context, dst_limb_12_col20),
            eval!(context, dst_limb_13_col21),
            eval!(context, dst_limb_14_col22),
            eval!(context, dst_limb_15_col23),
            eval!(context, dst_limb_16_col24),
            eval!(context, dst_limb_17_col25),
            eval!(context, dst_limb_18_col26),
            eval!(context, dst_limb_19_col27),
            eval!(context, dst_limb_20_col28),
            eval!(context, dst_limb_21_col29),
            eval!(context, dst_limb_22_col30),
            eval!(context, dst_limb_23_col31),
            eval!(context, dst_limb_24_col32),
            eval!(context, dst_limb_25_col33),
            eval!(context, dst_limb_26_col34),
            eval!(context, dst_limb_27_col35),
        ],
        context,
        component_data,
        acc,
    );

    let dst_sum_p_zero_tmp_f51a9_11 = eval!(
        context,
        ((((((((((((((((((((((((dst_limb_1_col9) + (dst_limb_2_col10))
            + (dst_limb_3_col11))
            + (dst_limb_4_col12))
            + (dst_limb_5_col13))
            + (dst_limb_6_col14))
            + (dst_limb_7_col15))
            + (dst_limb_8_col16))
            + (dst_limb_9_col17))
            + (dst_limb_10_col18))
            + (dst_limb_11_col19))
            + (dst_limb_12_col20))
            + (dst_limb_13_col21))
            + (dst_limb_14_col22))
            + (dst_limb_15_col23))
            + (dst_limb_16_col24))
            + (dst_limb_17_col25))
            + (dst_limb_18_col26))
            + (dst_limb_19_col27))
            + (dst_limb_20_col28))
            + (dst_limb_22_col30))
            + (dst_limb_23_col31))
            + (dst_limb_24_col32))
            + (dst_limb_25_col33))
            + (dst_limb_26_col34)
    );

    //dst doesn't equal 0.
    let constraint_4_value = eval!(
        context,
        (((dst_sum_p_zero_tmp_f51a9_11)
            + (((dst_limb_0_col8) + (dst_limb_21_col29)) + (dst_limb_27_col35)))
            * (dst_sum_inv_col36))
            - (1)
    );
    acc.add_constraint(context, constraint_4_value);

    let diff_from_p_tmp_f51a9_12 = eval!(context, (dst_limb_0_col8) - (1));

    let diff_from_p_tmp_f51a9_13 = eval!(context, (dst_limb_21_col29) - (136));

    let diff_from_p_tmp_f51a9_14 = eval!(context, (dst_limb_27_col35) - (256));

    //dst doesn't equal P.
    let constraint_8_value = eval!(
        context,
        (((dst_sum_p_zero_tmp_f51a9_11)
            + ((((diff_from_p_tmp_f51a9_12) * (diff_from_p_tmp_f51a9_12))
                + ((diff_from_p_tmp_f51a9_13) * (diff_from_p_tmp_f51a9_13)))
                + ((diff_from_p_tmp_f51a9_14) * (diff_from_p_tmp_f51a9_14))))
            * (dst_sum_squares_inv_col37))
            - (1)
    );
    acc.add_constraint(context, constraint_8_value);

    let [read_small_output_tmp_f51a9_24_limb_0] = read_small::accumulate_constraints(
        &[
            eval!(context, (input_pc_col0) + (1)),
            eval!(context, next_pc_id_col38),
            eval!(context, msb_col39),
            eval!(context, mid_limbs_set_col40),
            eval!(context, next_pc_limb_0_col41),
            eval!(context, next_pc_limb_1_col42),
            eval!(context, next_pc_limb_2_col43),
            eval!(context, remainder_bits_col44),
            eval!(context, partial_limb_msb_col45),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    // Use Opcodes.
    let tuple_10 = &[
        eval!(context, 428564188),
        eval!(context, input_pc_col0),
        eval!(context, input_ap_col1),
        eval!(context, input_fp_col2),
    ];
    let numerator_10 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_10, tuple_10);

    // Yield Opcodes.
    let tuple_11 = &[
        eval!(context, 428564188),
        eval!(context, (input_pc_col0) + (read_small_output_tmp_f51a9_24_limb_0)),
        eval!(context, (input_ap_col1) + (ap_update_add_1_col5)),
        eval!(context, input_fp_col2),
    ];
    let numerator_11 = eval!(context, -(enabler));
    acc.add_to_relation(context, numerator_11, tuple_11);
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
