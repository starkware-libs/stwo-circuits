use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 151;
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
        limbi_inv_or_one_col56,
        limbi_low_col57,
        limbi_high_col58,
        limbi_inv_or_one_col59,
        limbi_low_col60,
        limbi_high_col61,
        limbi_inv_or_one_col62,
        limbi_low_col63,
        limbi_high_col64,
        limbi_inv_or_one_col65,
        limbi_low_col66,
        limbi_high_col67,
        limbi_inv_or_one_col68,
        limbi_low_col69,
        limbi_high_col70,
        limbi_inv_or_one_col71,
        limbi_low_col72,
        limbi_high_col73,
        limbi_inv_or_one_col74,
        limbi_low_col75,
        limbi_high_col76,
        limbi_inv_or_one_col77,
        limbi_low_col78,
        limbi_high_col79,
        limbi_inv_or_one_col80,
        limbi_low_col81,
        limbi_high_col82,
        limbi_inv_or_one_col83,
        limbi_low_col84,
        limbi_high_col85,
        limbi_inv_or_one_col86,
        limbi_low_col87,
        limbi_high_col88,
        limbi_inv_or_one_col89,
        limbi_low_col90,
        limbi_high_col91,
        limbi_inv_or_one_col92,
        limbi_low_col93,
        limbi_high_col94,
        limbi_inv_or_one_col95,
        limbi_low_col96,
        limbi_high_col97,
        limbi_inv_or_one_col98,
        limbi_low_col99,
        limbi_high_col100,
        limbi_inv_or_one_col101,
        blake_round_output_limb_0_col102,
        blake_round_output_limb_1_col103,
        blake_round_output_limb_2_col104,
        blake_round_output_limb_3_col105,
        blake_round_output_limb_4_col106,
        blake_round_output_limb_5_col107,
        blake_round_output_limb_6_col108,
        blake_round_output_limb_7_col109,
        blake_round_output_limb_8_col110,
        blake_round_output_limb_9_col111,
        blake_round_output_limb_10_col112,
        blake_round_output_limb_11_col113,
        blake_round_output_limb_12_col114,
        blake_round_output_limb_13_col115,
        blake_round_output_limb_14_col116,
        blake_round_output_limb_15_col117,
        blake_round_output_limb_16_col118,
        blake_round_output_limb_17_col119,
        blake_round_output_limb_18_col120,
        blake_round_output_limb_19_col121,
        blake_round_output_limb_20_col122,
        blake_round_output_limb_21_col123,
        blake_round_output_limb_22_col124,
        blake_round_output_limb_23_col125,
        blake_round_output_limb_24_col126,
        blake_round_output_limb_25_col127,
        blake_round_output_limb_26_col128,
        blake_round_output_limb_27_col129,
        blake_round_output_limb_28_col130,
        blake_round_output_limb_29_col131,
        blake_round_output_limb_30_col132,
        blake_round_output_limb_31_col133,
        blake_round_output_limb_32_col134,
        triple_xor_32_output_limb_0_col135,
        triple_xor_32_output_limb_1_col136,
        triple_xor_32_output_limb_0_col137,
        triple_xor_32_output_limb_1_col138,
        triple_xor_32_output_limb_0_col139,
        triple_xor_32_output_limb_1_col140,
        triple_xor_32_output_limb_0_col141,
        triple_xor_32_output_limb_1_col142,
        triple_xor_32_output_limb_0_col143,
        triple_xor_32_output_limb_1_col144,
        triple_xor_32_output_limb_0_col145,
        triple_xor_32_output_limb_1_col146,
        triple_xor_32_output_limb_0_col147,
        triple_xor_32_output_limb_1_col148,
        triple_xor_32_output_limb_0_col149,
        triple_xor_32_output_limb_1_col150,
    ] = input.try_into().unwrap();
    let seq = component_data.seq_of_component_size(context, &acc.preprocessed_columns);
    let compress_enabler =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "compress_enabler".to_owned() });
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
            eval!(context, limbi_inv_or_one_col56),
            eval!(context, limbi_low_col57),
            eval!(context, limbi_high_col58),
            eval!(context, limbi_inv_or_one_col59),
            eval!(context, limbi_low_col60),
            eval!(context, limbi_high_col61),
            eval!(context, limbi_inv_or_one_col62),
            eval!(context, limbi_low_col63),
            eval!(context, limbi_high_col64),
            eval!(context, limbi_inv_or_one_col65),
            eval!(context, limbi_low_col66),
            eval!(context, limbi_high_col67),
            eval!(context, limbi_inv_or_one_col68),
            eval!(context, limbi_low_col69),
            eval!(context, limbi_high_col70),
            eval!(context, limbi_inv_or_one_col71),
            eval!(context, limbi_low_col72),
            eval!(context, limbi_high_col73),
            eval!(context, limbi_inv_or_one_col74),
            eval!(context, limbi_low_col75),
            eval!(context, limbi_high_col76),
            eval!(context, limbi_inv_or_one_col77),
            eval!(context, limbi_low_col78),
            eval!(context, limbi_high_col79),
            eval!(context, limbi_inv_or_one_col80),
            eval!(context, limbi_low_col81),
            eval!(context, limbi_high_col82),
            eval!(context, limbi_inv_or_one_col83),
            eval!(context, limbi_low_col84),
            eval!(context, limbi_high_col85),
            eval!(context, limbi_inv_or_one_col86),
            eval!(context, limbi_low_col87),
            eval!(context, limbi_high_col88),
            eval!(context, limbi_inv_or_one_col89),
            eval!(context, limbi_low_col90),
            eval!(context, limbi_high_col91),
            eval!(context, limbi_inv_or_one_col92),
            eval!(context, limbi_low_col93),
            eval!(context, limbi_high_col94),
            eval!(context, limbi_inv_or_one_col95),
            eval!(context, limbi_low_col96),
            eval!(context, limbi_high_col97),
            eval!(context, limbi_inv_or_one_col98),
            eval!(context, limbi_low_col99),
            eval!(context, limbi_high_col100),
            eval!(context, limbi_inv_or_one_col101),
        ],
        context,
        component_data,
        compress_enabler,
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
    let numerator_2 = eval!(context, -(compress_enabler));
    acc.add_to_relation(context, numerator_2, tuple_2);

    // Use BlakeRound.
    let tuple_3 = &[
        eval!(context, 40528774),
        eval!(context, seq),
        eval!(context, 10),
        eval!(context, blake_round_output_limb_0_col102),
        eval!(context, blake_round_output_limb_1_col103),
        eval!(context, blake_round_output_limb_2_col104),
        eval!(context, blake_round_output_limb_3_col105),
        eval!(context, blake_round_output_limb_4_col106),
        eval!(context, blake_round_output_limb_5_col107),
        eval!(context, blake_round_output_limb_6_col108),
        eval!(context, blake_round_output_limb_7_col109),
        eval!(context, blake_round_output_limb_8_col110),
        eval!(context, blake_round_output_limb_9_col111),
        eval!(context, blake_round_output_limb_10_col112),
        eval!(context, blake_round_output_limb_11_col113),
        eval!(context, blake_round_output_limb_12_col114),
        eval!(context, blake_round_output_limb_13_col115),
        eval!(context, blake_round_output_limb_14_col116),
        eval!(context, blake_round_output_limb_15_col117),
        eval!(context, blake_round_output_limb_16_col118),
        eval!(context, blake_round_output_limb_17_col119),
        eval!(context, blake_round_output_limb_18_col120),
        eval!(context, blake_round_output_limb_19_col121),
        eval!(context, blake_round_output_limb_20_col122),
        eval!(context, blake_round_output_limb_21_col123),
        eval!(context, blake_round_output_limb_22_col124),
        eval!(context, blake_round_output_limb_23_col125),
        eval!(context, blake_round_output_limb_24_col126),
        eval!(context, blake_round_output_limb_25_col127),
        eval!(context, blake_round_output_limb_26_col128),
        eval!(context, blake_round_output_limb_27_col129),
        eval!(context, blake_round_output_limb_28_col130),
        eval!(context, blake_round_output_limb_29_col131),
        eval!(context, blake_round_output_limb_30_col132),
        eval!(context, blake_round_output_limb_31_col133),
        eval!(context, blake_round_output_limb_32_col134),
    ];
    let numerator_3 = eval!(context, compress_enabler);
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
            eval!(context, blake_round_output_limb_0_col102),
            eval!(context, blake_round_output_limb_1_col103),
            eval!(context, blake_round_output_limb_2_col104),
            eval!(context, blake_round_output_limb_3_col105),
            eval!(context, blake_round_output_limb_4_col106),
            eval!(context, blake_round_output_limb_5_col107),
            eval!(context, blake_round_output_limb_6_col108),
            eval!(context, blake_round_output_limb_7_col109),
            eval!(context, blake_round_output_limb_8_col110),
            eval!(context, blake_round_output_limb_9_col111),
            eval!(context, blake_round_output_limb_10_col112),
            eval!(context, blake_round_output_limb_11_col113),
            eval!(context, blake_round_output_limb_12_col114),
            eval!(context, blake_round_output_limb_13_col115),
            eval!(context, blake_round_output_limb_14_col116),
            eval!(context, blake_round_output_limb_15_col117),
            eval!(context, blake_round_output_limb_16_col118),
            eval!(context, blake_round_output_limb_17_col119),
            eval!(context, blake_round_output_limb_18_col120),
            eval!(context, blake_round_output_limb_19_col121),
            eval!(context, blake_round_output_limb_20_col122),
            eval!(context, blake_round_output_limb_21_col123),
            eval!(context, blake_round_output_limb_22_col124),
            eval!(context, blake_round_output_limb_23_col125),
            eval!(context, blake_round_output_limb_24_col126),
            eval!(context, blake_round_output_limb_25_col127),
            eval!(context, blake_round_output_limb_26_col128),
            eval!(context, blake_round_output_limb_27_col129),
            eval!(context, blake_round_output_limb_28_col130),
            eval!(context, blake_round_output_limb_29_col131),
            eval!(context, blake_round_output_limb_30_col132),
            eval!(context, blake_round_output_limb_31_col133),
            eval!(context, triple_xor_32_output_limb_0_col135),
            eval!(context, triple_xor_32_output_limb_1_col136),
            eval!(context, triple_xor_32_output_limb_0_col137),
            eval!(context, triple_xor_32_output_limb_1_col138),
            eval!(context, triple_xor_32_output_limb_0_col139),
            eval!(context, triple_xor_32_output_limb_1_col140),
            eval!(context, triple_xor_32_output_limb_0_col141),
            eval!(context, triple_xor_32_output_limb_1_col142),
            eval!(context, triple_xor_32_output_limb_0_col143),
            eval!(context, triple_xor_32_output_limb_1_col144),
            eval!(context, triple_xor_32_output_limb_0_col145),
            eval!(context, triple_xor_32_output_limb_1_col146),
            eval!(context, triple_xor_32_output_limb_0_col147),
            eval!(context, triple_xor_32_output_limb_1_col148),
            eval!(context, triple_xor_32_output_limb_0_col149),
            eval!(context, triple_xor_32_output_limb_1_col150),
        ],
        context,
        component_data,
        compress_enabler,
        acc,
    );

    //Blake output h[0].low() matches expected.
    let constraint_5_value = eval!(
        context,
        (triple_xor_32_output_limb_0_col135) - (input_state_after_limb0_limb_0_col16)
    );
    acc.add_constraint(context, constraint_5_value);

    //Blake output h[0].high() matches expected.
    let constraint_6_value = eval!(
        context,
        (triple_xor_32_output_limb_1_col136) - (input_state_after_limb0_limb_1_col17)
    );
    acc.add_constraint(context, constraint_6_value);

    //Blake output h[1].low() matches expected.
    let constraint_7_value = eval!(
        context,
        (triple_xor_32_output_limb_0_col137) - (input_state_after_limb1_limb_0_col18)
    );
    acc.add_constraint(context, constraint_7_value);

    //Blake output h[1].high() matches expected.
    let constraint_8_value = eval!(
        context,
        (triple_xor_32_output_limb_1_col138) - (input_state_after_limb1_limb_1_col19)
    );
    acc.add_constraint(context, constraint_8_value);

    //Blake output h[2].low() matches expected.
    let constraint_9_value = eval!(
        context,
        (triple_xor_32_output_limb_0_col139) - (input_state_after_limb2_limb_0_col20)
    );
    acc.add_constraint(context, constraint_9_value);

    //Blake output h[2].high() matches expected.
    let constraint_10_value = eval!(
        context,
        (triple_xor_32_output_limb_1_col140) - (input_state_after_limb2_limb_1_col21)
    );
    acc.add_constraint(context, constraint_10_value);

    //Blake output h[3].low() matches expected.
    let constraint_11_value = eval!(
        context,
        (triple_xor_32_output_limb_0_col141) - (input_state_after_limb3_limb_0_col22)
    );
    acc.add_constraint(context, constraint_11_value);

    //Blake output h[3].high() matches expected.
    let constraint_12_value = eval!(
        context,
        (triple_xor_32_output_limb_1_col142) - (input_state_after_limb3_limb_1_col23)
    );
    acc.add_constraint(context, constraint_12_value);

    //Blake output h[4].low() matches expected.
    let constraint_13_value = eval!(
        context,
        (triple_xor_32_output_limb_0_col143) - (input_state_after_limb4_limb_0_col24)
    );
    acc.add_constraint(context, constraint_13_value);

    //Blake output h[4].high() matches expected.
    let constraint_14_value = eval!(
        context,
        (triple_xor_32_output_limb_1_col144) - (input_state_after_limb4_limb_1_col25)
    );
    acc.add_constraint(context, constraint_14_value);

    //Blake output h[5].low() matches expected.
    let constraint_15_value = eval!(
        context,
        (triple_xor_32_output_limb_0_col145) - (input_state_after_limb5_limb_0_col26)
    );
    acc.add_constraint(context, constraint_15_value);

    //Blake output h[5].high() matches expected.
    let constraint_16_value = eval!(
        context,
        (triple_xor_32_output_limb_1_col146) - (input_state_after_limb5_limb_1_col27)
    );
    acc.add_constraint(context, constraint_16_value);

    //Blake output h[6].low() matches expected.
    let constraint_17_value = eval!(
        context,
        (triple_xor_32_output_limb_0_col147) - (input_state_after_limb6_limb_0_col28)
    );
    acc.add_constraint(context, constraint_17_value);

    //Blake output h[6].high() matches expected.
    let constraint_18_value = eval!(
        context,
        (triple_xor_32_output_limb_1_col148) - (input_state_after_limb6_limb_1_col29)
    );
    acc.add_constraint(context, constraint_18_value);

    //Blake output h[7].low() matches expected.
    let constraint_19_value = eval!(
        context,
        (triple_xor_32_output_limb_0_col149) - (input_state_after_limb7_limb_0_col30)
    );
    acc.add_constraint(context, constraint_19_value);

    //Blake output h[7].high() matches expected.
    let constraint_20_value = eval!(
        context,
        (triple_xor_32_output_limb_1_col150) - (input_state_after_limb7_limb_1_col31)
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
    let numerator_21 = eval!(context, compress_enabler);
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
    let numerator_22 = eval!(context, -(compress_enabler));
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
    let numerator_23 = eval!(context, compress_enabler);
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
    let numerator_24 = eval!(context, compress_enabler);
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
    let numerator_25 = eval!(context, compress_enabler);
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
    let numerator_26 = eval!(context, compress_enabler);
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
