// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 135;
pub const N_INTERACTION_COLUMNS: usize = 136;

pub const RELATION_USES_PER_ROW: [RelationUse; 7] = [
    RelationUse { relation_id: "BlakeOutput", uses: 1 },
    RelationUse { relation_id: "BlakeRound", uses: 1 },
    RelationUse { relation_id: "Gate", uses: 4 },
    RelationUse { relation_id: "RangeCheck_15", uses: 16 },
    RelationUse { relation_id: "RangeCheck_16", uses: 16 },
    RelationUse { relation_id: "TripleXor32", uses: 8 },
    RelationUse { relation_id: "VerifyBitwiseXor_8", uses: 4 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [
        input_state_before_limb0_limb_0_col0,
        input_state_before_limb0_limb_1_col1,
        input_state_before_limb1_limb_0_col2,
        input_state_before_limb1_limb_1_col3,
        input_state_before_limb2_limb_0_col4,
        input_state_before_limb2_limb_1_col5,
        input_state_before_limb3_limb_0_col6,
        input_state_before_limb3_limb_1_col7,
        input_state_before_limb4_limb_0_col8,
        input_state_before_limb4_limb_1_col9,
        input_state_before_limb5_limb_0_col10,
        input_state_before_limb5_limb_1_col11,
        input_state_before_limb6_limb_0_col12,
        input_state_before_limb6_limb_1_col13,
        input_state_before_limb7_limb_0_col14,
        input_state_before_limb7_limb_1_col15,
        input_state_after_limb0_limb_0_col16,
        input_state_after_limb0_limb_1_col17,
        input_state_after_limb1_limb_0_col18,
        input_state_after_limb1_limb_1_col19,
        input_state_after_limb2_limb_0_col20,
        input_state_after_limb2_limb_1_col21,
        input_state_after_limb3_limb_0_col22,
        input_state_after_limb3_limb_1_col23,
        input_state_after_limb4_limb_0_col24,
        input_state_after_limb4_limb_1_col25,
        input_state_after_limb5_limb_0_col26,
        input_state_after_limb5_limb_1_col27,
        input_state_after_limb6_limb_0_col28,
        input_state_after_limb6_limb_1_col29,
        input_state_after_limb7_limb_0_col30,
        input_state_after_limb7_limb_1_col31,
        input_message_limb0_col32,
        input_message_limb1_col33,
        input_message_limb2_col34,
        input_message_limb3_col35,
        input_message_limb4_col36,
        input_message_limb5_col37,
        input_message_limb6_col38,
        input_message_limb7_col39,
        input_message_limb8_col40,
        input_message_limb9_col41,
        input_message_limb10_col42,
        input_message_limb11_col43,
        input_message_limb12_col44,
        input_message_limb13_col45,
        input_message_limb14_col46,
        input_message_limb15_col47,
        ms_8_bits_col48,
        ms_8_bits_col49,
        xor_col50,
        xor_col51,
        xor_col52,
        xor_col53,
        limbi_low_col54,
        limbi_high_col55,
        limbi_low_col56,
        limbi_high_col57,
        limbi_low_col58,
        limbi_high_col59,
        limbi_low_col60,
        limbi_high_col61,
        limbi_low_col62,
        limbi_high_col63,
        limbi_low_col64,
        limbi_high_col65,
        limbi_low_col66,
        limbi_high_col67,
        limbi_low_col68,
        limbi_high_col69,
        limbi_low_col70,
        limbi_high_col71,
        limbi_low_col72,
        limbi_high_col73,
        limbi_low_col74,
        limbi_high_col75,
        limbi_low_col76,
        limbi_high_col77,
        limbi_low_col78,
        limbi_high_col79,
        limbi_low_col80,
        limbi_high_col81,
        limbi_low_col82,
        limbi_high_col83,
        limbi_low_col84,
        limbi_high_col85,
        blake_round_output_limb_0_col86,
        blake_round_output_limb_1_col87,
        blake_round_output_limb_2_col88,
        blake_round_output_limb_3_col89,
        blake_round_output_limb_4_col90,
        blake_round_output_limb_5_col91,
        blake_round_output_limb_6_col92,
        blake_round_output_limb_7_col93,
        blake_round_output_limb_8_col94,
        blake_round_output_limb_9_col95,
        blake_round_output_limb_10_col96,
        blake_round_output_limb_11_col97,
        blake_round_output_limb_12_col98,
        blake_round_output_limb_13_col99,
        blake_round_output_limb_14_col100,
        blake_round_output_limb_15_col101,
        blake_round_output_limb_16_col102,
        blake_round_output_limb_17_col103,
        blake_round_output_limb_18_col104,
        blake_round_output_limb_19_col105,
        blake_round_output_limb_20_col106,
        blake_round_output_limb_21_col107,
        blake_round_output_limb_22_col108,
        blake_round_output_limb_23_col109,
        blake_round_output_limb_24_col110,
        blake_round_output_limb_25_col111,
        blake_round_output_limb_26_col112,
        blake_round_output_limb_27_col113,
        blake_round_output_limb_28_col114,
        blake_round_output_limb_29_col115,
        blake_round_output_limb_30_col116,
        blake_round_output_limb_31_col117,
        blake_round_output_limb_32_col118,
        triple_xor_32_output_limb_0_col119,
        triple_xor_32_output_limb_1_col120,
        triple_xor_32_output_limb_0_col121,
        triple_xor_32_output_limb_1_col122,
        triple_xor_32_output_limb_0_col123,
        triple_xor_32_output_limb_1_col124,
        triple_xor_32_output_limb_0_col125,
        triple_xor_32_output_limb_1_col126,
        triple_xor_32_output_limb_0_col127,
        triple_xor_32_output_limb_1_col128,
        triple_xor_32_output_limb_0_col129,
        triple_xor_32_output_limb_1_col130,
        triple_xor_32_output_limb_0_col131,
        triple_xor_32_output_limb_1_col132,
        triple_xor_32_output_limb_0_col133,
        triple_xor_32_output_limb_1_col134,
    ] = input.try_into().unwrap();
    let seq = seq_of_component_size(context, component_data, acc);
    let blake_gate_enabler =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_gate_enabler".to_owned() });
    let finalize_flag =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "finalize_flag".to_owned() });
    let message0_addr =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "message0_addr".to_owned() });
    let message1_addr =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "message1_addr".to_owned() });
    let message2_addr =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "message2_addr".to_owned() });
    let message3_addr =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "message3_addr".to_owned() });
    let state_after_addr =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "state_after_addr".to_owned() });
    let state_before_addr =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "state_before_addr".to_owned() });

    let [
        create_blake_round_input_output_tmp_8e0ec_12_limb_0,
        create_blake_round_input_output_tmp_8e0ec_12_limb_1,
        create_blake_round_input_output_tmp_8e0ec_12_limb_2,
        create_blake_round_input_output_tmp_8e0ec_12_limb_3,
        create_blake_round_input_output_tmp_8e0ec_12_limb_4,
        create_blake_round_input_output_tmp_8e0ec_12_limb_5,
        create_blake_round_input_output_tmp_8e0ec_12_limb_6,
        create_blake_round_input_output_tmp_8e0ec_12_limb_7,
        create_blake_round_input_output_tmp_8e0ec_12_limb_8,
        create_blake_round_input_output_tmp_8e0ec_12_limb_9,
        create_blake_round_input_output_tmp_8e0ec_12_limb_10,
        create_blake_round_input_output_tmp_8e0ec_12_limb_11,
        create_blake_round_input_output_tmp_8e0ec_12_limb_12,
        create_blake_round_input_output_tmp_8e0ec_12_limb_13,
        create_blake_round_input_output_tmp_8e0ec_12_limb_14,
        create_blake_round_input_output_tmp_8e0ec_12_limb_15,
        create_blake_round_input_output_tmp_8e0ec_12_limb_24,
        create_blake_round_input_output_tmp_8e0ec_12_limb_25,
        create_blake_round_input_output_tmp_8e0ec_12_limb_28,
        create_blake_round_input_output_tmp_8e0ec_12_limb_29,
    ] = create_blake_round_input::accumulate_constraints(
        &[
            eval!(context, input_state_before_limb0_limb_0_col0),
            eval!(context, input_state_before_limb0_limb_1_col1),
            eval!(context, input_state_before_limb1_limb_0_col2),
            eval!(context, input_state_before_limb1_limb_1_col3),
            eval!(context, input_state_before_limb2_limb_0_col4),
            eval!(context, input_state_before_limb2_limb_1_col5),
            eval!(context, input_state_before_limb3_limb_0_col6),
            eval!(context, input_state_before_limb3_limb_1_col7),
            eval!(context, input_state_before_limb4_limb_0_col8),
            eval!(context, input_state_before_limb4_limb_1_col9),
            eval!(context, input_state_before_limb5_limb_0_col10),
            eval!(context, input_state_before_limb5_limb_1_col11),
            eval!(context, input_state_before_limb6_limb_0_col12),
            eval!(context, input_state_before_limb6_limb_1_col13),
            eval!(context, input_state_before_limb7_limb_0_col14),
            eval!(context, input_state_before_limb7_limb_1_col15),
            eval!(context, finalize_flag),
            eval!(context, ms_8_bits_col48),
            eval!(context, ms_8_bits_col49),
            eval!(context, xor_col50),
            eval!(context, xor_col51),
            eval!(context, xor_col52),
            eval!(context, xor_col53),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    qm_31_into_u_32::accumulate_constraints(
        &[
            eval!(context, input_message_limb0_col32),
            eval!(context, input_message_limb1_col33),
            eval!(context, input_message_limb2_col34),
            eval!(context, input_message_limb3_col35),
            eval!(context, input_message_limb4_col36),
            eval!(context, input_message_limb5_col37),
            eval!(context, input_message_limb6_col38),
            eval!(context, input_message_limb7_col39),
            eval!(context, input_message_limb8_col40),
            eval!(context, input_message_limb9_col41),
            eval!(context, input_message_limb10_col42),
            eval!(context, input_message_limb11_col43),
            eval!(context, input_message_limb12_col44),
            eval!(context, input_message_limb13_col45),
            eval!(context, input_message_limb14_col46),
            eval!(context, input_message_limb15_col47),
            eval!(context, seq),
            eval!(context, limbi_low_col54),
            eval!(context, limbi_high_col55),
            eval!(context, limbi_low_col56),
            eval!(context, limbi_high_col57),
            eval!(context, limbi_low_col58),
            eval!(context, limbi_high_col59),
            eval!(context, limbi_low_col60),
            eval!(context, limbi_high_col61),
            eval!(context, limbi_low_col62),
            eval!(context, limbi_high_col63),
            eval!(context, limbi_low_col64),
            eval!(context, limbi_high_col65),
            eval!(context, limbi_low_col66),
            eval!(context, limbi_high_col67),
            eval!(context, limbi_low_col68),
            eval!(context, limbi_high_col69),
            eval!(context, limbi_low_col70),
            eval!(context, limbi_high_col71),
            eval!(context, limbi_low_col72),
            eval!(context, limbi_high_col73),
            eval!(context, limbi_low_col74),
            eval!(context, limbi_high_col75),
            eval!(context, limbi_low_col76),
            eval!(context, limbi_high_col77),
            eval!(context, limbi_low_col78),
            eval!(context, limbi_high_col79),
            eval!(context, limbi_low_col80),
            eval!(context, limbi_high_col81),
            eval!(context, limbi_low_col82),
            eval!(context, limbi_high_col83),
            eval!(context, limbi_low_col84),
            eval!(context, limbi_high_col85),
        ],
        context,
        component_data,
        acc,
    );

    // Yield BlakeRound.
    let tuple_2 = &[
        eval!(context, 40528774),
        eval!(context, seq),
        eval!(context, 0),
        eval!(context, input_state_before_limb0_limb_0_col0),
        eval!(context, input_state_before_limb0_limb_1_col1),
        eval!(context, input_state_before_limb1_limb_0_col2),
        eval!(context, input_state_before_limb1_limb_1_col3),
        eval!(context, input_state_before_limb2_limb_0_col4),
        eval!(context, input_state_before_limb2_limb_1_col5),
        eval!(context, input_state_before_limb3_limb_0_col6),
        eval!(context, input_state_before_limb3_limb_1_col7),
        eval!(context, input_state_before_limb4_limb_0_col8),
        eval!(context, input_state_before_limb4_limb_1_col9),
        eval!(context, input_state_before_limb5_limb_0_col10),
        eval!(context, input_state_before_limb5_limb_1_col11),
        eval!(context, input_state_before_limb6_limb_0_col12),
        eval!(context, input_state_before_limb6_limb_1_col13),
        eval!(context, input_state_before_limb7_limb_0_col14),
        eval!(context, input_state_before_limb7_limb_1_col15),
        eval!(context, 58983),
        eval!(context, 27145),
        eval!(context, 44677),
        eval!(context, 47975),
        eval!(context, 62322),
        eval!(context, 15470),
        eval!(context, 62778),
        eval!(context, 42319),
        eval!(context, create_blake_round_input_output_tmp_8e0ec_12_limb_24),
        eval!(context, create_blake_round_input_output_tmp_8e0ec_12_limb_25),
        eval!(context, 26764),
        eval!(context, 39685),
        eval!(context, create_blake_round_input_output_tmp_8e0ec_12_limb_28),
        eval!(context, create_blake_round_input_output_tmp_8e0ec_12_limb_29),
        eval!(context, 52505),
        eval!(context, 23520),
        eval!(context, seq),
    ];
    let numerator_2 = eval!(context, -(1));
    acc.add_to_relation(context, numerator_2, tuple_2);

    // Use BlakeRound.
    let tuple_3 = &[
        eval!(context, 40528774),
        eval!(context, seq),
        eval!(context, 10),
        eval!(context, blake_round_output_limb_0_col86),
        eval!(context, blake_round_output_limb_1_col87),
        eval!(context, blake_round_output_limb_2_col88),
        eval!(context, blake_round_output_limb_3_col89),
        eval!(context, blake_round_output_limb_4_col90),
        eval!(context, blake_round_output_limb_5_col91),
        eval!(context, blake_round_output_limb_6_col92),
        eval!(context, blake_round_output_limb_7_col93),
        eval!(context, blake_round_output_limb_8_col94),
        eval!(context, blake_round_output_limb_9_col95),
        eval!(context, blake_round_output_limb_10_col96),
        eval!(context, blake_round_output_limb_11_col97),
        eval!(context, blake_round_output_limb_12_col98),
        eval!(context, blake_round_output_limb_13_col99),
        eval!(context, blake_round_output_limb_14_col100),
        eval!(context, blake_round_output_limb_15_col101),
        eval!(context, blake_round_output_limb_16_col102),
        eval!(context, blake_round_output_limb_17_col103),
        eval!(context, blake_round_output_limb_18_col104),
        eval!(context, blake_round_output_limb_19_col105),
        eval!(context, blake_round_output_limb_20_col106),
        eval!(context, blake_round_output_limb_21_col107),
        eval!(context, blake_round_output_limb_22_col108),
        eval!(context, blake_round_output_limb_23_col109),
        eval!(context, blake_round_output_limb_24_col110),
        eval!(context, blake_round_output_limb_25_col111),
        eval!(context, blake_round_output_limb_26_col112),
        eval!(context, blake_round_output_limb_27_col113),
        eval!(context, blake_round_output_limb_28_col114),
        eval!(context, blake_round_output_limb_29_col115),
        eval!(context, blake_round_output_limb_30_col116),
        eval!(context, blake_round_output_limb_31_col117),
        eval!(context, blake_round_output_limb_32_col118),
    ];
    let numerator_3 = eval!(context, 1);
    acc.add_to_relation(context, numerator_3, tuple_3);

    create_blake_output::accumulate_constraints(
        &[
            eval!(context, input_state_before_limb0_limb_0_col0),
            eval!(context, input_state_before_limb0_limb_1_col1),
            eval!(context, input_state_before_limb1_limb_0_col2),
            eval!(context, input_state_before_limb1_limb_1_col3),
            eval!(context, input_state_before_limb2_limb_0_col4),
            eval!(context, input_state_before_limb2_limb_1_col5),
            eval!(context, input_state_before_limb3_limb_0_col6),
            eval!(context, input_state_before_limb3_limb_1_col7),
            eval!(context, input_state_before_limb4_limb_0_col8),
            eval!(context, input_state_before_limb4_limb_1_col9),
            eval!(context, input_state_before_limb5_limb_0_col10),
            eval!(context, input_state_before_limb5_limb_1_col11),
            eval!(context, input_state_before_limb6_limb_0_col12),
            eval!(context, input_state_before_limb6_limb_1_col13),
            eval!(context, input_state_before_limb7_limb_0_col14),
            eval!(context, input_state_before_limb7_limb_1_col15),
            eval!(context, blake_round_output_limb_0_col86),
            eval!(context, blake_round_output_limb_1_col87),
            eval!(context, blake_round_output_limb_2_col88),
            eval!(context, blake_round_output_limb_3_col89),
            eval!(context, blake_round_output_limb_4_col90),
            eval!(context, blake_round_output_limb_5_col91),
            eval!(context, blake_round_output_limb_6_col92),
            eval!(context, blake_round_output_limb_7_col93),
            eval!(context, blake_round_output_limb_8_col94),
            eval!(context, blake_round_output_limb_9_col95),
            eval!(context, blake_round_output_limb_10_col96),
            eval!(context, blake_round_output_limb_11_col97),
            eval!(context, blake_round_output_limb_12_col98),
            eval!(context, blake_round_output_limb_13_col99),
            eval!(context, blake_round_output_limb_14_col100),
            eval!(context, blake_round_output_limb_15_col101),
            eval!(context, blake_round_output_limb_16_col102),
            eval!(context, blake_round_output_limb_17_col103),
            eval!(context, blake_round_output_limb_18_col104),
            eval!(context, blake_round_output_limb_19_col105),
            eval!(context, blake_round_output_limb_20_col106),
            eval!(context, blake_round_output_limb_21_col107),
            eval!(context, blake_round_output_limb_22_col108),
            eval!(context, blake_round_output_limb_23_col109),
            eval!(context, blake_round_output_limb_24_col110),
            eval!(context, blake_round_output_limb_25_col111),
            eval!(context, blake_round_output_limb_26_col112),
            eval!(context, blake_round_output_limb_27_col113),
            eval!(context, blake_round_output_limb_28_col114),
            eval!(context, blake_round_output_limb_29_col115),
            eval!(context, blake_round_output_limb_30_col116),
            eval!(context, blake_round_output_limb_31_col117),
            eval!(context, triple_xor_32_output_limb_0_col119),
            eval!(context, triple_xor_32_output_limb_1_col120),
            eval!(context, triple_xor_32_output_limb_0_col121),
            eval!(context, triple_xor_32_output_limb_1_col122),
            eval!(context, triple_xor_32_output_limb_0_col123),
            eval!(context, triple_xor_32_output_limb_1_col124),
            eval!(context, triple_xor_32_output_limb_0_col125),
            eval!(context, triple_xor_32_output_limb_1_col126),
            eval!(context, triple_xor_32_output_limb_0_col127),
            eval!(context, triple_xor_32_output_limb_1_col128),
            eval!(context, triple_xor_32_output_limb_0_col129),
            eval!(context, triple_xor_32_output_limb_1_col130),
            eval!(context, triple_xor_32_output_limb_0_col131),
            eval!(context, triple_xor_32_output_limb_1_col132),
            eval!(context, triple_xor_32_output_limb_0_col133),
            eval!(context, triple_xor_32_output_limb_1_col134),
        ],
        context,
        component_data,
        acc,
    );

    //Blake output h[0].low() matches expected.
    let constraint_5_value = eval!(
        context,
        (triple_xor_32_output_limb_0_col119) - (input_state_after_limb0_limb_0_col16)
    );
    acc.add_constraint(context, constraint_5_value);

    //Blake output h[0].high() matches expected.
    let constraint_6_value = eval!(
        context,
        (triple_xor_32_output_limb_1_col120) - (input_state_after_limb0_limb_1_col17)
    );
    acc.add_constraint(context, constraint_6_value);

    //Blake output h[1].low() matches expected.
    let constraint_7_value = eval!(
        context,
        (triple_xor_32_output_limb_0_col121) - (input_state_after_limb1_limb_0_col18)
    );
    acc.add_constraint(context, constraint_7_value);

    //Blake output h[1].high() matches expected.
    let constraint_8_value = eval!(
        context,
        (triple_xor_32_output_limb_1_col122) - (input_state_after_limb1_limb_1_col19)
    );
    acc.add_constraint(context, constraint_8_value);

    //Blake output h[2].low() matches expected.
    let constraint_9_value = eval!(
        context,
        (triple_xor_32_output_limb_0_col123) - (input_state_after_limb2_limb_0_col20)
    );
    acc.add_constraint(context, constraint_9_value);

    //Blake output h[2].high() matches expected.
    let constraint_10_value = eval!(
        context,
        (triple_xor_32_output_limb_1_col124) - (input_state_after_limb2_limb_1_col21)
    );
    acc.add_constraint(context, constraint_10_value);

    //Blake output h[3].low() matches expected.
    let constraint_11_value = eval!(
        context,
        (triple_xor_32_output_limb_0_col125) - (input_state_after_limb3_limb_0_col22)
    );
    acc.add_constraint(context, constraint_11_value);

    //Blake output h[3].high() matches expected.
    let constraint_12_value = eval!(
        context,
        (triple_xor_32_output_limb_1_col126) - (input_state_after_limb3_limb_1_col23)
    );
    acc.add_constraint(context, constraint_12_value);

    //Blake output h[4].low() matches expected.
    let constraint_13_value = eval!(
        context,
        (triple_xor_32_output_limb_0_col127) - (input_state_after_limb4_limb_0_col24)
    );
    acc.add_constraint(context, constraint_13_value);

    //Blake output h[4].high() matches expected.
    let constraint_14_value = eval!(
        context,
        (triple_xor_32_output_limb_1_col128) - (input_state_after_limb4_limb_1_col25)
    );
    acc.add_constraint(context, constraint_14_value);

    //Blake output h[5].low() matches expected.
    let constraint_15_value = eval!(
        context,
        (triple_xor_32_output_limb_0_col129) - (input_state_after_limb5_limb_0_col26)
    );
    acc.add_constraint(context, constraint_15_value);

    //Blake output h[5].high() matches expected.
    let constraint_16_value = eval!(
        context,
        (triple_xor_32_output_limb_1_col130) - (input_state_after_limb5_limb_1_col27)
    );
    acc.add_constraint(context, constraint_16_value);

    //Blake output h[6].low() matches expected.
    let constraint_17_value = eval!(
        context,
        (triple_xor_32_output_limb_0_col131) - (input_state_after_limb6_limb_0_col28)
    );
    acc.add_constraint(context, constraint_17_value);

    //Blake output h[6].high() matches expected.
    let constraint_18_value = eval!(
        context,
        (triple_xor_32_output_limb_1_col132) - (input_state_after_limb6_limb_1_col29)
    );
    acc.add_constraint(context, constraint_18_value);

    //Blake output h[7].low() matches expected.
    let constraint_19_value = eval!(
        context,
        (triple_xor_32_output_limb_0_col133) - (input_state_after_limb7_limb_0_col30)
    );
    acc.add_constraint(context, constraint_19_value);

    //Blake output h[7].high() matches expected.
    let constraint_20_value = eval!(
        context,
        (triple_xor_32_output_limb_1_col134) - (input_state_after_limb7_limb_1_col31)
    );
    acc.add_constraint(context, constraint_20_value);

    // Use BlakeOutput.
    let tuple_21 = &[
        eval!(context, 1061955672),
        eval!(context, state_before_addr),
        eval!(context, input_state_before_limb0_limb_0_col0),
        eval!(context, input_state_before_limb0_limb_1_col1),
        eval!(context, input_state_before_limb1_limb_0_col2),
        eval!(context, input_state_before_limb1_limb_1_col3),
        eval!(context, input_state_before_limb2_limb_0_col4),
        eval!(context, input_state_before_limb2_limb_1_col5),
        eval!(context, input_state_before_limb3_limb_0_col6),
        eval!(context, input_state_before_limb3_limb_1_col7),
        eval!(context, input_state_before_limb4_limb_0_col8),
        eval!(context, input_state_before_limb4_limb_1_col9),
        eval!(context, input_state_before_limb5_limb_0_col10),
        eval!(context, input_state_before_limb5_limb_1_col11),
        eval!(context, input_state_before_limb6_limb_0_col12),
        eval!(context, input_state_before_limb6_limb_1_col13),
        eval!(context, input_state_before_limb7_limb_0_col14),
        eval!(context, input_state_before_limb7_limb_1_col15),
    ];
    let numerator_21 = eval!(context, blake_gate_enabler);
    acc.add_to_relation(context, numerator_21, tuple_21);

    // Yield BlakeOutput.
    let tuple_22 = &[
        eval!(context, 1061955672),
        eval!(context, state_after_addr),
        eval!(context, input_state_after_limb0_limb_0_col16),
        eval!(context, input_state_after_limb0_limb_1_col17),
        eval!(context, input_state_after_limb1_limb_0_col18),
        eval!(context, input_state_after_limb1_limb_1_col19),
        eval!(context, input_state_after_limb2_limb_0_col20),
        eval!(context, input_state_after_limb2_limb_1_col21),
        eval!(context, input_state_after_limb3_limb_0_col22),
        eval!(context, input_state_after_limb3_limb_1_col23),
        eval!(context, input_state_after_limb4_limb_0_col24),
        eval!(context, input_state_after_limb4_limb_1_col25),
        eval!(context, input_state_after_limb5_limb_0_col26),
        eval!(context, input_state_after_limb5_limb_1_col27),
        eval!(context, input_state_after_limb6_limb_0_col28),
        eval!(context, input_state_after_limb6_limb_1_col29),
        eval!(context, input_state_after_limb7_limb_0_col30),
        eval!(context, input_state_after_limb7_limb_1_col31),
    ];
    let numerator_22 = eval!(context, -(blake_gate_enabler));
    acc.add_to_relation(context, numerator_22, tuple_22);

    // Use Gate.
    let tuple_23 = &[
        eval!(context, 378353459),
        eval!(context, message0_addr),
        eval!(context, input_message_limb0_col32),
        eval!(context, input_message_limb1_col33),
        eval!(context, input_message_limb2_col34),
        eval!(context, input_message_limb3_col35),
    ];
    let numerator_23 = eval!(context, blake_gate_enabler);
    acc.add_to_relation(context, numerator_23, tuple_23);

    // Use Gate.
    let tuple_24 = &[
        eval!(context, 378353459),
        eval!(context, message1_addr),
        eval!(context, input_message_limb4_col36),
        eval!(context, input_message_limb5_col37),
        eval!(context, input_message_limb6_col38),
        eval!(context, input_message_limb7_col39),
    ];
    let numerator_24 = eval!(context, blake_gate_enabler);
    acc.add_to_relation(context, numerator_24, tuple_24);

    // Use Gate.
    let tuple_25 = &[
        eval!(context, 378353459),
        eval!(context, message2_addr),
        eval!(context, input_message_limb8_col40),
        eval!(context, input_message_limb9_col41),
        eval!(context, input_message_limb10_col42),
        eval!(context, input_message_limb11_col43),
    ];
    let numerator_25 = eval!(context, blake_gate_enabler);
    acc.add_to_relation(context, numerator_25, tuple_25);

    // Use Gate.
    let tuple_26 = &[
        eval!(context, 378353459),
        eval!(context, message3_addr),
        eval!(context, input_message_limb12_col44),
        eval!(context, input_message_limb13_col45),
        eval!(context, input_message_limb14_col46),
        eval!(context, input_message_limb15_col47),
    ];
    let numerator_26 = eval!(context, blake_gate_enabler);
    acc.add_to_relation(context, numerator_26, tuple_26);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "blake_gate".to_string()
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
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use stwo::core::fields::qm31::QM31;

    #[allow(unused_imports)]
    use crate::components::prelude::PreProcessedColumnId;
    use crate::sample_evaluations::*;
    use crate::test::TestComponentData;
    use circuits::context::Context;
    use circuits::ivalue::qm31_from_u32s;
    use circuits_stark_verifier::constraint_eval::*;

    use super::Component;

    #[test]
    fn test_evaluation_result() {
        let component = Component {};
        let mut context: Context<QM31> = Default::default();
        context.enable_assert_eq_on_eval();
        let trace_columns = [
            qm31_from_u32s(1659099300, 905558730, 651199673, 1375009625),
            qm31_from_u32s(1591990121, 771341002, 584090809, 1375009625),
            qm31_from_u32s(1793317658, 1173994186, 785417401, 1375009625),
            qm31_from_u32s(1726208479, 1039776458, 718308537, 1375009625),
            qm31_from_u32s(1390662584, 368687818, 382764217, 1375009625),
            qm31_from_u32s(1323553405, 234470090, 315655353, 1375009625),
            qm31_from_u32s(1524880942, 637123274, 516981945, 1375009625),
            qm31_from_u32s(1457771763, 502905546, 449873081, 1375009625),
            qm31_from_u32s(48489085, 1979300555, 1188070585, 1375009625),
            qm31_from_u32s(2128863553, 1845082826, 1120961721, 1375009625),
            qm31_from_u32s(1852335767, 645078115, 2059236183, 343880121),
            qm31_from_u32s(1919444946, 779295843, 2126345047, 343880121),
            qm31_from_u32s(1986554125, 913513571, 45970264, 343880122),
            qm31_from_u32s(2053663304, 1047731299, 113079128, 343880122),
            qm31_from_u32s(1583899051, 108207203, 1790800727, 343880121),
            qm31_from_u32s(1651008230, 242424931, 1857909591, 343880121),
            qm31_from_u32s(1718117409, 376642659, 1925018455, 343880121),
            qm31_from_u32s(1785226588, 510860387, 1992127319, 343880121),
            qm31_from_u32s(1315462335, 1718819938, 1522365270, 343880121),
            qm31_from_u32s(1382571514, 1853037666, 1589474134, 343880121),
            qm31_from_u32s(1986820986, 913513739, 45970432, 343880178),
            qm31_from_u32s(1919711807, 779296011, 2126345215, 343880177),
            qm31_from_u32s(2121039344, 1181949195, 180188160, 343880178),
            qm31_from_u32s(2053930165, 1047731467, 113079296, 343880178),
            qm31_from_u32s(1718384270, 376642827, 1925018623, 343880177),
            qm31_from_u32s(1651275091, 242425099, 1857909759, 343880177),
            qm31_from_u32s(1852602628, 645078283, 2059236351, 343880177),
            qm31_from_u32s(1785493449, 510860555, 1992127487, 343880177),
            qm31_from_u32s(1449947554, 1987255562, 1656583166, 343880177),
            qm31_from_u32s(1382838375, 1853037834, 1589474302, 343880177),
            qm31_from_u32s(510356977, 108207322, 717059022, 343880161),
            qm31_from_u32s(577466156, 242425050, 784167886, 343880161),
            qm31_from_u32s(376138619, 1987255513, 582841293, 343880161),
            qm31_from_u32s(443247798, 2121473241, 649950157, 343880161),
            qm31_from_u32s(778793693, 645078234, 985494478, 343880161),
            qm31_from_u32s(845902872, 779295962, 1052603342, 343880161),
            qm31_from_u32s(644575335, 376642778, 851276750, 343880161),
            qm31_from_u32s(711684514, 510860506, 918385614, 343880161),
            qm31_from_u32s(1047230409, 1181949146, 1253929934, 343880161),
            qm31_from_u32s(1114339588, 1316166874, 1321038798, 343880161),
            qm31_from_u32s(1717810224, 376642479, 1925018275, 343880061),
            qm31_from_u32s(1650701045, 242424751, 1857909411, 343880061),
            qm31_from_u32s(1583591866, 108207023, 1790800547, 343880061),
            qm31_from_u32s(1516482687, 2121472942, 1723691682, 343880061),
            qm31_from_u32s(1986246940, 913513391, 45970084, 343880062),
            qm31_from_u32s(1919137761, 779295663, 2126344867, 343880061),
            qm31_from_u32s(1852028582, 645077935, 2059236003, 343880061),
            qm31_from_u32s(1784919403, 510860207, 1992127139, 343880061),
            qm31_from_u32s(1180936792, 1450384302, 1388147362, 343880061),
            qm31_from_u32s(1113827613, 1316166574, 1321038498, 343880061),
            qm31_from_u32s(241305891, 1718819697, 448623205, 343880041),
            qm31_from_u32s(308415070, 1853037425, 515732069, 343880041),
            qm31_from_u32s(375524249, 1987255153, 582840933, 343880041),
            qm31_from_u32s(442633428, 2121472881, 649949797, 343880041),
            qm31_from_u32s(509742607, 108206962, 717058662, 343880041),
            qm31_from_u32s(576851786, 242424690, 784167526, 343880041),
            qm31_from_u32s(643960965, 376642418, 851276390, 343880041),
            qm31_from_u32s(711070144, 510860146, 918385254, 343880041),
            qm31_from_u32s(778179323, 645077874, 985494118, 343880041),
            qm31_from_u32s(845288502, 779295602, 1052602982, 343880041),
            qm31_from_u32s(375831434, 1987255333, 582841113, 343880101),
            qm31_from_u32s(308722255, 1853037605, 515732249, 343880101),
            qm31_from_u32s(510049792, 108207142, 717058842, 343880101),
            qm31_from_u32s(442940613, 2121473061, 649949977, 343880101),
            qm31_from_u32s(644268150, 376642598, 851276570, 343880101),
            qm31_from_u32s(577158971, 242424870, 784167706, 343880101),
            qm31_from_u32s(778486508, 645078054, 985494298, 343880101),
            qm31_from_u32s(711377329, 510860326, 918385434, 343880101),
            qm31_from_u32s(912704866, 913513510, 1119712026, 343880101),
            qm31_from_u32s(845595687, 779295782, 1052603162, 343880101),
            qm31_from_u32s(1046820829, 1181948906, 1253929694, 343880081),
            qm31_from_u32s(1113930008, 1316166634, 1321038558, 343880081),
            qm31_from_u32s(912602471, 913513450, 1119711966, 343880081),
            qm31_from_u32s(979711650, 1047731178, 1186820830, 343880081),
            qm31_from_u32s(778384113, 645077994, 985494238, 343880081),
            qm31_from_u32s(845493292, 779295722, 1052603102, 343880081),
            qm31_from_u32s(644165755, 376642538, 851276510, 343880081),
            qm31_from_u32s(711274934, 510860266, 918385374, 343880081),
            qm31_from_u32s(1583694261, 108207083, 1790800607, 343880081),
            qm31_from_u32s(1650803440, 242424811, 1857909471, 343880081),
            qm31_from_u32s(108388425, 1450385012, 314406248, 343880298),
            qm31_from_u32s(41279246, 1316167284, 247297384, 343880298),
            qm31_from_u32s(2121653714, 1181949555, 180188520, 343880298),
            qm31_from_u32s(2054544535, 1047731827, 113079656, 343880298),
            qm31_from_u32s(1987435356, 913514099, 45970792, 343880298),
            qm31_from_u32s(1920326177, 779296371, 2126345575, 343880297),
            qm31_from_u32s(1853216998, 645078643, 2059236711, 343880297),
            qm31_from_u32s(1786107819, 510860915, 1992127847, 343880297),
            qm31_from_u32s(1718998640, 376643187, 1925018983, 343880297),
            qm31_from_u32s(1651889461, 242425459, 1857910119, 343880297),
            qm31_from_u32s(779367739, 645078582, 985494826, 343880277),
            qm31_from_u32s(846476918, 779296310, 1052603690, 343880277),
            qm31_from_u32s(913586097, 913514038, 1119712554, 343880277),
            qm31_from_u32s(980695276, 1047731766, 1186821418, 343880277),
            qm31_from_u32s(510931023, 108207670, 717059370, 343880277),
            qm31_from_u32s(578040202, 242425398, 784168234, 343880277),
            qm31_from_u32s(645149381, 376643126, 851277098, 343880277),
            qm31_from_u32s(712258560, 510860854, 918385962, 343880277),
            qm31_from_u32s(1316241171, 1718820406, 1522365738, 343880277),
            qm31_from_u32s(1383350350, 1853038134, 1589474602, 343880277),
            qm31_from_u32s(1340598866, 536394231, 1198633759, 502514173),
            qm31_from_u32s(1407708045, 670611959, 1265742623, 502514173),
            qm31_from_u32s(1474817224, 804829687, 1332851487, 502514173),
            qm31_from_u32s(1541926403, 939047415, 1399960351, 502514173),
            qm31_from_u32s(1072162150, 2147006966, 930198302, 502514173),
            qm31_from_u32s(1139271329, 133741047, 997307167, 502514173),
            qm31_from_u32s(1206380508, 267958775, 1064416031, 502514173),
            qm31_from_u32s(1273489687, 402176503, 1131524895, 502514173),
            qm31_from_u32s(1877472298, 1610136055, 1735504671, 502514173),
            qm31_from_u32s(1944581477, 1744353783, 1802613535, 502514173),
            qm31_from_u32s(669619552, 1341700661, 527545181, 502514194),
            qm31_from_u32s(602510373, 1207482933, 460436317, 502514194),
            qm31_from_u32s(535401194, 1073265205, 393327453, 502514194),
            qm31_from_u32s(468292015, 939047477, 326218589, 502514194),
            qm31_from_u32s(401182836, 804829749, 259109725, 502514194),
            qm31_from_u32s(334073657, 670612021, 192000861, 502514194),
            qm31_from_u32s(266964478, 536394293, 124891997, 502514194),
            qm31_from_u32s(199855299, 402176565, 57783133, 502514194),
            qm31_from_u32s(132746120, 267958837, 2138157916, 502514193),
            qm31_from_u32s(65636941, 133741109, 2071049052, 502514193),
            qm31_from_u32s(2146113804, 2147007087, 2003940247, 502514213),
            qm31_from_u32s(65739336, 133741169, 2071049112, 502514213),
            qm31_from_u32s(2011895446, 1878571631, 1869722519, 502514213),
            qm31_from_u32s(2079004625, 2012789359, 1936831383, 502514213),
            qm31_from_u32s(267066873, 536394353, 124892057, 502514214),
            qm31_from_u32s(334176052, 670612081, 192000921, 502514214),
            qm31_from_u32s(132848515, 267958897, 2138157976, 502514213),
            qm31_from_u32s(199957694, 402176625, 57783193, 502514214),
            qm31_from_u32s(1609240372, 1073265263, 1467069335, 502514213),
            qm31_from_u32s(1676349551, 1207482991, 1534178199, 502514213),
            qm31_from_u32s(1475124409, 804829867, 1332851667, 502514233),
            qm31_from_u32s(1408015230, 670612139, 1265742803, 502514233),
            qm31_from_u32s(1609342767, 1073265323, 1467069395, 502514233),
            qm31_from_u32s(1542233588, 939047595, 1399960531, 502514233),
            qm31_from_u32s(902525010, 1115155995, 130434373, 2116865290),
        ];
        let interaction_columns = [
            qm31_from_u32s(1005168032, 79980996, 1847888101, 1941984119),
            qm31_from_u32s(1072277211, 214198724, 1914996965, 1941984119),
            qm31_from_u32s(1139386390, 348416452, 1982105829, 1941984119),
            qm31_from_u32s(1206495569, 482634180, 2049214693, 1941984119),
            qm31_from_u32s(736731316, 1690593731, 1579452644, 1941984119),
            qm31_from_u32s(803840495, 1824811459, 1646561508, 1941984119),
            qm31_from_u32s(870949674, 1959029187, 1713670372, 1941984119),
            qm31_from_u32s(938058853, 2093246915, 1780779236, 1941984119),
            qm31_from_u32s(1542041464, 1153722820, 237275366, 1941984120),
            qm31_from_u32s(1609150643, 1287940548, 304384230, 1941984120),
            qm31_from_u32s(1577898798, 106101108, 1738096752, 1261630210),
            qm31_from_u32s(1510789619, 2119367027, 1670987887, 1261630210),
            qm31_from_u32s(1443680440, 1985149299, 1603879023, 1261630210),
            qm31_from_u32s(1376571261, 1850931571, 1536770159, 1261630210),
            qm31_from_u32s(1309462082, 1716713843, 1469661295, 1261630210),
            qm31_from_u32s(1242352903, 1582496115, 1402552431, 1261630210),
            qm31_from_u32s(1175243724, 1448278387, 1335443567, 1261630210),
            qm31_from_u32s(1108134545, 1314060659, 1268334703, 1261630210),
            qm31_from_u32s(2114772230, 1179842932, 127484017, 1261630211),
            qm31_from_u32s(2047663051, 1045625204, 60375153, 1261630211),
            qm31_from_u32s(906909403, 911407535, 1067008171, 1261630230),
            qm31_from_u32s(974018582, 1045625263, 1134117035, 1261630230),
            qm31_from_u32s(772691045, 642972079, 932790443, 1261630230),
            qm31_from_u32s(839800224, 777189807, 999899307, 1261630230),
            qm31_from_u32s(1175346119, 1448278447, 1335443627, 1261630230),
            qm31_from_u32s(1242455298, 1582496175, 1402552491, 1261630230),
            qm31_from_u32s(1041127761, 1179842991, 1201225899, 1261630230),
            qm31_from_u32s(1108236940, 1314060719, 1268334763, 1261630230),
            qm31_from_u32s(1443782835, 1985149359, 1603879083, 1261630230),
            qm31_from_u32s(1510892014, 2119367087, 1670987947, 1261630230),
            qm31_from_u32s(235889765, 1716713953, 395919581, 1261630247),
            qm31_from_u32s(168780586, 1582496225, 328810717, 1261630247),
            qm31_from_u32s(370108123, 1985149409, 530137309, 1261630247),
            qm31_from_u32s(302998944, 1850931681, 463028445, 1261630247),
        ];
        let component_data = TestComponentData::from_values(
            &mut context,
            &trace_columns,
            &interaction_columns,
            qm31_from_u32s(1115374022, 1127856551, 489657863, 643630026),
            qm31_from_u32s(1398335417, 314974026, 1722107152, 821933968),
            32768,
        );
        let random_coeff =
            context.new_var(qm31_from_u32s(474642921, 876336632, 1911695779, 974600512));
        let interaction_elements = [
            context.new_var(qm31_from_u32s(445623802, 202571636, 1360224996, 131355117)),
            context.new_var(qm31_from_u32s(476823935, 939223384, 62486082, 122423602)),
        ];
        let preprocessed_columns = HashMap::from([
            (
                PreProcessedColumnId { id: "t0".to_owned() },
                context.constant(qm31_from_u32s(1841080911, 1051182885, 1790603552, 1141156649)),
            ),
            (
                PreProcessedColumnId { id: "t1".to_owned() },
                context.constant(qm31_from_u32s(1773971732, 916965157, 1723494688, 1141156649)),
            ),
            (
                PreProcessedColumnId { id: "finalize_flag".to_owned() },
                context.constant(qm31_from_u32s(1527621812, 1359459191, 1300848279, 1424553637)),
            ),
            (
                PreProcessedColumnId { id: "seq_15".to_owned() },
                context.constant(qm31_from_u32s(735272696, 1215403647, 795393303, 879304430)),
            ),
            (
                PreProcessedColumnId { id: "state_before_addr".to_owned() },
                context.constant(qm31_from_u32s(112055314, 652805097, 890017957, 1059987843)),
            ),
            (
                PreProcessedColumnId { id: "blake_gate_enabler".to_owned() },
                context.constant(qm31_from_u32s(1354572623, 214328864, 1486167532, 984653619)),
            ),
            (
                PreProcessedColumnId { id: "state_after_addr".to_owned() },
                context.constant(qm31_from_u32s(1835257341, 625501858, 928280751, 1965247051)),
            ),
            (
                PreProcessedColumnId { id: "message0_addr".to_owned() },
                context.constant(qm31_from_u32s(1672755108, 1335465101, 150577708, 945587926)),
            ),
            (
                PreProcessedColumnId { id: "message1_addr".to_owned() },
                context.constant(qm31_from_u32s(559581411, 1820988583, 98946663, 749420627)),
            ),
            (
                PreProcessedColumnId { id: "message2_addr".to_owned() },
                context.constant(qm31_from_u32s(1220991681, 1856768612, 1476249475, 224258229)),
            ),
            (
                PreProcessedColumnId { id: "message3_addr".to_owned() },
                context.constant(qm31_from_u32s(19392245, 526654392, 682814369, 854564706)),
            ),
        ]);
        let public_params = HashMap::from([]);
        let mut accumulator = CompositionConstraintAccumulator::new(
            &mut context,
            preprocessed_columns,
            public_params,
            random_coeff,
            interaction_elements,
        );
        accumulator.set_enable_bit(context.one());
        component.evaluate(&mut context, &component_data, &mut accumulator);
        accumulator.finalize_logup_in_pairs(
            &mut context,
            <TestComponentData as ComponentDataTrait<QM31>>::interaction_columns(&component_data),
            &component_data,
        );

        let result = accumulator.finalize();
        let result_value = context.get(result);
        assert_eq!(result_value, BLAKE_GATE_SAMPLE_EVAL_RESULT)
    }
}
