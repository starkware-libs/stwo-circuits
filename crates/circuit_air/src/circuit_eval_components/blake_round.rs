// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 148;
pub const N_INTERACTION_COLUMNS: usize = 56;

pub const RELATION_USES_PER_ROW: [RelationUse; 4] = [
    RelationUse { relation_id: "BlakeG", uses: 8 },
    RelationUse { relation_id: "BlakeMessage", uses: 16 },
    RelationUse { relation_id: "BlakeRound", uses: 1 },
    RelationUse { relation_id: "BlakeRoundSigma", uses: 1 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [
        input_limb_0_col0,
        input_limb_1_col1,
        input_limb_2_col2,
        input_limb_3_col3,
        input_limb_4_col4,
        input_limb_5_col5,
        input_limb_6_col6,
        input_limb_7_col7,
        input_limb_8_col8,
        input_limb_9_col9,
        input_limb_10_col10,
        input_limb_11_col11,
        input_limb_12_col12,
        input_limb_13_col13,
        input_limb_14_col14,
        input_limb_15_col15,
        input_limb_16_col16,
        input_limb_17_col17,
        input_limb_18_col18,
        input_limb_19_col19,
        input_limb_20_col20,
        input_limb_21_col21,
        input_limb_22_col22,
        input_limb_23_col23,
        input_limb_24_col24,
        input_limb_25_col25,
        input_limb_26_col26,
        input_limb_27_col27,
        input_limb_28_col28,
        input_limb_29_col29,
        input_limb_30_col30,
        input_limb_31_col31,
        input_limb_32_col32,
        input_limb_33_col33,
        input_limb_34_col34,
        blake_round_sigma_output_limb_0_col35,
        blake_round_sigma_output_limb_1_col36,
        blake_round_sigma_output_limb_2_col37,
        blake_round_sigma_output_limb_3_col38,
        blake_round_sigma_output_limb_4_col39,
        blake_round_sigma_output_limb_5_col40,
        blake_round_sigma_output_limb_6_col41,
        blake_round_sigma_output_limb_7_col42,
        blake_round_sigma_output_limb_8_col43,
        blake_round_sigma_output_limb_9_col44,
        blake_round_sigma_output_limb_10_col45,
        blake_round_sigma_output_limb_11_col46,
        blake_round_sigma_output_limb_12_col47,
        blake_round_sigma_output_limb_13_col48,
        blake_round_sigma_output_limb_14_col49,
        blake_round_sigma_output_limb_15_col50,
        blake_message_output_message_limb_limb_0_col51,
        blake_message_output_message_limb_limb_1_col52,
        blake_message_output_message_limb_limb_0_col53,
        blake_message_output_message_limb_limb_1_col54,
        blake_message_output_message_limb_limb_0_col55,
        blake_message_output_message_limb_limb_1_col56,
        blake_message_output_message_limb_limb_0_col57,
        blake_message_output_message_limb_limb_1_col58,
        blake_message_output_message_limb_limb_0_col59,
        blake_message_output_message_limb_limb_1_col60,
        blake_message_output_message_limb_limb_0_col61,
        blake_message_output_message_limb_limb_1_col62,
        blake_message_output_message_limb_limb_0_col63,
        blake_message_output_message_limb_limb_1_col64,
        blake_message_output_message_limb_limb_0_col65,
        blake_message_output_message_limb_limb_1_col66,
        blake_message_output_message_limb_limb_0_col67,
        blake_message_output_message_limb_limb_1_col68,
        blake_message_output_message_limb_limb_0_col69,
        blake_message_output_message_limb_limb_1_col70,
        blake_message_output_message_limb_limb_0_col71,
        blake_message_output_message_limb_limb_1_col72,
        blake_message_output_message_limb_limb_0_col73,
        blake_message_output_message_limb_limb_1_col74,
        blake_message_output_message_limb_limb_0_col75,
        blake_message_output_message_limb_limb_1_col76,
        blake_message_output_message_limb_limb_0_col77,
        blake_message_output_message_limb_limb_1_col78,
        blake_message_output_message_limb_limb_0_col79,
        blake_message_output_message_limb_limb_1_col80,
        blake_message_output_message_limb_limb_0_col81,
        blake_message_output_message_limb_limb_1_col82,
        blake_g_output_limb_0_col83,
        blake_g_output_limb_1_col84,
        blake_g_output_limb_2_col85,
        blake_g_output_limb_3_col86,
        blake_g_output_limb_4_col87,
        blake_g_output_limb_5_col88,
        blake_g_output_limb_6_col89,
        blake_g_output_limb_7_col90,
        blake_g_output_limb_0_col91,
        blake_g_output_limb_1_col92,
        blake_g_output_limb_2_col93,
        blake_g_output_limb_3_col94,
        blake_g_output_limb_4_col95,
        blake_g_output_limb_5_col96,
        blake_g_output_limb_6_col97,
        blake_g_output_limb_7_col98,
        blake_g_output_limb_0_col99,
        blake_g_output_limb_1_col100,
        blake_g_output_limb_2_col101,
        blake_g_output_limb_3_col102,
        blake_g_output_limb_4_col103,
        blake_g_output_limb_5_col104,
        blake_g_output_limb_6_col105,
        blake_g_output_limb_7_col106,
        blake_g_output_limb_0_col107,
        blake_g_output_limb_1_col108,
        blake_g_output_limb_2_col109,
        blake_g_output_limb_3_col110,
        blake_g_output_limb_4_col111,
        blake_g_output_limb_5_col112,
        blake_g_output_limb_6_col113,
        blake_g_output_limb_7_col114,
        blake_g_output_limb_0_col115,
        blake_g_output_limb_1_col116,
        blake_g_output_limb_2_col117,
        blake_g_output_limb_3_col118,
        blake_g_output_limb_4_col119,
        blake_g_output_limb_5_col120,
        blake_g_output_limb_6_col121,
        blake_g_output_limb_7_col122,
        blake_g_output_limb_0_col123,
        blake_g_output_limb_1_col124,
        blake_g_output_limb_2_col125,
        blake_g_output_limb_3_col126,
        blake_g_output_limb_4_col127,
        blake_g_output_limb_5_col128,
        blake_g_output_limb_6_col129,
        blake_g_output_limb_7_col130,
        blake_g_output_limb_0_col131,
        blake_g_output_limb_1_col132,
        blake_g_output_limb_2_col133,
        blake_g_output_limb_3_col134,
        blake_g_output_limb_4_col135,
        blake_g_output_limb_5_col136,
        blake_g_output_limb_6_col137,
        blake_g_output_limb_7_col138,
        blake_g_output_limb_0_col139,
        blake_g_output_limb_1_col140,
        blake_g_output_limb_2_col141,
        blake_g_output_limb_3_col142,
        blake_g_output_limb_4_col143,
        blake_g_output_limb_5_col144,
        blake_g_output_limb_6_col145,
        blake_g_output_limb_7_col146,
        enabler_col147,
    ] = input.try_into().unwrap();

    let constraint_0_value =
        eval!(context, ((enabler_col147) * (enabler_col147)) - (enabler_col147));
    acc.add_constraint(context, constraint_0_value);

    // Use BlakeRoundSigma.
    let tuple_1 = &[
        eval!(context, 1805967942),
        eval!(context, input_limb_1_col1),
        eval!(context, blake_round_sigma_output_limb_0_col35),
        eval!(context, blake_round_sigma_output_limb_1_col36),
        eval!(context, blake_round_sigma_output_limb_2_col37),
        eval!(context, blake_round_sigma_output_limb_3_col38),
        eval!(context, blake_round_sigma_output_limb_4_col39),
        eval!(context, blake_round_sigma_output_limb_5_col40),
        eval!(context, blake_round_sigma_output_limb_6_col41),
        eval!(context, blake_round_sigma_output_limb_7_col42),
        eval!(context, blake_round_sigma_output_limb_8_col43),
        eval!(context, blake_round_sigma_output_limb_9_col44),
        eval!(context, blake_round_sigma_output_limb_10_col45),
        eval!(context, blake_round_sigma_output_limb_11_col46),
        eval!(context, blake_round_sigma_output_limb_12_col47),
        eval!(context, blake_round_sigma_output_limb_13_col48),
        eval!(context, blake_round_sigma_output_limb_14_col49),
        eval!(context, blake_round_sigma_output_limb_15_col50),
    ];
    let numerator_1 = eval!(context, 1);
    acc.add_to_relation(context, numerator_1, tuple_1);

    // Use BlakeMessage.
    let tuple_2 = &[
        eval!(context, 1492981981),
        eval!(context, input_limb_34_col34),
        eval!(context, blake_round_sigma_output_limb_0_col35),
        eval!(context, blake_message_output_message_limb_limb_0_col51),
        eval!(context, blake_message_output_message_limb_limb_1_col52),
    ];
    let numerator_2 = eval!(context, enabler_col147);
    acc.add_to_relation(context, numerator_2, tuple_2);

    // Use BlakeMessage.
    let tuple_3 = &[
        eval!(context, 1492981981),
        eval!(context, input_limb_34_col34),
        eval!(context, blake_round_sigma_output_limb_1_col36),
        eval!(context, blake_message_output_message_limb_limb_0_col53),
        eval!(context, blake_message_output_message_limb_limb_1_col54),
    ];
    let numerator_3 = eval!(context, enabler_col147);
    acc.add_to_relation(context, numerator_3, tuple_3);

    // Use BlakeMessage.
    let tuple_4 = &[
        eval!(context, 1492981981),
        eval!(context, input_limb_34_col34),
        eval!(context, blake_round_sigma_output_limb_2_col37),
        eval!(context, blake_message_output_message_limb_limb_0_col55),
        eval!(context, blake_message_output_message_limb_limb_1_col56),
    ];
    let numerator_4 = eval!(context, enabler_col147);
    acc.add_to_relation(context, numerator_4, tuple_4);

    // Use BlakeMessage.
    let tuple_5 = &[
        eval!(context, 1492981981),
        eval!(context, input_limb_34_col34),
        eval!(context, blake_round_sigma_output_limb_3_col38),
        eval!(context, blake_message_output_message_limb_limb_0_col57),
        eval!(context, blake_message_output_message_limb_limb_1_col58),
    ];
    let numerator_5 = eval!(context, enabler_col147);
    acc.add_to_relation(context, numerator_5, tuple_5);

    // Use BlakeMessage.
    let tuple_6 = &[
        eval!(context, 1492981981),
        eval!(context, input_limb_34_col34),
        eval!(context, blake_round_sigma_output_limb_4_col39),
        eval!(context, blake_message_output_message_limb_limb_0_col59),
        eval!(context, blake_message_output_message_limb_limb_1_col60),
    ];
    let numerator_6 = eval!(context, enabler_col147);
    acc.add_to_relation(context, numerator_6, tuple_6);

    // Use BlakeMessage.
    let tuple_7 = &[
        eval!(context, 1492981981),
        eval!(context, input_limb_34_col34),
        eval!(context, blake_round_sigma_output_limb_5_col40),
        eval!(context, blake_message_output_message_limb_limb_0_col61),
        eval!(context, blake_message_output_message_limb_limb_1_col62),
    ];
    let numerator_7 = eval!(context, enabler_col147);
    acc.add_to_relation(context, numerator_7, tuple_7);

    // Use BlakeMessage.
    let tuple_8 = &[
        eval!(context, 1492981981),
        eval!(context, input_limb_34_col34),
        eval!(context, blake_round_sigma_output_limb_6_col41),
        eval!(context, blake_message_output_message_limb_limb_0_col63),
        eval!(context, blake_message_output_message_limb_limb_1_col64),
    ];
    let numerator_8 = eval!(context, enabler_col147);
    acc.add_to_relation(context, numerator_8, tuple_8);

    // Use BlakeMessage.
    let tuple_9 = &[
        eval!(context, 1492981981),
        eval!(context, input_limb_34_col34),
        eval!(context, blake_round_sigma_output_limb_7_col42),
        eval!(context, blake_message_output_message_limb_limb_0_col65),
        eval!(context, blake_message_output_message_limb_limb_1_col66),
    ];
    let numerator_9 = eval!(context, enabler_col147);
    acc.add_to_relation(context, numerator_9, tuple_9);

    // Use BlakeMessage.
    let tuple_10 = &[
        eval!(context, 1492981981),
        eval!(context, input_limb_34_col34),
        eval!(context, blake_round_sigma_output_limb_8_col43),
        eval!(context, blake_message_output_message_limb_limb_0_col67),
        eval!(context, blake_message_output_message_limb_limb_1_col68),
    ];
    let numerator_10 = eval!(context, enabler_col147);
    acc.add_to_relation(context, numerator_10, tuple_10);

    // Use BlakeMessage.
    let tuple_11 = &[
        eval!(context, 1492981981),
        eval!(context, input_limb_34_col34),
        eval!(context, blake_round_sigma_output_limb_9_col44),
        eval!(context, blake_message_output_message_limb_limb_0_col69),
        eval!(context, blake_message_output_message_limb_limb_1_col70),
    ];
    let numerator_11 = eval!(context, enabler_col147);
    acc.add_to_relation(context, numerator_11, tuple_11);

    // Use BlakeMessage.
    let tuple_12 = &[
        eval!(context, 1492981981),
        eval!(context, input_limb_34_col34),
        eval!(context, blake_round_sigma_output_limb_10_col45),
        eval!(context, blake_message_output_message_limb_limb_0_col71),
        eval!(context, blake_message_output_message_limb_limb_1_col72),
    ];
    let numerator_12 = eval!(context, enabler_col147);
    acc.add_to_relation(context, numerator_12, tuple_12);

    // Use BlakeMessage.
    let tuple_13 = &[
        eval!(context, 1492981981),
        eval!(context, input_limb_34_col34),
        eval!(context, blake_round_sigma_output_limb_11_col46),
        eval!(context, blake_message_output_message_limb_limb_0_col73),
        eval!(context, blake_message_output_message_limb_limb_1_col74),
    ];
    let numerator_13 = eval!(context, enabler_col147);
    acc.add_to_relation(context, numerator_13, tuple_13);

    // Use BlakeMessage.
    let tuple_14 = &[
        eval!(context, 1492981981),
        eval!(context, input_limb_34_col34),
        eval!(context, blake_round_sigma_output_limb_12_col47),
        eval!(context, blake_message_output_message_limb_limb_0_col75),
        eval!(context, blake_message_output_message_limb_limb_1_col76),
    ];
    let numerator_14 = eval!(context, enabler_col147);
    acc.add_to_relation(context, numerator_14, tuple_14);

    // Use BlakeMessage.
    let tuple_15 = &[
        eval!(context, 1492981981),
        eval!(context, input_limb_34_col34),
        eval!(context, blake_round_sigma_output_limb_13_col48),
        eval!(context, blake_message_output_message_limb_limb_0_col77),
        eval!(context, blake_message_output_message_limb_limb_1_col78),
    ];
    let numerator_15 = eval!(context, enabler_col147);
    acc.add_to_relation(context, numerator_15, tuple_15);

    // Use BlakeMessage.
    let tuple_16 = &[
        eval!(context, 1492981981),
        eval!(context, input_limb_34_col34),
        eval!(context, blake_round_sigma_output_limb_14_col49),
        eval!(context, blake_message_output_message_limb_limb_0_col79),
        eval!(context, blake_message_output_message_limb_limb_1_col80),
    ];
    let numerator_16 = eval!(context, enabler_col147);
    acc.add_to_relation(context, numerator_16, tuple_16);

    // Use BlakeMessage.
    let tuple_17 = &[
        eval!(context, 1492981981),
        eval!(context, input_limb_34_col34),
        eval!(context, blake_round_sigma_output_limb_15_col50),
        eval!(context, blake_message_output_message_limb_limb_0_col81),
        eval!(context, blake_message_output_message_limb_limb_1_col82),
    ];
    let numerator_17 = eval!(context, enabler_col147);
    acc.add_to_relation(context, numerator_17, tuple_17);

    // Use BlakeG.
    let tuple_18 = &[
        eval!(context, 1139985212),
        eval!(context, input_limb_2_col2),
        eval!(context, input_limb_3_col3),
        eval!(context, input_limb_10_col10),
        eval!(context, input_limb_11_col11),
        eval!(context, input_limb_18_col18),
        eval!(context, input_limb_19_col19),
        eval!(context, input_limb_26_col26),
        eval!(context, input_limb_27_col27),
        eval!(context, blake_message_output_message_limb_limb_0_col51),
        eval!(context, blake_message_output_message_limb_limb_1_col52),
        eval!(context, blake_message_output_message_limb_limb_0_col53),
        eval!(context, blake_message_output_message_limb_limb_1_col54),
        eval!(context, blake_g_output_limb_0_col83),
        eval!(context, blake_g_output_limb_1_col84),
        eval!(context, blake_g_output_limb_2_col85),
        eval!(context, blake_g_output_limb_3_col86),
        eval!(context, blake_g_output_limb_4_col87),
        eval!(context, blake_g_output_limb_5_col88),
        eval!(context, blake_g_output_limb_6_col89),
        eval!(context, blake_g_output_limb_7_col90),
    ];
    let numerator_18 = eval!(context, 1);
    acc.add_to_relation(context, numerator_18, tuple_18);

    // Use BlakeG.
    let tuple_19 = &[
        eval!(context, 1139985212),
        eval!(context, input_limb_4_col4),
        eval!(context, input_limb_5_col5),
        eval!(context, input_limb_12_col12),
        eval!(context, input_limb_13_col13),
        eval!(context, input_limb_20_col20),
        eval!(context, input_limb_21_col21),
        eval!(context, input_limb_28_col28),
        eval!(context, input_limb_29_col29),
        eval!(context, blake_message_output_message_limb_limb_0_col55),
        eval!(context, blake_message_output_message_limb_limb_1_col56),
        eval!(context, blake_message_output_message_limb_limb_0_col57),
        eval!(context, blake_message_output_message_limb_limb_1_col58),
        eval!(context, blake_g_output_limb_0_col91),
        eval!(context, blake_g_output_limb_1_col92),
        eval!(context, blake_g_output_limb_2_col93),
        eval!(context, blake_g_output_limb_3_col94),
        eval!(context, blake_g_output_limb_4_col95),
        eval!(context, blake_g_output_limb_5_col96),
        eval!(context, blake_g_output_limb_6_col97),
        eval!(context, blake_g_output_limb_7_col98),
    ];
    let numerator_19 = eval!(context, 1);
    acc.add_to_relation(context, numerator_19, tuple_19);

    // Use BlakeG.
    let tuple_20 = &[
        eval!(context, 1139985212),
        eval!(context, input_limb_6_col6),
        eval!(context, input_limb_7_col7),
        eval!(context, input_limb_14_col14),
        eval!(context, input_limb_15_col15),
        eval!(context, input_limb_22_col22),
        eval!(context, input_limb_23_col23),
        eval!(context, input_limb_30_col30),
        eval!(context, input_limb_31_col31),
        eval!(context, blake_message_output_message_limb_limb_0_col59),
        eval!(context, blake_message_output_message_limb_limb_1_col60),
        eval!(context, blake_message_output_message_limb_limb_0_col61),
        eval!(context, blake_message_output_message_limb_limb_1_col62),
        eval!(context, blake_g_output_limb_0_col99),
        eval!(context, blake_g_output_limb_1_col100),
        eval!(context, blake_g_output_limb_2_col101),
        eval!(context, blake_g_output_limb_3_col102),
        eval!(context, blake_g_output_limb_4_col103),
        eval!(context, blake_g_output_limb_5_col104),
        eval!(context, blake_g_output_limb_6_col105),
        eval!(context, blake_g_output_limb_7_col106),
    ];
    let numerator_20 = eval!(context, 1);
    acc.add_to_relation(context, numerator_20, tuple_20);

    // Use BlakeG.
    let tuple_21 = &[
        eval!(context, 1139985212),
        eval!(context, input_limb_8_col8),
        eval!(context, input_limb_9_col9),
        eval!(context, input_limb_16_col16),
        eval!(context, input_limb_17_col17),
        eval!(context, input_limb_24_col24),
        eval!(context, input_limb_25_col25),
        eval!(context, input_limb_32_col32),
        eval!(context, input_limb_33_col33),
        eval!(context, blake_message_output_message_limb_limb_0_col63),
        eval!(context, blake_message_output_message_limb_limb_1_col64),
        eval!(context, blake_message_output_message_limb_limb_0_col65),
        eval!(context, blake_message_output_message_limb_limb_1_col66),
        eval!(context, blake_g_output_limb_0_col107),
        eval!(context, blake_g_output_limb_1_col108),
        eval!(context, blake_g_output_limb_2_col109),
        eval!(context, blake_g_output_limb_3_col110),
        eval!(context, blake_g_output_limb_4_col111),
        eval!(context, blake_g_output_limb_5_col112),
        eval!(context, blake_g_output_limb_6_col113),
        eval!(context, blake_g_output_limb_7_col114),
    ];
    let numerator_21 = eval!(context, 1);
    acc.add_to_relation(context, numerator_21, tuple_21);

    // Use BlakeG.
    let tuple_22 = &[
        eval!(context, 1139985212),
        eval!(context, blake_g_output_limb_0_col83),
        eval!(context, blake_g_output_limb_1_col84),
        eval!(context, blake_g_output_limb_2_col93),
        eval!(context, blake_g_output_limb_3_col94),
        eval!(context, blake_g_output_limb_4_col103),
        eval!(context, blake_g_output_limb_5_col104),
        eval!(context, blake_g_output_limb_6_col113),
        eval!(context, blake_g_output_limb_7_col114),
        eval!(context, blake_message_output_message_limb_limb_0_col67),
        eval!(context, blake_message_output_message_limb_limb_1_col68),
        eval!(context, blake_message_output_message_limb_limb_0_col69),
        eval!(context, blake_message_output_message_limb_limb_1_col70),
        eval!(context, blake_g_output_limb_0_col115),
        eval!(context, blake_g_output_limb_1_col116),
        eval!(context, blake_g_output_limb_2_col117),
        eval!(context, blake_g_output_limb_3_col118),
        eval!(context, blake_g_output_limb_4_col119),
        eval!(context, blake_g_output_limb_5_col120),
        eval!(context, blake_g_output_limb_6_col121),
        eval!(context, blake_g_output_limb_7_col122),
    ];
    let numerator_22 = eval!(context, 1);
    acc.add_to_relation(context, numerator_22, tuple_22);

    // Use BlakeG.
    let tuple_23 = &[
        eval!(context, 1139985212),
        eval!(context, blake_g_output_limb_0_col91),
        eval!(context, blake_g_output_limb_1_col92),
        eval!(context, blake_g_output_limb_2_col101),
        eval!(context, blake_g_output_limb_3_col102),
        eval!(context, blake_g_output_limb_4_col111),
        eval!(context, blake_g_output_limb_5_col112),
        eval!(context, blake_g_output_limb_6_col89),
        eval!(context, blake_g_output_limb_7_col90),
        eval!(context, blake_message_output_message_limb_limb_0_col71),
        eval!(context, blake_message_output_message_limb_limb_1_col72),
        eval!(context, blake_message_output_message_limb_limb_0_col73),
        eval!(context, blake_message_output_message_limb_limb_1_col74),
        eval!(context, blake_g_output_limb_0_col123),
        eval!(context, blake_g_output_limb_1_col124),
        eval!(context, blake_g_output_limb_2_col125),
        eval!(context, blake_g_output_limb_3_col126),
        eval!(context, blake_g_output_limb_4_col127),
        eval!(context, blake_g_output_limb_5_col128),
        eval!(context, blake_g_output_limb_6_col129),
        eval!(context, blake_g_output_limb_7_col130),
    ];
    let numerator_23 = eval!(context, 1);
    acc.add_to_relation(context, numerator_23, tuple_23);

    // Use BlakeG.
    let tuple_24 = &[
        eval!(context, 1139985212),
        eval!(context, blake_g_output_limb_0_col99),
        eval!(context, blake_g_output_limb_1_col100),
        eval!(context, blake_g_output_limb_2_col109),
        eval!(context, blake_g_output_limb_3_col110),
        eval!(context, blake_g_output_limb_4_col87),
        eval!(context, blake_g_output_limb_5_col88),
        eval!(context, blake_g_output_limb_6_col97),
        eval!(context, blake_g_output_limb_7_col98),
        eval!(context, blake_message_output_message_limb_limb_0_col75),
        eval!(context, blake_message_output_message_limb_limb_1_col76),
        eval!(context, blake_message_output_message_limb_limb_0_col77),
        eval!(context, blake_message_output_message_limb_limb_1_col78),
        eval!(context, blake_g_output_limb_0_col131),
        eval!(context, blake_g_output_limb_1_col132),
        eval!(context, blake_g_output_limb_2_col133),
        eval!(context, blake_g_output_limb_3_col134),
        eval!(context, blake_g_output_limb_4_col135),
        eval!(context, blake_g_output_limb_5_col136),
        eval!(context, blake_g_output_limb_6_col137),
        eval!(context, blake_g_output_limb_7_col138),
    ];
    let numerator_24 = eval!(context, 1);
    acc.add_to_relation(context, numerator_24, tuple_24);

    // Use BlakeG.
    let tuple_25 = &[
        eval!(context, 1139985212),
        eval!(context, blake_g_output_limb_0_col107),
        eval!(context, blake_g_output_limb_1_col108),
        eval!(context, blake_g_output_limb_2_col85),
        eval!(context, blake_g_output_limb_3_col86),
        eval!(context, blake_g_output_limb_4_col95),
        eval!(context, blake_g_output_limb_5_col96),
        eval!(context, blake_g_output_limb_6_col105),
        eval!(context, blake_g_output_limb_7_col106),
        eval!(context, blake_message_output_message_limb_limb_0_col79),
        eval!(context, blake_message_output_message_limb_limb_1_col80),
        eval!(context, blake_message_output_message_limb_limb_0_col81),
        eval!(context, blake_message_output_message_limb_limb_1_col82),
        eval!(context, blake_g_output_limb_0_col139),
        eval!(context, blake_g_output_limb_1_col140),
        eval!(context, blake_g_output_limb_2_col141),
        eval!(context, blake_g_output_limb_3_col142),
        eval!(context, blake_g_output_limb_4_col143),
        eval!(context, blake_g_output_limb_5_col144),
        eval!(context, blake_g_output_limb_6_col145),
        eval!(context, blake_g_output_limb_7_col146),
    ];
    let numerator_25 = eval!(context, 1);
    acc.add_to_relation(context, numerator_25, tuple_25);

    // Use BlakeRound.
    let tuple_26 = &[
        eval!(context, 40528774),
        eval!(context, input_limb_0_col0),
        eval!(context, input_limb_1_col1),
        eval!(context, input_limb_2_col2),
        eval!(context, input_limb_3_col3),
        eval!(context, input_limb_4_col4),
        eval!(context, input_limb_5_col5),
        eval!(context, input_limb_6_col6),
        eval!(context, input_limb_7_col7),
        eval!(context, input_limb_8_col8),
        eval!(context, input_limb_9_col9),
        eval!(context, input_limb_10_col10),
        eval!(context, input_limb_11_col11),
        eval!(context, input_limb_12_col12),
        eval!(context, input_limb_13_col13),
        eval!(context, input_limb_14_col14),
        eval!(context, input_limb_15_col15),
        eval!(context, input_limb_16_col16),
        eval!(context, input_limb_17_col17),
        eval!(context, input_limb_18_col18),
        eval!(context, input_limb_19_col19),
        eval!(context, input_limb_20_col20),
        eval!(context, input_limb_21_col21),
        eval!(context, input_limb_22_col22),
        eval!(context, input_limb_23_col23),
        eval!(context, input_limb_24_col24),
        eval!(context, input_limb_25_col25),
        eval!(context, input_limb_26_col26),
        eval!(context, input_limb_27_col27),
        eval!(context, input_limb_28_col28),
        eval!(context, input_limb_29_col29),
        eval!(context, input_limb_30_col30),
        eval!(context, input_limb_31_col31),
        eval!(context, input_limb_32_col32),
        eval!(context, input_limb_33_col33),
        eval!(context, input_limb_34_col34),
    ];
    let numerator_26 = eval!(context, enabler_col147);
    acc.add_to_relation(context, numerator_26, tuple_26);

    // Yield BlakeRound.
    let tuple_27 = &[
        eval!(context, 40528774),
        eval!(context, input_limb_0_col0),
        eval!(context, (input_limb_1_col1) + (1)),
        eval!(context, blake_g_output_limb_0_col115),
        eval!(context, blake_g_output_limb_1_col116),
        eval!(context, blake_g_output_limb_0_col123),
        eval!(context, blake_g_output_limb_1_col124),
        eval!(context, blake_g_output_limb_0_col131),
        eval!(context, blake_g_output_limb_1_col132),
        eval!(context, blake_g_output_limb_0_col139),
        eval!(context, blake_g_output_limb_1_col140),
        eval!(context, blake_g_output_limb_2_col141),
        eval!(context, blake_g_output_limb_3_col142),
        eval!(context, blake_g_output_limb_2_col117),
        eval!(context, blake_g_output_limb_3_col118),
        eval!(context, blake_g_output_limb_2_col125),
        eval!(context, blake_g_output_limb_3_col126),
        eval!(context, blake_g_output_limb_2_col133),
        eval!(context, blake_g_output_limb_3_col134),
        eval!(context, blake_g_output_limb_4_col135),
        eval!(context, blake_g_output_limb_5_col136),
        eval!(context, blake_g_output_limb_4_col143),
        eval!(context, blake_g_output_limb_5_col144),
        eval!(context, blake_g_output_limb_4_col119),
        eval!(context, blake_g_output_limb_5_col120),
        eval!(context, blake_g_output_limb_4_col127),
        eval!(context, blake_g_output_limb_5_col128),
        eval!(context, blake_g_output_limb_6_col129),
        eval!(context, blake_g_output_limb_7_col130),
        eval!(context, blake_g_output_limb_6_col137),
        eval!(context, blake_g_output_limb_7_col138),
        eval!(context, blake_g_output_limb_6_col145),
        eval!(context, blake_g_output_limb_7_col146),
        eval!(context, blake_g_output_limb_6_col121),
        eval!(context, blake_g_output_limb_7_col122),
        eval!(context, input_limb_34_col34),
    ];
    let numerator_27 = eval!(context, -(enabler_col147));
    acc.add_to_relation(context, numerator_27, tuple_27);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "blake_round".to_string()
    }

    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &dyn ComponentDataTrait<Value>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        accumulate_constraints(component_data.trace_columns(), context, component_data, acc);
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
