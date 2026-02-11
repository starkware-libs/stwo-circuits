// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 89;
pub const N_INTERACTION_COLUMNS: usize = 76;

pub const RELATION_USES_PER_ROW: [RelationUse; 4] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 5 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 5 },
    RelationUse { relation_id: "VerifyBitwiseXor_8", uses: 1 },
    RelationUse { relation_id: "VerifyBitwiseXor_9", uses: 27 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [
        op0_id_col0,
        op0_limb_0_col1,
        op0_limb_1_col2,
        op0_limb_2_col3,
        op0_limb_3_col4,
        op0_limb_4_col5,
        op0_limb_5_col6,
        op0_limb_6_col7,
        op0_limb_7_col8,
        op0_limb_8_col9,
        op0_limb_9_col10,
        op0_limb_10_col11,
        op0_limb_11_col12,
        op0_limb_12_col13,
        op0_limb_13_col14,
        op0_limb_14_col15,
        op0_limb_15_col16,
        op0_limb_16_col17,
        op0_limb_17_col18,
        op0_limb_18_col19,
        op0_limb_19_col20,
        op0_limb_20_col21,
        op0_limb_21_col22,
        op0_limb_22_col23,
        op0_limb_23_col24,
        op0_limb_24_col25,
        op0_limb_25_col26,
        op0_limb_26_col27,
        op0_limb_27_col28,
        op1_id_col29,
        op1_limb_0_col30,
        op1_limb_1_col31,
        op1_limb_2_col32,
        op1_limb_3_col33,
        op1_limb_4_col34,
        op1_limb_5_col35,
        op1_limb_6_col36,
        op1_limb_7_col37,
        op1_limb_8_col38,
        op1_limb_9_col39,
        op1_limb_10_col40,
        op1_limb_11_col41,
        op1_limb_12_col42,
        op1_limb_13_col43,
        op1_limb_14_col44,
        op1_limb_15_col45,
        op1_limb_16_col46,
        op1_limb_17_col47,
        op1_limb_18_col48,
        op1_limb_19_col49,
        op1_limb_20_col50,
        op1_limb_21_col51,
        op1_limb_22_col52,
        op1_limb_23_col53,
        op1_limb_24_col54,
        op1_limb_25_col55,
        op1_limb_26_col56,
        op1_limb_27_col57,
        xor_col58,
        xor_col59,
        xor_col60,
        xor_col61,
        xor_col62,
        xor_col63,
        xor_col64,
        xor_col65,
        xor_col66,
        xor_col67,
        xor_col68,
        xor_col69,
        xor_col70,
        xor_col71,
        xor_col72,
        xor_col73,
        xor_col74,
        xor_col75,
        xor_col76,
        xor_col77,
        xor_col78,
        xor_col79,
        xor_col80,
        xor_col81,
        xor_col82,
        xor_col83,
        xor_col84,
        xor_col85,
        and_id_col86,
        xor_id_col87,
        or_id_col88,
    ] = input.try_into().unwrap();
    let seq = seq_of_component_size(context, component_data, acc);
    let bitwise_builtin_segment_start =
        *acc.public_params.get("bitwise_builtin_segment_start").unwrap();

    read_positive_num_bits_252::accumulate_constraints(
        &[
            eval!(context, (bitwise_builtin_segment_start) + ((seq) * (5))),
            eval!(context, op0_id_col0),
            eval!(context, op0_limb_0_col1),
            eval!(context, op0_limb_1_col2),
            eval!(context, op0_limb_2_col3),
            eval!(context, op0_limb_3_col4),
            eval!(context, op0_limb_4_col5),
            eval!(context, op0_limb_5_col6),
            eval!(context, op0_limb_6_col7),
            eval!(context, op0_limb_7_col8),
            eval!(context, op0_limb_8_col9),
            eval!(context, op0_limb_9_col10),
            eval!(context, op0_limb_10_col11),
            eval!(context, op0_limb_11_col12),
            eval!(context, op0_limb_12_col13),
            eval!(context, op0_limb_13_col14),
            eval!(context, op0_limb_14_col15),
            eval!(context, op0_limb_15_col16),
            eval!(context, op0_limb_16_col17),
            eval!(context, op0_limb_17_col18),
            eval!(context, op0_limb_18_col19),
            eval!(context, op0_limb_19_col20),
            eval!(context, op0_limb_20_col21),
            eval!(context, op0_limb_21_col22),
            eval!(context, op0_limb_22_col23),
            eval!(context, op0_limb_23_col24),
            eval!(context, op0_limb_24_col25),
            eval!(context, op0_limb_25_col26),
            eval!(context, op0_limb_26_col27),
            eval!(context, op0_limb_27_col28),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_252::accumulate_constraints(
        &[
            eval!(context, ((bitwise_builtin_segment_start) + ((seq) * (5))) + (1)),
            eval!(context, op1_id_col29),
            eval!(context, op1_limb_0_col30),
            eval!(context, op1_limb_1_col31),
            eval!(context, op1_limb_2_col32),
            eval!(context, op1_limb_3_col33),
            eval!(context, op1_limb_4_col34),
            eval!(context, op1_limb_5_col35),
            eval!(context, op1_limb_6_col36),
            eval!(context, op1_limb_7_col37),
            eval!(context, op1_limb_8_col38),
            eval!(context, op1_limb_9_col39),
            eval!(context, op1_limb_10_col40),
            eval!(context, op1_limb_11_col41),
            eval!(context, op1_limb_12_col42),
            eval!(context, op1_limb_13_col43),
            eval!(context, op1_limb_14_col44),
            eval!(context, op1_limb_15_col45),
            eval!(context, op1_limb_16_col46),
            eval!(context, op1_limb_17_col47),
            eval!(context, op1_limb_18_col48),
            eval!(context, op1_limb_19_col49),
            eval!(context, op1_limb_20_col50),
            eval!(context, op1_limb_21_col51),
            eval!(context, op1_limb_22_col52),
            eval!(context, op1_limb_23_col53),
            eval!(context, op1_limb_24_col54),
            eval!(context, op1_limb_25_col55),
            eval!(context, op1_limb_26_col56),
            eval!(context, op1_limb_27_col57),
        ],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_0_col1),
            eval!(context, op1_limb_0_col30),
            eval!(context, xor_col58),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_12 =
        eval!(context, (1073741824) * (((op0_limb_0_col1) + (op1_limb_0_col30)) - (xor_col58)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_1_col2),
            eval!(context, op1_limb_1_col31),
            eval!(context, xor_col59),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_15 =
        eval!(context, (1073741824) * (((op0_limb_1_col2) + (op1_limb_1_col31)) - (xor_col59)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_2_col3),
            eval!(context, op1_limb_2_col32),
            eval!(context, xor_col60),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_18 =
        eval!(context, (1073741824) * (((op0_limb_2_col3) + (op1_limb_2_col32)) - (xor_col60)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_3_col4),
            eval!(context, op1_limb_3_col33),
            eval!(context, xor_col61),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_21 =
        eval!(context, (1073741824) * (((op0_limb_3_col4) + (op1_limb_3_col33)) - (xor_col61)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_4_col5),
            eval!(context, op1_limb_4_col34),
            eval!(context, xor_col62),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_24 =
        eval!(context, (1073741824) * (((op0_limb_4_col5) + (op1_limb_4_col34)) - (xor_col62)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_5_col6),
            eval!(context, op1_limb_5_col35),
            eval!(context, xor_col63),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_27 =
        eval!(context, (1073741824) * (((op0_limb_5_col6) + (op1_limb_5_col35)) - (xor_col63)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_6_col7),
            eval!(context, op1_limb_6_col36),
            eval!(context, xor_col64),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_30 =
        eval!(context, (1073741824) * (((op0_limb_6_col7) + (op1_limb_6_col36)) - (xor_col64)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_7_col8),
            eval!(context, op1_limb_7_col37),
            eval!(context, xor_col65),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_33 =
        eval!(context, (1073741824) * (((op0_limb_7_col8) + (op1_limb_7_col37)) - (xor_col65)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_8_col9),
            eval!(context, op1_limb_8_col38),
            eval!(context, xor_col66),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_36 =
        eval!(context, (1073741824) * (((op0_limb_8_col9) + (op1_limb_8_col38)) - (xor_col66)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_9_col10),
            eval!(context, op1_limb_9_col39),
            eval!(context, xor_col67),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_39 =
        eval!(context, (1073741824) * (((op0_limb_9_col10) + (op1_limb_9_col39)) - (xor_col67)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_10_col11),
            eval!(context, op1_limb_10_col40),
            eval!(context, xor_col68),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_42 =
        eval!(context, (1073741824) * (((op0_limb_10_col11) + (op1_limb_10_col40)) - (xor_col68)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_11_col12),
            eval!(context, op1_limb_11_col41),
            eval!(context, xor_col69),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_45 =
        eval!(context, (1073741824) * (((op0_limb_11_col12) + (op1_limb_11_col41)) - (xor_col69)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_12_col13),
            eval!(context, op1_limb_12_col42),
            eval!(context, xor_col70),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_48 =
        eval!(context, (1073741824) * (((op0_limb_12_col13) + (op1_limb_12_col42)) - (xor_col70)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_13_col14),
            eval!(context, op1_limb_13_col43),
            eval!(context, xor_col71),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_51 =
        eval!(context, (1073741824) * (((op0_limb_13_col14) + (op1_limb_13_col43)) - (xor_col71)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_14_col15),
            eval!(context, op1_limb_14_col44),
            eval!(context, xor_col72),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_54 =
        eval!(context, (1073741824) * (((op0_limb_14_col15) + (op1_limb_14_col44)) - (xor_col72)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_15_col16),
            eval!(context, op1_limb_15_col45),
            eval!(context, xor_col73),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_57 =
        eval!(context, (1073741824) * (((op0_limb_15_col16) + (op1_limb_15_col45)) - (xor_col73)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_16_col17),
            eval!(context, op1_limb_16_col46),
            eval!(context, xor_col74),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_60 =
        eval!(context, (1073741824) * (((op0_limb_16_col17) + (op1_limb_16_col46)) - (xor_col74)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_17_col18),
            eval!(context, op1_limb_17_col47),
            eval!(context, xor_col75),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_63 =
        eval!(context, (1073741824) * (((op0_limb_17_col18) + (op1_limb_17_col47)) - (xor_col75)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_18_col19),
            eval!(context, op1_limb_18_col48),
            eval!(context, xor_col76),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_66 =
        eval!(context, (1073741824) * (((op0_limb_18_col19) + (op1_limb_18_col48)) - (xor_col76)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_19_col20),
            eval!(context, op1_limb_19_col49),
            eval!(context, xor_col77),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_69 =
        eval!(context, (1073741824) * (((op0_limb_19_col20) + (op1_limb_19_col49)) - (xor_col77)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_20_col21),
            eval!(context, op1_limb_20_col50),
            eval!(context, xor_col78),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_72 =
        eval!(context, (1073741824) * (((op0_limb_20_col21) + (op1_limb_20_col50)) - (xor_col78)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_21_col22),
            eval!(context, op1_limb_21_col51),
            eval!(context, xor_col79),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_75 =
        eval!(context, (1073741824) * (((op0_limb_21_col22) + (op1_limb_21_col51)) - (xor_col79)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_22_col23),
            eval!(context, op1_limb_22_col52),
            eval!(context, xor_col80),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_78 =
        eval!(context, (1073741824) * (((op0_limb_22_col23) + (op1_limb_22_col52)) - (xor_col80)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_23_col24),
            eval!(context, op1_limb_23_col53),
            eval!(context, xor_col81),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_81 =
        eval!(context, (1073741824) * (((op0_limb_23_col24) + (op1_limb_23_col53)) - (xor_col81)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_24_col25),
            eval!(context, op1_limb_24_col54),
            eval!(context, xor_col82),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_84 =
        eval!(context, (1073741824) * (((op0_limb_24_col25) + (op1_limb_24_col54)) - (xor_col82)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_25_col26),
            eval!(context, op1_limb_25_col55),
            eval!(context, xor_col83),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_87 =
        eval!(context, (1073741824) * (((op0_limb_25_col26) + (op1_limb_25_col55)) - (xor_col83)));

    bitwise_xor_num_bits_9::accumulate_constraints(
        &[
            eval!(context, op0_limb_26_col27),
            eval!(context, op1_limb_26_col56),
            eval!(context, xor_col84),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_90 =
        eval!(context, (1073741824) * (((op0_limb_26_col27) + (op1_limb_26_col56)) - (xor_col84)));

    bitwise_xor_num_bits_8::accumulate_constraints(
        &[
            eval!(context, op0_limb_27_col28),
            eval!(context, op1_limb_27_col57),
            eval!(context, xor_col85),
        ],
        context,
        component_data,
        acc,
    );

    let and_tmp_efb2a_93 =
        eval!(context, (1073741824) * (((op0_limb_27_col28) + (op1_limb_27_col57)) - (xor_col85)));

    mem_verify::accumulate_constraints(
        &[
            eval!(context, ((bitwise_builtin_segment_start) + ((seq) * (5))) + (2)),
            eval!(context, and_tmp_efb2a_12),
            eval!(context, and_tmp_efb2a_15),
            eval!(context, and_tmp_efb2a_18),
            eval!(context, and_tmp_efb2a_21),
            eval!(context, and_tmp_efb2a_24),
            eval!(context, and_tmp_efb2a_27),
            eval!(context, and_tmp_efb2a_30),
            eval!(context, and_tmp_efb2a_33),
            eval!(context, and_tmp_efb2a_36),
            eval!(context, and_tmp_efb2a_39),
            eval!(context, and_tmp_efb2a_42),
            eval!(context, and_tmp_efb2a_45),
            eval!(context, and_tmp_efb2a_48),
            eval!(context, and_tmp_efb2a_51),
            eval!(context, and_tmp_efb2a_54),
            eval!(context, and_tmp_efb2a_57),
            eval!(context, and_tmp_efb2a_60),
            eval!(context, and_tmp_efb2a_63),
            eval!(context, and_tmp_efb2a_66),
            eval!(context, and_tmp_efb2a_69),
            eval!(context, and_tmp_efb2a_72),
            eval!(context, and_tmp_efb2a_75),
            eval!(context, and_tmp_efb2a_78),
            eval!(context, and_tmp_efb2a_81),
            eval!(context, and_tmp_efb2a_84),
            eval!(context, and_tmp_efb2a_87),
            eval!(context, and_tmp_efb2a_90),
            eval!(context, and_tmp_efb2a_93),
            eval!(context, and_id_col86),
        ],
        context,
        component_data,
        acc,
    );

    mem_verify::accumulate_constraints(
        &[
            eval!(context, ((bitwise_builtin_segment_start) + ((seq) * (5))) + (3)),
            eval!(context, xor_col58),
            eval!(context, xor_col59),
            eval!(context, xor_col60),
            eval!(context, xor_col61),
            eval!(context, xor_col62),
            eval!(context, xor_col63),
            eval!(context, xor_col64),
            eval!(context, xor_col65),
            eval!(context, xor_col66),
            eval!(context, xor_col67),
            eval!(context, xor_col68),
            eval!(context, xor_col69),
            eval!(context, xor_col70),
            eval!(context, xor_col71),
            eval!(context, xor_col72),
            eval!(context, xor_col73),
            eval!(context, xor_col74),
            eval!(context, xor_col75),
            eval!(context, xor_col76),
            eval!(context, xor_col77),
            eval!(context, xor_col78),
            eval!(context, xor_col79),
            eval!(context, xor_col80),
            eval!(context, xor_col81),
            eval!(context, xor_col82),
            eval!(context, xor_col83),
            eval!(context, xor_col84),
            eval!(context, xor_col85),
            eval!(context, xor_id_col87),
        ],
        context,
        component_data,
        acc,
    );

    mem_verify::accumulate_constraints(
        &[
            eval!(context, ((bitwise_builtin_segment_start) + ((seq) * (5))) + (4)),
            eval!(context, (and_tmp_efb2a_12) + (xor_col58)),
            eval!(context, (and_tmp_efb2a_15) + (xor_col59)),
            eval!(context, (and_tmp_efb2a_18) + (xor_col60)),
            eval!(context, (and_tmp_efb2a_21) + (xor_col61)),
            eval!(context, (and_tmp_efb2a_24) + (xor_col62)),
            eval!(context, (and_tmp_efb2a_27) + (xor_col63)),
            eval!(context, (and_tmp_efb2a_30) + (xor_col64)),
            eval!(context, (and_tmp_efb2a_33) + (xor_col65)),
            eval!(context, (and_tmp_efb2a_36) + (xor_col66)),
            eval!(context, (and_tmp_efb2a_39) + (xor_col67)),
            eval!(context, (and_tmp_efb2a_42) + (xor_col68)),
            eval!(context, (and_tmp_efb2a_45) + (xor_col69)),
            eval!(context, (and_tmp_efb2a_48) + (xor_col70)),
            eval!(context, (and_tmp_efb2a_51) + (xor_col71)),
            eval!(context, (and_tmp_efb2a_54) + (xor_col72)),
            eval!(context, (and_tmp_efb2a_57) + (xor_col73)),
            eval!(context, (and_tmp_efb2a_60) + (xor_col74)),
            eval!(context, (and_tmp_efb2a_63) + (xor_col75)),
            eval!(context, (and_tmp_efb2a_66) + (xor_col76)),
            eval!(context, (and_tmp_efb2a_69) + (xor_col77)),
            eval!(context, (and_tmp_efb2a_72) + (xor_col78)),
            eval!(context, (and_tmp_efb2a_75) + (xor_col79)),
            eval!(context, (and_tmp_efb2a_78) + (xor_col80)),
            eval!(context, (and_tmp_efb2a_81) + (xor_col81)),
            eval!(context, (and_tmp_efb2a_84) + (xor_col82)),
            eval!(context, (and_tmp_efb2a_87) + (xor_col83)),
            eval!(context, (and_tmp_efb2a_90) + (xor_col84)),
            eval!(context, (and_tmp_efb2a_93) + (xor_col85)),
            eval!(context, or_id_col88),
        ],
        context,
        component_data,
        acc,
    );
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
