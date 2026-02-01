// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

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
        enabler,
    ] = input.try_into().unwrap();
    let enabler_constraint_value = eval!(context, ((enabler) * (enabler)) - (enabler));
    acc.add_constraint(context, enabler_constraint_value);

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
    let tuple_16 = &[
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
    let numerator_16 = eval!(context, -(enabler));
    acc.add_to_relation(context, numerator_16, tuple_16);
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
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use stwo::core::fields::qm31::QM31;

    #[allow(unused_imports)]
    use crate::cairo_air::components::prelude::PreProcessedColumnId;
    use crate::cairo_air::sample_evaluations::*;
    use crate::cairo_air::test::TestComponentData;
    use crate::circuits::context::Context;
    use crate::circuits::ivalue::qm31_from_u32s;
    #[allow(unused_imports)]
    use crate::eval;
    #[allow(unused_imports)]
    use crate::stark_verifier::circle::denom_inverse;
    use crate::stark_verifier::constraint_eval::*;

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
            qm31_from_u32s(902525010, 1115155995, 130434373, 2116865290),
        ];
        let interaction_columns = [
            qm31_from_u32s(1005168032, 79980996, 1847888101, 1941984119),
            qm31_from_u32s(1072277211, 214198724, 1914996965, 1941984119),
            qm31_from_u32s(1139386390, 348416452, 1982105829, 1941984119),
            qm31_from_u32s(1206495569, 482634180, 2049214693, 1941984119),
            qm31_from_u32s(736731316, 1690593731, 1579452644, 1941984119),
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
        let preprocessed_columns = HashMap::from([]);
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
        assert_eq!(result_value, TRIPLE_XOR_32_SAMPLE_EVAL_RESULT)
    }
}
