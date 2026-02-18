// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 19;
pub const N_INTERACTION_COLUMNS: usize = 16;

pub const RELATION_USES_PER_ROW: [RelationUse; 4] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 3 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 1 },
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
        offset1_col4,
        offset2_col5,
        dst_base_fp_col6,
        op0_base_fp_col7,
        ap_update_add_1_col8,
        mem_dst_base_col9,
        mem0_base_col10,
        mem1_base_id_col11,
        mem1_base_limb_0_col12,
        mem1_base_limb_1_col13,
        mem1_base_limb_2_col14,
        mem1_base_limb_3_col15,
        partial_limb_msb_col16,
        dst_id_col17,
        enabler_col18,
    ] = input.try_into().unwrap();

    let constraint_0_value = eval!(context, ((enabler_col18) * (enabler_col18)) - (enabler_col18));
    acc.add_constraint(context, constraint_0_value);

    let [
        decode_instruction_cb32b_output_tmp_b1151_8_offset0,
        decode_instruction_cb32b_output_tmp_b1151_8_offset1,
        decode_instruction_cb32b_output_tmp_b1151_8_offset2,
    ] = decode_instruction_cb32b::accumulate_constraints(
        &[
            eval!(context, input_pc_col0),
            eval!(context, offset0_col3),
            eval!(context, offset1_col4),
            eval!(context, offset2_col5),
            eval!(context, dst_base_fp_col6),
            eval!(context, op0_base_fp_col7),
            eval!(context, ap_update_add_1_col8),
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
        (mem_dst_base_col9)
            - (((dst_base_fp_col6) * (input_fp_col2))
                + (((1) - (dst_base_fp_col6)) * (input_ap_col1)))
    );
    acc.add_constraint(context, constraint_2_value);

    //mem0_base.
    let constraint_3_value = eval!(
        context,
        (mem0_base_col10)
            - (((op0_base_fp_col7) * (input_fp_col2))
                + (((1) - (op0_base_fp_col7)) * (input_ap_col1)))
    );
    acc.add_constraint(context, constraint_3_value);

    read_positive_num_bits_29::accumulate_constraints(
        &[
            eval!(
                context,
                (mem0_base_col10) + (decode_instruction_cb32b_output_tmp_b1151_8_offset1)
            ),
            eval!(context, mem1_base_id_col11),
            eval!(context, mem1_base_limb_0_col12),
            eval!(context, mem1_base_limb_1_col13),
            eval!(context, mem1_base_limb_2_col14),
            eval!(context, mem1_base_limb_3_col15),
            eval!(context, partial_limb_msb_col16),
        ],
        context,
        component_data,
        acc,
    );

    mem_verify_equal::accumulate_constraints(
        &[
            eval!(
                context,
                (mem_dst_base_col9) + (decode_instruction_cb32b_output_tmp_b1151_8_offset0)
            ),
            eval!(
                context,
                ((((mem1_base_limb_0_col12) + ((mem1_base_limb_1_col13) * (512)))
                    + ((mem1_base_limb_2_col14) * (262144)))
                    + ((mem1_base_limb_3_col15) * (134217728)))
                    + (decode_instruction_cb32b_output_tmp_b1151_8_offset2)
            ),
            eval!(context, dst_id_col17),
        ],
        context,
        component_data,
        acc,
    );

    // Use Opcodes.
    let tuple_6 = &[
        eval!(context, 428564188),
        eval!(context, input_pc_col0),
        eval!(context, input_ap_col1),
        eval!(context, input_fp_col2),
    ];
    let numerator_6 = eval!(context, enabler_col18);
    acc.add_to_relation(context, numerator_6, tuple_6);

    // Yield Opcodes.
    let tuple_7 = &[
        eval!(context, 428564188),
        eval!(context, (input_pc_col0) + (1)),
        eval!(context, (input_ap_col1) + (ap_update_add_1_col8)),
        eval!(context, input_fp_col2),
    ];
    let numerator_7 = eval!(context, -(enabler_col18));
    acc.add_to_relation(context, numerator_7, tuple_7);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "assert_eq_opcode_double_deref".to_string()
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
        assert_eq!(result_value, ASSERT_EQ_OPCODE_DOUBLE_DEREF_SAMPLE_EVAL_RESULT)
    }
}
