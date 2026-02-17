// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 21;
pub const N_INTERACTION_COLUMNS: usize = 20;

pub const RELATION_USES_PER_ROW: [RelationUse; 2] = [
    RelationUse { relation_id: "VerifyBitwiseXor_8", uses: 4 },
    RelationUse { relation_id: "VerifyBitwiseXor_8_B", uses: 4 },
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
        ms_8_bits_col6,
        ms_8_bits_col7,
        ms_8_bits_col8,
        ms_8_bits_col9,
        ms_8_bits_col10,
        ms_8_bits_col11,
        xor_col12,
        xor_col13,
        xor_col14,
        xor_col15,
        xor_col16,
        xor_col17,
        xor_col18,
        xor_col19,
        enabler_col20,
    ] = input.try_into().unwrap();

    let constraint_0_value = eval!(context, ((enabler_col20) * (enabler_col20)) - (enabler_col20));
    acc.add_constraint(context, constraint_0_value);

    let [split_16_low_part_size_8_output_tmp_298db_1_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[eval!(context, input_limb_0_col0), eval!(context, ms_8_bits_col6)],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_8_output_tmp_298db_3_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[eval!(context, input_limb_1_col1), eval!(context, ms_8_bits_col7)],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_8_output_tmp_298db_5_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[eval!(context, input_limb_2_col2), eval!(context, ms_8_bits_col8)],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_8_output_tmp_298db_7_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[eval!(context, input_limb_3_col3), eval!(context, ms_8_bits_col9)],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_8_output_tmp_298db_9_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[eval!(context, input_limb_4_col4), eval!(context, ms_8_bits_col10)],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    let [split_16_low_part_size_8_output_tmp_298db_11_limb_0] =
        split_16_low_part_size_8::accumulate_constraints(
            &[eval!(context, input_limb_5_col5), eval!(context, ms_8_bits_col11)],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    bitwise_xor_num_bits_8::accumulate_constraints(
        &[
            eval!(context, split_16_low_part_size_8_output_tmp_298db_1_limb_0),
            eval!(context, split_16_low_part_size_8_output_tmp_298db_5_limb_0),
            eval!(context, xor_col12),
        ],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_8::accumulate_constraints(
        &[
            eval!(context, xor_col12),
            eval!(context, split_16_low_part_size_8_output_tmp_298db_9_limb_0),
            eval!(context, xor_col13),
        ],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_8::accumulate_constraints(
        &[
            eval!(context, ms_8_bits_col6),
            eval!(context, ms_8_bits_col8),
            eval!(context, xor_col14),
        ],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_8::accumulate_constraints(
        &[eval!(context, xor_col14), eval!(context, ms_8_bits_col10), eval!(context, xor_col15)],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_8_b::accumulate_constraints(
        &[
            eval!(context, split_16_low_part_size_8_output_tmp_298db_3_limb_0),
            eval!(context, split_16_low_part_size_8_output_tmp_298db_7_limb_0),
            eval!(context, xor_col16),
        ],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_8_b::accumulate_constraints(
        &[
            eval!(context, xor_col16),
            eval!(context, split_16_low_part_size_8_output_tmp_298db_11_limb_0),
            eval!(context, xor_col17),
        ],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_8_b::accumulate_constraints(
        &[
            eval!(context, ms_8_bits_col7),
            eval!(context, ms_8_bits_col9),
            eval!(context, xor_col18),
        ],
        context,
        component_data,
        acc,
    );

    bitwise_xor_num_bits_8_b::accumulate_constraints(
        &[eval!(context, xor_col18), eval!(context, ms_8_bits_col11), eval!(context, xor_col19)],
        context,
        component_data,
        acc,
    );

    let triple_xor32_output_tmp_298db_28_limb_0 =
        eval!(context, (xor_col13) + ((xor_col15) * (256)));

    let triple_xor32_output_tmp_298db_28_limb_1 =
        eval!(context, (xor_col17) + ((xor_col19) * (256)));

    // Yield TripleXor32.
    let tuple_17 = &[
        eval!(context, 990559919),
        eval!(context, input_limb_0_col0),
        eval!(context, input_limb_1_col1),
        eval!(context, input_limb_2_col2),
        eval!(context, input_limb_3_col3),
        eval!(context, input_limb_4_col4),
        eval!(context, input_limb_5_col5),
        eval!(context, triple_xor32_output_tmp_298db_28_limb_0),
        eval!(context, triple_xor32_output_tmp_298db_28_limb_1),
    ];
    let numerator_17 = eval!(context, -(enabler_col20));
    acc.add_to_relation(context, numerator_17, tuple_17);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
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
