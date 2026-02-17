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
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) {
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
        enabler_col46,
    ] = input.try_into().unwrap();

    let constraint_0_value = eval!(context, ((enabler_col46) * (enabler_col46)) - (enabler_col46));
    acc.add_constraint(context, constraint_0_value);

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
    let constraint_2_value = eval!(
        context,
        (mem_dst_base_col6)
            - (((dst_base_fp_col4) * (input_fp_col2))
                + (((1) - (dst_base_fp_col4)) * (input_ap_col1)))
    );
    acc.add_constraint(context, constraint_2_value);

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
    let constraint_5_value = eval!(
        context,
        (((dst_sum_p_zero_tmp_f51a9_11)
            + (((dst_limb_0_col8) + (dst_limb_21_col29)) + (dst_limb_27_col35)))
            * (dst_sum_inv_col36))
            - (1)
    );
    acc.add_constraint(context, constraint_5_value);

    let diff_from_p_tmp_f51a9_12 = eval!(context, (dst_limb_0_col8) - (1));

    let diff_from_p_tmp_f51a9_13 = eval!(context, (dst_limb_21_col29) - (136));

    let diff_from_p_tmp_f51a9_14 = eval!(context, (dst_limb_27_col35) - (256));

    //dst doesn't equal P.
    let constraint_9_value = eval!(
        context,
        (((dst_sum_p_zero_tmp_f51a9_11)
            + ((((diff_from_p_tmp_f51a9_12) * (diff_from_p_tmp_f51a9_12))
                + ((diff_from_p_tmp_f51a9_13) * (diff_from_p_tmp_f51a9_13)))
                + ((diff_from_p_tmp_f51a9_14) * (diff_from_p_tmp_f51a9_14))))
            * (dst_sum_squares_inv_col37))
            - (1)
    );
    acc.add_constraint(context, constraint_9_value);

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
    let tuple_11 = &[
        eval!(context, 428564188),
        eval!(context, input_pc_col0),
        eval!(context, input_ap_col1),
        eval!(context, input_fp_col2),
    ];
    let numerator_11 = eval!(context, enabler_col46);
    acc.add_to_relation(context, numerator_11, tuple_11);

    // Yield Opcodes.
    let tuple_12 = &[
        eval!(context, 428564188),
        eval!(context, (input_pc_col0) + (read_small_output_tmp_f51a9_24_limb_0)),
        eval!(context, (input_ap_col1) + (ap_update_add_1_col5)),
        eval!(context, input_fp_col2),
    ];
    let numerator_12 = eval!(context, -(enabler_col46));
    acc.add_to_relation(context, numerator_12, tuple_12);
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
            qm31_from_u32s(902525010, 1115155995, 130434373, 2116865290),
        ];
        let interaction_columns = [
            qm31_from_u32s(1005168032, 79980996, 1847888101, 1941984119),
            qm31_from_u32s(1072277211, 214198724, 1914996965, 1941984119),
            qm31_from_u32s(1139386390, 348416452, 1982105829, 1941984119),
            qm31_from_u32s(1206495569, 482634180, 2049214693, 1941984119),
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
        assert_eq!(result_value, JNZ_OPCODE_TAKEN_SAMPLE_EVAL_RESULT)
    }
}
