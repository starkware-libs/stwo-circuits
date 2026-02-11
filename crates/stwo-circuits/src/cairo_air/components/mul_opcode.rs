// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 130;
pub const N_INTERACTION_COLUMNS: usize = 76;

pub const RELATION_USES_PER_ROW: [RelationUse; 12] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 3 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 3 },
    RelationUse { relation_id: "Opcodes", uses: 1 },
    RelationUse { relation_id: "RangeCheck_20", uses: 4 },
    RelationUse { relation_id: "RangeCheck_20_B", uses: 4 },
    RelationUse { relation_id: "RangeCheck_20_C", uses: 4 },
    RelationUse { relation_id: "RangeCheck_20_D", uses: 4 },
    RelationUse { relation_id: "RangeCheck_20_E", uses: 3 },
    RelationUse { relation_id: "RangeCheck_20_F", uses: 3 },
    RelationUse { relation_id: "RangeCheck_20_G", uses: 3 },
    RelationUse { relation_id: "RangeCheck_20_H", uses: 3 },
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
        dst_limb_8_col23,
        dst_limb_9_col24,
        dst_limb_10_col25,
        dst_limb_11_col26,
        dst_limb_12_col27,
        dst_limb_13_col28,
        dst_limb_14_col29,
        dst_limb_15_col30,
        dst_limb_16_col31,
        dst_limb_17_col32,
        dst_limb_18_col33,
        dst_limb_19_col34,
        dst_limb_20_col35,
        dst_limb_21_col36,
        dst_limb_22_col37,
        dst_limb_23_col38,
        dst_limb_24_col39,
        dst_limb_25_col40,
        dst_limb_26_col41,
        dst_limb_27_col42,
        op0_id_col43,
        op0_limb_0_col44,
        op0_limb_1_col45,
        op0_limb_2_col46,
        op0_limb_3_col47,
        op0_limb_4_col48,
        op0_limb_5_col49,
        op0_limb_6_col50,
        op0_limb_7_col51,
        op0_limb_8_col52,
        op0_limb_9_col53,
        op0_limb_10_col54,
        op0_limb_11_col55,
        op0_limb_12_col56,
        op0_limb_13_col57,
        op0_limb_14_col58,
        op0_limb_15_col59,
        op0_limb_16_col60,
        op0_limb_17_col61,
        op0_limb_18_col62,
        op0_limb_19_col63,
        op0_limb_20_col64,
        op0_limb_21_col65,
        op0_limb_22_col66,
        op0_limb_23_col67,
        op0_limb_24_col68,
        op0_limb_25_col69,
        op0_limb_26_col70,
        op0_limb_27_col71,
        op1_id_col72,
        op1_limb_0_col73,
        op1_limb_1_col74,
        op1_limb_2_col75,
        op1_limb_3_col76,
        op1_limb_4_col77,
        op1_limb_5_col78,
        op1_limb_6_col79,
        op1_limb_7_col80,
        op1_limb_8_col81,
        op1_limb_9_col82,
        op1_limb_10_col83,
        op1_limb_11_col84,
        op1_limb_12_col85,
        op1_limb_13_col86,
        op1_limb_14_col87,
        op1_limb_15_col88,
        op1_limb_16_col89,
        op1_limb_17_col90,
        op1_limb_18_col91,
        op1_limb_19_col92,
        op1_limb_20_col93,
        op1_limb_21_col94,
        op1_limb_22_col95,
        op1_limb_23_col96,
        op1_limb_24_col97,
        op1_limb_25_col98,
        op1_limb_26_col99,
        op1_limb_27_col100,
        k_col101,
        carry_0_col102,
        carry_1_col103,
        carry_2_col104,
        carry_3_col105,
        carry_4_col106,
        carry_5_col107,
        carry_6_col108,
        carry_7_col109,
        carry_8_col110,
        carry_9_col111,
        carry_10_col112,
        carry_11_col113,
        carry_12_col114,
        carry_13_col115,
        carry_14_col116,
        carry_15_col117,
        carry_16_col118,
        carry_17_col119,
        carry_18_col120,
        carry_19_col121,
        carry_20_col122,
        carry_21_col123,
        carry_22_col124,
        carry_23_col125,
        carry_24_col126,
        carry_25_col127,
        carry_26_col128,
        enabler,
    ] = input.try_into().unwrap();
    let enabler_constraint_value = eval!(context, ((enabler) * (enabler)) - (enabler));
    acc.add_constraint(context, enabler_constraint_value);

    let [
        decode_instruction_4b8cf_output_tmp_42314_11_offset0,
        decode_instruction_4b8cf_output_tmp_42314_11_offset1,
        decode_instruction_4b8cf_output_tmp_42314_11_offset2,
        decode_instruction_4b8cf_output_tmp_42314_11_op1_base_ap,
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
        (op1_imm_col8) * ((1) - (decode_instruction_4b8cf_output_tmp_42314_11_offset2))
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
                + ((decode_instruction_4b8cf_output_tmp_42314_11_op1_base_ap) * (input_ap_col1)))
    );
    acc.add_constraint(context, constraint_4_value);

    read_positive_num_bits_252::accumulate_constraints(
        &[
            eval!(
                context,
                (mem_dst_base_col11) + (decode_instruction_4b8cf_output_tmp_42314_11_offset0)
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
            eval!(context, dst_limb_8_col23),
            eval!(context, dst_limb_9_col24),
            eval!(context, dst_limb_10_col25),
            eval!(context, dst_limb_11_col26),
            eval!(context, dst_limb_12_col27),
            eval!(context, dst_limb_13_col28),
            eval!(context, dst_limb_14_col29),
            eval!(context, dst_limb_15_col30),
            eval!(context, dst_limb_16_col31),
            eval!(context, dst_limb_17_col32),
            eval!(context, dst_limb_18_col33),
            eval!(context, dst_limb_19_col34),
            eval!(context, dst_limb_20_col35),
            eval!(context, dst_limb_21_col36),
            eval!(context, dst_limb_22_col37),
            eval!(context, dst_limb_23_col38),
            eval!(context, dst_limb_24_col39),
            eval!(context, dst_limb_25_col40),
            eval!(context, dst_limb_26_col41),
            eval!(context, dst_limb_27_col42),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_252::accumulate_constraints(
        &[
            eval!(
                context,
                (mem0_base_col12) + (decode_instruction_4b8cf_output_tmp_42314_11_offset1)
            ),
            eval!(context, op0_id_col43),
            eval!(context, op0_limb_0_col44),
            eval!(context, op0_limb_1_col45),
            eval!(context, op0_limb_2_col46),
            eval!(context, op0_limb_3_col47),
            eval!(context, op0_limb_4_col48),
            eval!(context, op0_limb_5_col49),
            eval!(context, op0_limb_6_col50),
            eval!(context, op0_limb_7_col51),
            eval!(context, op0_limb_8_col52),
            eval!(context, op0_limb_9_col53),
            eval!(context, op0_limb_10_col54),
            eval!(context, op0_limb_11_col55),
            eval!(context, op0_limb_12_col56),
            eval!(context, op0_limb_13_col57),
            eval!(context, op0_limb_14_col58),
            eval!(context, op0_limb_15_col59),
            eval!(context, op0_limb_16_col60),
            eval!(context, op0_limb_17_col61),
            eval!(context, op0_limb_18_col62),
            eval!(context, op0_limb_19_col63),
            eval!(context, op0_limb_20_col64),
            eval!(context, op0_limb_21_col65),
            eval!(context, op0_limb_22_col66),
            eval!(context, op0_limb_23_col67),
            eval!(context, op0_limb_24_col68),
            eval!(context, op0_limb_25_col69),
            eval!(context, op0_limb_26_col70),
            eval!(context, op0_limb_27_col71),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_252::accumulate_constraints(
        &[
            eval!(
                context,
                (mem1_base_col13) + (decode_instruction_4b8cf_output_tmp_42314_11_offset2)
            ),
            eval!(context, op1_id_col72),
            eval!(context, op1_limb_0_col73),
            eval!(context, op1_limb_1_col74),
            eval!(context, op1_limb_2_col75),
            eval!(context, op1_limb_3_col76),
            eval!(context, op1_limb_4_col77),
            eval!(context, op1_limb_5_col78),
            eval!(context, op1_limb_6_col79),
            eval!(context, op1_limb_7_col80),
            eval!(context, op1_limb_8_col81),
            eval!(context, op1_limb_9_col82),
            eval!(context, op1_limb_10_col83),
            eval!(context, op1_limb_11_col84),
            eval!(context, op1_limb_12_col85),
            eval!(context, op1_limb_13_col86),
            eval!(context, op1_limb_14_col87),
            eval!(context, op1_limb_15_col88),
            eval!(context, op1_limb_16_col89),
            eval!(context, op1_limb_17_col90),
            eval!(context, op1_limb_18_col91),
            eval!(context, op1_limb_19_col92),
            eval!(context, op1_limb_20_col93),
            eval!(context, op1_limb_21_col94),
            eval!(context, op1_limb_22_col95),
            eval!(context, op1_limb_23_col96),
            eval!(context, op1_limb_24_col97),
            eval!(context, op1_limb_25_col98),
            eval!(context, op1_limb_26_col99),
            eval!(context, op1_limb_27_col100),
        ],
        context,
        component_data,
        acc,
    );

    verify_mul_252::accumulate_constraints(
        &[
            eval!(context, op0_limb_0_col44),
            eval!(context, op0_limb_1_col45),
            eval!(context, op0_limb_2_col46),
            eval!(context, op0_limb_3_col47),
            eval!(context, op0_limb_4_col48),
            eval!(context, op0_limb_5_col49),
            eval!(context, op0_limb_6_col50),
            eval!(context, op0_limb_7_col51),
            eval!(context, op0_limb_8_col52),
            eval!(context, op0_limb_9_col53),
            eval!(context, op0_limb_10_col54),
            eval!(context, op0_limb_11_col55),
            eval!(context, op0_limb_12_col56),
            eval!(context, op0_limb_13_col57),
            eval!(context, op0_limb_14_col58),
            eval!(context, op0_limb_15_col59),
            eval!(context, op0_limb_16_col60),
            eval!(context, op0_limb_17_col61),
            eval!(context, op0_limb_18_col62),
            eval!(context, op0_limb_19_col63),
            eval!(context, op0_limb_20_col64),
            eval!(context, op0_limb_21_col65),
            eval!(context, op0_limb_22_col66),
            eval!(context, op0_limb_23_col67),
            eval!(context, op0_limb_24_col68),
            eval!(context, op0_limb_25_col69),
            eval!(context, op0_limb_26_col70),
            eval!(context, op0_limb_27_col71),
            eval!(context, op1_limb_0_col73),
            eval!(context, op1_limb_1_col74),
            eval!(context, op1_limb_2_col75),
            eval!(context, op1_limb_3_col76),
            eval!(context, op1_limb_4_col77),
            eval!(context, op1_limb_5_col78),
            eval!(context, op1_limb_6_col79),
            eval!(context, op1_limb_7_col80),
            eval!(context, op1_limb_8_col81),
            eval!(context, op1_limb_9_col82),
            eval!(context, op1_limb_10_col83),
            eval!(context, op1_limb_11_col84),
            eval!(context, op1_limb_12_col85),
            eval!(context, op1_limb_13_col86),
            eval!(context, op1_limb_14_col87),
            eval!(context, op1_limb_15_col88),
            eval!(context, op1_limb_16_col89),
            eval!(context, op1_limb_17_col90),
            eval!(context, op1_limb_18_col91),
            eval!(context, op1_limb_19_col92),
            eval!(context, op1_limb_20_col93),
            eval!(context, op1_limb_21_col94),
            eval!(context, op1_limb_22_col95),
            eval!(context, op1_limb_23_col96),
            eval!(context, op1_limb_24_col97),
            eval!(context, op1_limb_25_col98),
            eval!(context, op1_limb_26_col99),
            eval!(context, op1_limb_27_col100),
            eval!(context, dst_limb_0_col15),
            eval!(context, dst_limb_1_col16),
            eval!(context, dst_limb_2_col17),
            eval!(context, dst_limb_3_col18),
            eval!(context, dst_limb_4_col19),
            eval!(context, dst_limb_5_col20),
            eval!(context, dst_limb_6_col21),
            eval!(context, dst_limb_7_col22),
            eval!(context, dst_limb_8_col23),
            eval!(context, dst_limb_9_col24),
            eval!(context, dst_limb_10_col25),
            eval!(context, dst_limb_11_col26),
            eval!(context, dst_limb_12_col27),
            eval!(context, dst_limb_13_col28),
            eval!(context, dst_limb_14_col29),
            eval!(context, dst_limb_15_col30),
            eval!(context, dst_limb_16_col31),
            eval!(context, dst_limb_17_col32),
            eval!(context, dst_limb_18_col33),
            eval!(context, dst_limb_19_col34),
            eval!(context, dst_limb_20_col35),
            eval!(context, dst_limb_21_col36),
            eval!(context, dst_limb_22_col37),
            eval!(context, dst_limb_23_col38),
            eval!(context, dst_limb_24_col39),
            eval!(context, dst_limb_25_col40),
            eval!(context, dst_limb_26_col41),
            eval!(context, dst_limb_27_col42),
            eval!(context, k_col101),
            eval!(context, carry_0_col102),
            eval!(context, carry_1_col103),
            eval!(context, carry_2_col104),
            eval!(context, carry_3_col105),
            eval!(context, carry_4_col106),
            eval!(context, carry_5_col107),
            eval!(context, carry_6_col108),
            eval!(context, carry_7_col109),
            eval!(context, carry_8_col110),
            eval!(context, carry_9_col111),
            eval!(context, carry_10_col112),
            eval!(context, carry_11_col113),
            eval!(context, carry_12_col114),
            eval!(context, carry_13_col115),
            eval!(context, carry_14_col116),
            eval!(context, carry_15_col117),
            eval!(context, carry_16_col118),
            eval!(context, carry_17_col119),
            eval!(context, carry_18_col120),
            eval!(context, carry_19_col121),
            eval!(context, carry_20_col122),
            eval!(context, carry_21_col123),
            eval!(context, carry_22_col124),
            eval!(context, carry_23_col125),
            eval!(context, carry_24_col126),
            eval!(context, carry_25_col127),
            eval!(context, carry_26_col128),
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
