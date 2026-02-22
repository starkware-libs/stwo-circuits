// This file was created by the AIR team.

use super::super::prelude::*;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] =
    [RelationUse { relation_id: "VerifyBitwiseXor_8", uses: 4 }];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Vec<Var> {
    let [
        create_blake_round_input_input_limb_0,
        create_blake_round_input_input_limb_1,
        create_blake_round_input_input_limb_2,
        create_blake_round_input_input_limb_3,
        create_blake_round_input_input_limb_4,
        create_blake_round_input_input_limb_5,
        create_blake_round_input_input_limb_6,
        create_blake_round_input_input_limb_7,
        create_blake_round_input_input_limb_8,
        create_blake_round_input_input_limb_9,
        create_blake_round_input_input_limb_10,
        create_blake_round_input_input_limb_11,
        create_blake_round_input_input_limb_12,
        create_blake_round_input_input_limb_13,
        create_blake_round_input_input_limb_14,
        create_blake_round_input_input_limb_15,
        create_blake_round_input_input_limb_16,
        ms_8_bits_col0,
        ms_8_bits_col1,
        xor_col2,
        xor_col3,
        xor_col4,
        xor_col5,
    ] = input.try_into().unwrap();
    let t0 = acc.get_preprocessed_column(&PreProcessedColumnId { id: "t0".to_owned() });
    let t1 = acc.get_preprocessed_column(&PreProcessedColumnId { id: "t1".to_owned() });

    let [split_16_low_part_size_8_output_tmp_4d188_1_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[eval!(context, t0), eval!(context, ms_8_bits_col0)],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_8_output_tmp_4d188_3_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[eval!(context, t1), eval!(context, ms_8_bits_col1)],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    bitwise_xor_num_bits_8::accumulate_constraints(
        &[
            eval!(context, split_16_low_part_size_8_output_tmp_4d188_1_limb_0),
            eval!(context, 127),
            eval!(context, xor_col2),
        ],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_8::accumulate_constraints(
        &[eval!(context, ms_8_bits_col0), eval!(context, 82), eval!(context, xor_col3)],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_8::accumulate_constraints(
        &[
            eval!(context, split_16_low_part_size_8_output_tmp_4d188_3_limb_0),
            eval!(context, 14),
            eval!(context, xor_col4),
        ],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_8::accumulate_constraints(
        &[eval!(context, ms_8_bits_col1), eval!(context, 81), eval!(context, xor_col5)],
        context,
        component_data,
        acc,
    );
    vec![
        eval!(context, create_blake_round_input_input_limb_0),
        eval!(context, create_blake_round_input_input_limb_1),
        eval!(context, create_blake_round_input_input_limb_2),
        eval!(context, create_blake_round_input_input_limb_3),
        eval!(context, create_blake_round_input_input_limb_4),
        eval!(context, create_blake_round_input_input_limb_5),
        eval!(context, create_blake_round_input_input_limb_6),
        eval!(context, create_blake_round_input_input_limb_7),
        eval!(context, create_blake_round_input_input_limb_8),
        eval!(context, create_blake_round_input_input_limb_9),
        eval!(context, create_blake_round_input_input_limb_10),
        eval!(context, create_blake_round_input_input_limb_11),
        eval!(context, create_blake_round_input_input_limb_12),
        eval!(context, create_blake_round_input_input_limb_13),
        eval!(context, create_blake_round_input_input_limb_14),
        eval!(context, create_blake_round_input_input_limb_15),
        eval!(context, (xor_col2) + ((xor_col3) * (256))),
        eval!(context, (xor_col4) + ((xor_col5) * (256))),
        eval!(
            context,
            ((create_blake_round_input_input_limb_16) * (9812))
                + (((1) - (create_blake_round_input_input_limb_16)) * (55723))
        ),
        eval!(
            context,
            ((create_blake_round_input_input_limb_16) * (57468))
                + (((1) - (create_blake_round_input_input_limb_16)) * (8067))
        ),
    ]
}
