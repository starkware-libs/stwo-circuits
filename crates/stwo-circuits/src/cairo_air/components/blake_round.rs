// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 212;
pub const N_INTERACTION_COLUMNS: usize = 120;

pub const RELATION_USES_PER_ROW: [RelationUse; 6] = [
    RelationUse { relation_id: "BlakeG", uses: 8 },
    RelationUse { relation_id: "BlakeRound", uses: 1 },
    RelationUse { relation_id: "BlakeRoundSigma", uses: 1 },
    RelationUse { relation_id: "MemoryAddressToId", uses: 16 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 16 },
    RelationUse { relation_id: "RangeCheck_7_2_5", uses: 16 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
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
        low_16_bits_col51,
        high_16_bits_col52,
        low_7_ms_bits_col53,
        high_14_ms_bits_col54,
        high_5_ms_bits_col55,
        message_word_0_id_col56,
        low_16_bits_col57,
        high_16_bits_col58,
        low_7_ms_bits_col59,
        high_14_ms_bits_col60,
        high_5_ms_bits_col61,
        message_word_1_id_col62,
        low_16_bits_col63,
        high_16_bits_col64,
        low_7_ms_bits_col65,
        high_14_ms_bits_col66,
        high_5_ms_bits_col67,
        message_word_2_id_col68,
        low_16_bits_col69,
        high_16_bits_col70,
        low_7_ms_bits_col71,
        high_14_ms_bits_col72,
        high_5_ms_bits_col73,
        message_word_3_id_col74,
        low_16_bits_col75,
        high_16_bits_col76,
        low_7_ms_bits_col77,
        high_14_ms_bits_col78,
        high_5_ms_bits_col79,
        message_word_4_id_col80,
        low_16_bits_col81,
        high_16_bits_col82,
        low_7_ms_bits_col83,
        high_14_ms_bits_col84,
        high_5_ms_bits_col85,
        message_word_5_id_col86,
        low_16_bits_col87,
        high_16_bits_col88,
        low_7_ms_bits_col89,
        high_14_ms_bits_col90,
        high_5_ms_bits_col91,
        message_word_6_id_col92,
        low_16_bits_col93,
        high_16_bits_col94,
        low_7_ms_bits_col95,
        high_14_ms_bits_col96,
        high_5_ms_bits_col97,
        message_word_7_id_col98,
        low_16_bits_col99,
        high_16_bits_col100,
        low_7_ms_bits_col101,
        high_14_ms_bits_col102,
        high_5_ms_bits_col103,
        message_word_8_id_col104,
        low_16_bits_col105,
        high_16_bits_col106,
        low_7_ms_bits_col107,
        high_14_ms_bits_col108,
        high_5_ms_bits_col109,
        message_word_9_id_col110,
        low_16_bits_col111,
        high_16_bits_col112,
        low_7_ms_bits_col113,
        high_14_ms_bits_col114,
        high_5_ms_bits_col115,
        message_word_10_id_col116,
        low_16_bits_col117,
        high_16_bits_col118,
        low_7_ms_bits_col119,
        high_14_ms_bits_col120,
        high_5_ms_bits_col121,
        message_word_11_id_col122,
        low_16_bits_col123,
        high_16_bits_col124,
        low_7_ms_bits_col125,
        high_14_ms_bits_col126,
        high_5_ms_bits_col127,
        message_word_12_id_col128,
        low_16_bits_col129,
        high_16_bits_col130,
        low_7_ms_bits_col131,
        high_14_ms_bits_col132,
        high_5_ms_bits_col133,
        message_word_13_id_col134,
        low_16_bits_col135,
        high_16_bits_col136,
        low_7_ms_bits_col137,
        high_14_ms_bits_col138,
        high_5_ms_bits_col139,
        message_word_14_id_col140,
        low_16_bits_col141,
        high_16_bits_col142,
        low_7_ms_bits_col143,
        high_14_ms_bits_col144,
        high_5_ms_bits_col145,
        message_word_15_id_col146,
        blake_g_output_limb_0_col147,
        blake_g_output_limb_1_col148,
        blake_g_output_limb_2_col149,
        blake_g_output_limb_3_col150,
        blake_g_output_limb_4_col151,
        blake_g_output_limb_5_col152,
        blake_g_output_limb_6_col153,
        blake_g_output_limb_7_col154,
        blake_g_output_limb_0_col155,
        blake_g_output_limb_1_col156,
        blake_g_output_limb_2_col157,
        blake_g_output_limb_3_col158,
        blake_g_output_limb_4_col159,
        blake_g_output_limb_5_col160,
        blake_g_output_limb_6_col161,
        blake_g_output_limb_7_col162,
        blake_g_output_limb_0_col163,
        blake_g_output_limb_1_col164,
        blake_g_output_limb_2_col165,
        blake_g_output_limb_3_col166,
        blake_g_output_limb_4_col167,
        blake_g_output_limb_5_col168,
        blake_g_output_limb_6_col169,
        blake_g_output_limb_7_col170,
        blake_g_output_limb_0_col171,
        blake_g_output_limb_1_col172,
        blake_g_output_limb_2_col173,
        blake_g_output_limb_3_col174,
        blake_g_output_limb_4_col175,
        blake_g_output_limb_5_col176,
        blake_g_output_limb_6_col177,
        blake_g_output_limb_7_col178,
        blake_g_output_limb_0_col179,
        blake_g_output_limb_1_col180,
        blake_g_output_limb_2_col181,
        blake_g_output_limb_3_col182,
        blake_g_output_limb_4_col183,
        blake_g_output_limb_5_col184,
        blake_g_output_limb_6_col185,
        blake_g_output_limb_7_col186,
        blake_g_output_limb_0_col187,
        blake_g_output_limb_1_col188,
        blake_g_output_limb_2_col189,
        blake_g_output_limb_3_col190,
        blake_g_output_limb_4_col191,
        blake_g_output_limb_5_col192,
        blake_g_output_limb_6_col193,
        blake_g_output_limb_7_col194,
        blake_g_output_limb_0_col195,
        blake_g_output_limb_1_col196,
        blake_g_output_limb_2_col197,
        blake_g_output_limb_3_col198,
        blake_g_output_limb_4_col199,
        blake_g_output_limb_5_col200,
        blake_g_output_limb_6_col201,
        blake_g_output_limb_7_col202,
        blake_g_output_limb_0_col203,
        blake_g_output_limb_1_col204,
        blake_g_output_limb_2_col205,
        blake_g_output_limb_3_col206,
        blake_g_output_limb_4_col207,
        blake_g_output_limb_5_col208,
        blake_g_output_limb_6_col209,
        blake_g_output_limb_7_col210,
        enabler,
    ] = input.try_into().unwrap();
    let enabler_constraint_value = eval!(context, ((enabler) * (enabler)) - (enabler));
    acc.add_constraint(context, enabler_constraint_value);

    // Use BlakeRoundSigma.
    let tuple_0 = &[
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
    let numerator_0 = eval!(context, 1);
    acc.add_to_relation(context, numerator_0, tuple_0);

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (input_limb_34_col34) + (blake_round_sigma_output_limb_0_col35)),
            eval!(context, low_16_bits_col51),
            eval!(context, high_16_bits_col52),
            eval!(context, low_7_ms_bits_col53),
            eval!(context, high_14_ms_bits_col54),
            eval!(context, high_5_ms_bits_col55),
            eval!(context, message_word_0_id_col56),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (input_limb_34_col34) + (blake_round_sigma_output_limb_1_col36)),
            eval!(context, low_16_bits_col57),
            eval!(context, high_16_bits_col58),
            eval!(context, low_7_ms_bits_col59),
            eval!(context, high_14_ms_bits_col60),
            eval!(context, high_5_ms_bits_col61),
            eval!(context, message_word_1_id_col62),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (input_limb_34_col34) + (blake_round_sigma_output_limb_2_col37)),
            eval!(context, low_16_bits_col63),
            eval!(context, high_16_bits_col64),
            eval!(context, low_7_ms_bits_col65),
            eval!(context, high_14_ms_bits_col66),
            eval!(context, high_5_ms_bits_col67),
            eval!(context, message_word_2_id_col68),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (input_limb_34_col34) + (blake_round_sigma_output_limb_3_col38)),
            eval!(context, low_16_bits_col69),
            eval!(context, high_16_bits_col70),
            eval!(context, low_7_ms_bits_col71),
            eval!(context, high_14_ms_bits_col72),
            eval!(context, high_5_ms_bits_col73),
            eval!(context, message_word_3_id_col74),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (input_limb_34_col34) + (blake_round_sigma_output_limb_4_col39)),
            eval!(context, low_16_bits_col75),
            eval!(context, high_16_bits_col76),
            eval!(context, low_7_ms_bits_col77),
            eval!(context, high_14_ms_bits_col78),
            eval!(context, high_5_ms_bits_col79),
            eval!(context, message_word_4_id_col80),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (input_limb_34_col34) + (blake_round_sigma_output_limb_5_col40)),
            eval!(context, low_16_bits_col81),
            eval!(context, high_16_bits_col82),
            eval!(context, low_7_ms_bits_col83),
            eval!(context, high_14_ms_bits_col84),
            eval!(context, high_5_ms_bits_col85),
            eval!(context, message_word_5_id_col86),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (input_limb_34_col34) + (blake_round_sigma_output_limb_6_col41)),
            eval!(context, low_16_bits_col87),
            eval!(context, high_16_bits_col88),
            eval!(context, low_7_ms_bits_col89),
            eval!(context, high_14_ms_bits_col90),
            eval!(context, high_5_ms_bits_col91),
            eval!(context, message_word_6_id_col92),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (input_limb_34_col34) + (blake_round_sigma_output_limb_7_col42)),
            eval!(context, low_16_bits_col93),
            eval!(context, high_16_bits_col94),
            eval!(context, low_7_ms_bits_col95),
            eval!(context, high_14_ms_bits_col96),
            eval!(context, high_5_ms_bits_col97),
            eval!(context, message_word_7_id_col98),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (input_limb_34_col34) + (blake_round_sigma_output_limb_8_col43)),
            eval!(context, low_16_bits_col99),
            eval!(context, high_16_bits_col100),
            eval!(context, low_7_ms_bits_col101),
            eval!(context, high_14_ms_bits_col102),
            eval!(context, high_5_ms_bits_col103),
            eval!(context, message_word_8_id_col104),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (input_limb_34_col34) + (blake_round_sigma_output_limb_9_col44)),
            eval!(context, low_16_bits_col105),
            eval!(context, high_16_bits_col106),
            eval!(context, low_7_ms_bits_col107),
            eval!(context, high_14_ms_bits_col108),
            eval!(context, high_5_ms_bits_col109),
            eval!(context, message_word_9_id_col110),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (input_limb_34_col34) + (blake_round_sigma_output_limb_10_col45)),
            eval!(context, low_16_bits_col111),
            eval!(context, high_16_bits_col112),
            eval!(context, low_7_ms_bits_col113),
            eval!(context, high_14_ms_bits_col114),
            eval!(context, high_5_ms_bits_col115),
            eval!(context, message_word_10_id_col116),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (input_limb_34_col34) + (blake_round_sigma_output_limb_11_col46)),
            eval!(context, low_16_bits_col117),
            eval!(context, high_16_bits_col118),
            eval!(context, low_7_ms_bits_col119),
            eval!(context, high_14_ms_bits_col120),
            eval!(context, high_5_ms_bits_col121),
            eval!(context, message_word_11_id_col122),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (input_limb_34_col34) + (blake_round_sigma_output_limb_12_col47)),
            eval!(context, low_16_bits_col123),
            eval!(context, high_16_bits_col124),
            eval!(context, low_7_ms_bits_col125),
            eval!(context, high_14_ms_bits_col126),
            eval!(context, high_5_ms_bits_col127),
            eval!(context, message_word_12_id_col128),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (input_limb_34_col34) + (blake_round_sigma_output_limb_13_col48)),
            eval!(context, low_16_bits_col129),
            eval!(context, high_16_bits_col130),
            eval!(context, low_7_ms_bits_col131),
            eval!(context, high_14_ms_bits_col132),
            eval!(context, high_5_ms_bits_col133),
            eval!(context, message_word_13_id_col134),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (input_limb_34_col34) + (blake_round_sigma_output_limb_14_col49)),
            eval!(context, low_16_bits_col135),
            eval!(context, high_16_bits_col136),
            eval!(context, low_7_ms_bits_col137),
            eval!(context, high_14_ms_bits_col138),
            eval!(context, high_5_ms_bits_col139),
            eval!(context, message_word_14_id_col140),
        ],
        context,
        component_data,
        acc,
    );

    read_u_32::accumulate_constraints(
        &[
            eval!(context, (input_limb_34_col34) + (blake_round_sigma_output_limb_15_col50)),
            eval!(context, low_16_bits_col141),
            eval!(context, high_16_bits_col142),
            eval!(context, low_7_ms_bits_col143),
            eval!(context, high_14_ms_bits_col144),
            eval!(context, high_5_ms_bits_col145),
            eval!(context, message_word_15_id_col146),
        ],
        context,
        component_data,
        acc,
    );

    // Use BlakeG.
    let tuple_17 = &[
        eval!(context, 1139985212),
        eval!(context, input_limb_2_col2),
        eval!(context, input_limb_3_col3),
        eval!(context, input_limb_10_col10),
        eval!(context, input_limb_11_col11),
        eval!(context, input_limb_18_col18),
        eval!(context, input_limb_19_col19),
        eval!(context, input_limb_26_col26),
        eval!(context, input_limb_27_col27),
        eval!(context, low_16_bits_col51),
        eval!(context, high_16_bits_col52),
        eval!(context, low_16_bits_col57),
        eval!(context, high_16_bits_col58),
        eval!(context, blake_g_output_limb_0_col147),
        eval!(context, blake_g_output_limb_1_col148),
        eval!(context, blake_g_output_limb_2_col149),
        eval!(context, blake_g_output_limb_3_col150),
        eval!(context, blake_g_output_limb_4_col151),
        eval!(context, blake_g_output_limb_5_col152),
        eval!(context, blake_g_output_limb_6_col153),
        eval!(context, blake_g_output_limb_7_col154),
    ];
    let numerator_17 = eval!(context, 1);
    acc.add_to_relation(context, numerator_17, tuple_17);

    // Use BlakeG.
    let tuple_18 = &[
        eval!(context, 1139985212),
        eval!(context, input_limb_4_col4),
        eval!(context, input_limb_5_col5),
        eval!(context, input_limb_12_col12),
        eval!(context, input_limb_13_col13),
        eval!(context, input_limb_20_col20),
        eval!(context, input_limb_21_col21),
        eval!(context, input_limb_28_col28),
        eval!(context, input_limb_29_col29),
        eval!(context, low_16_bits_col63),
        eval!(context, high_16_bits_col64),
        eval!(context, low_16_bits_col69),
        eval!(context, high_16_bits_col70),
        eval!(context, blake_g_output_limb_0_col155),
        eval!(context, blake_g_output_limb_1_col156),
        eval!(context, blake_g_output_limb_2_col157),
        eval!(context, blake_g_output_limb_3_col158),
        eval!(context, blake_g_output_limb_4_col159),
        eval!(context, blake_g_output_limb_5_col160),
        eval!(context, blake_g_output_limb_6_col161),
        eval!(context, blake_g_output_limb_7_col162),
    ];
    let numerator_18 = eval!(context, 1);
    acc.add_to_relation(context, numerator_18, tuple_18);

    // Use BlakeG.
    let tuple_19 = &[
        eval!(context, 1139985212),
        eval!(context, input_limb_6_col6),
        eval!(context, input_limb_7_col7),
        eval!(context, input_limb_14_col14),
        eval!(context, input_limb_15_col15),
        eval!(context, input_limb_22_col22),
        eval!(context, input_limb_23_col23),
        eval!(context, input_limb_30_col30),
        eval!(context, input_limb_31_col31),
        eval!(context, low_16_bits_col75),
        eval!(context, high_16_bits_col76),
        eval!(context, low_16_bits_col81),
        eval!(context, high_16_bits_col82),
        eval!(context, blake_g_output_limb_0_col163),
        eval!(context, blake_g_output_limb_1_col164),
        eval!(context, blake_g_output_limb_2_col165),
        eval!(context, blake_g_output_limb_3_col166),
        eval!(context, blake_g_output_limb_4_col167),
        eval!(context, blake_g_output_limb_5_col168),
        eval!(context, blake_g_output_limb_6_col169),
        eval!(context, blake_g_output_limb_7_col170),
    ];
    let numerator_19 = eval!(context, 1);
    acc.add_to_relation(context, numerator_19, tuple_19);

    // Use BlakeG.
    let tuple_20 = &[
        eval!(context, 1139985212),
        eval!(context, input_limb_8_col8),
        eval!(context, input_limb_9_col9),
        eval!(context, input_limb_16_col16),
        eval!(context, input_limb_17_col17),
        eval!(context, input_limb_24_col24),
        eval!(context, input_limb_25_col25),
        eval!(context, input_limb_32_col32),
        eval!(context, input_limb_33_col33),
        eval!(context, low_16_bits_col87),
        eval!(context, high_16_bits_col88),
        eval!(context, low_16_bits_col93),
        eval!(context, high_16_bits_col94),
        eval!(context, blake_g_output_limb_0_col171),
        eval!(context, blake_g_output_limb_1_col172),
        eval!(context, blake_g_output_limb_2_col173),
        eval!(context, blake_g_output_limb_3_col174),
        eval!(context, blake_g_output_limb_4_col175),
        eval!(context, blake_g_output_limb_5_col176),
        eval!(context, blake_g_output_limb_6_col177),
        eval!(context, blake_g_output_limb_7_col178),
    ];
    let numerator_20 = eval!(context, 1);
    acc.add_to_relation(context, numerator_20, tuple_20);

    // Use BlakeG.
    let tuple_21 = &[
        eval!(context, 1139985212),
        eval!(context, blake_g_output_limb_0_col147),
        eval!(context, blake_g_output_limb_1_col148),
        eval!(context, blake_g_output_limb_2_col157),
        eval!(context, blake_g_output_limb_3_col158),
        eval!(context, blake_g_output_limb_4_col167),
        eval!(context, blake_g_output_limb_5_col168),
        eval!(context, blake_g_output_limb_6_col177),
        eval!(context, blake_g_output_limb_7_col178),
        eval!(context, low_16_bits_col99),
        eval!(context, high_16_bits_col100),
        eval!(context, low_16_bits_col105),
        eval!(context, high_16_bits_col106),
        eval!(context, blake_g_output_limb_0_col179),
        eval!(context, blake_g_output_limb_1_col180),
        eval!(context, blake_g_output_limb_2_col181),
        eval!(context, blake_g_output_limb_3_col182),
        eval!(context, blake_g_output_limb_4_col183),
        eval!(context, blake_g_output_limb_5_col184),
        eval!(context, blake_g_output_limb_6_col185),
        eval!(context, blake_g_output_limb_7_col186),
    ];
    let numerator_21 = eval!(context, 1);
    acc.add_to_relation(context, numerator_21, tuple_21);

    // Use BlakeG.
    let tuple_22 = &[
        eval!(context, 1139985212),
        eval!(context, blake_g_output_limb_0_col155),
        eval!(context, blake_g_output_limb_1_col156),
        eval!(context, blake_g_output_limb_2_col165),
        eval!(context, blake_g_output_limb_3_col166),
        eval!(context, blake_g_output_limb_4_col175),
        eval!(context, blake_g_output_limb_5_col176),
        eval!(context, blake_g_output_limb_6_col153),
        eval!(context, blake_g_output_limb_7_col154),
        eval!(context, low_16_bits_col111),
        eval!(context, high_16_bits_col112),
        eval!(context, low_16_bits_col117),
        eval!(context, high_16_bits_col118),
        eval!(context, blake_g_output_limb_0_col187),
        eval!(context, blake_g_output_limb_1_col188),
        eval!(context, blake_g_output_limb_2_col189),
        eval!(context, blake_g_output_limb_3_col190),
        eval!(context, blake_g_output_limb_4_col191),
        eval!(context, blake_g_output_limb_5_col192),
        eval!(context, blake_g_output_limb_6_col193),
        eval!(context, blake_g_output_limb_7_col194),
    ];
    let numerator_22 = eval!(context, 1);
    acc.add_to_relation(context, numerator_22, tuple_22);

    // Use BlakeG.
    let tuple_23 = &[
        eval!(context, 1139985212),
        eval!(context, blake_g_output_limb_0_col163),
        eval!(context, blake_g_output_limb_1_col164),
        eval!(context, blake_g_output_limb_2_col173),
        eval!(context, blake_g_output_limb_3_col174),
        eval!(context, blake_g_output_limb_4_col151),
        eval!(context, blake_g_output_limb_5_col152),
        eval!(context, blake_g_output_limb_6_col161),
        eval!(context, blake_g_output_limb_7_col162),
        eval!(context, low_16_bits_col123),
        eval!(context, high_16_bits_col124),
        eval!(context, low_16_bits_col129),
        eval!(context, high_16_bits_col130),
        eval!(context, blake_g_output_limb_0_col195),
        eval!(context, blake_g_output_limb_1_col196),
        eval!(context, blake_g_output_limb_2_col197),
        eval!(context, blake_g_output_limb_3_col198),
        eval!(context, blake_g_output_limb_4_col199),
        eval!(context, blake_g_output_limb_5_col200),
        eval!(context, blake_g_output_limb_6_col201),
        eval!(context, blake_g_output_limb_7_col202),
    ];
    let numerator_23 = eval!(context, 1);
    acc.add_to_relation(context, numerator_23, tuple_23);

    // Use BlakeG.
    let tuple_24 = &[
        eval!(context, 1139985212),
        eval!(context, blake_g_output_limb_0_col171),
        eval!(context, blake_g_output_limb_1_col172),
        eval!(context, blake_g_output_limb_2_col149),
        eval!(context, blake_g_output_limb_3_col150),
        eval!(context, blake_g_output_limb_4_col159),
        eval!(context, blake_g_output_limb_5_col160),
        eval!(context, blake_g_output_limb_6_col169),
        eval!(context, blake_g_output_limb_7_col170),
        eval!(context, low_16_bits_col135),
        eval!(context, high_16_bits_col136),
        eval!(context, low_16_bits_col141),
        eval!(context, high_16_bits_col142),
        eval!(context, blake_g_output_limb_0_col203),
        eval!(context, blake_g_output_limb_1_col204),
        eval!(context, blake_g_output_limb_2_col205),
        eval!(context, blake_g_output_limb_3_col206),
        eval!(context, blake_g_output_limb_4_col207),
        eval!(context, blake_g_output_limb_5_col208),
        eval!(context, blake_g_output_limb_6_col209),
        eval!(context, blake_g_output_limb_7_col210),
    ];
    let numerator_24 = eval!(context, 1);
    acc.add_to_relation(context, numerator_24, tuple_24);

    // Use BlakeRound.
    let tuple_25 = &[
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
    let numerator_25 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_25, tuple_25);

    // Yield BlakeRound.
    let tuple_26 = &[
        eval!(context, 40528774),
        eval!(context, input_limb_0_col0),
        eval!(context, (input_limb_1_col1) + (1)),
        eval!(context, blake_g_output_limb_0_col179),
        eval!(context, blake_g_output_limb_1_col180),
        eval!(context, blake_g_output_limb_0_col187),
        eval!(context, blake_g_output_limb_1_col188),
        eval!(context, blake_g_output_limb_0_col195),
        eval!(context, blake_g_output_limb_1_col196),
        eval!(context, blake_g_output_limb_0_col203),
        eval!(context, blake_g_output_limb_1_col204),
        eval!(context, blake_g_output_limb_2_col205),
        eval!(context, blake_g_output_limb_3_col206),
        eval!(context, blake_g_output_limb_2_col181),
        eval!(context, blake_g_output_limb_3_col182),
        eval!(context, blake_g_output_limb_2_col189),
        eval!(context, blake_g_output_limb_3_col190),
        eval!(context, blake_g_output_limb_2_col197),
        eval!(context, blake_g_output_limb_3_col198),
        eval!(context, blake_g_output_limb_4_col199),
        eval!(context, blake_g_output_limb_5_col200),
        eval!(context, blake_g_output_limb_4_col207),
        eval!(context, blake_g_output_limb_5_col208),
        eval!(context, blake_g_output_limb_4_col183),
        eval!(context, blake_g_output_limb_5_col184),
        eval!(context, blake_g_output_limb_4_col191),
        eval!(context, blake_g_output_limb_5_col192),
        eval!(context, blake_g_output_limb_6_col193),
        eval!(context, blake_g_output_limb_7_col194),
        eval!(context, blake_g_output_limb_6_col201),
        eval!(context, blake_g_output_limb_7_col202),
        eval!(context, blake_g_output_limb_6_col209),
        eval!(context, blake_g_output_limb_7_col210),
        eval!(context, blake_g_output_limb_6_col185),
        eval!(context, blake_g_output_limb_7_col186),
        eval!(context, input_limb_34_col34),
    ];
    let numerator_26 = eval!(context, -(enabler));
    acc.add_to_relation(context, numerator_26, tuple_26);
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
