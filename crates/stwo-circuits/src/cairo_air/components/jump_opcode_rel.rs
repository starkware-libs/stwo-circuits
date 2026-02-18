// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 16;
pub const N_INTERACTION_COLUMNS: usize = 12;

pub const RELATION_USES_PER_ROW: [RelationUse; 4] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 1 },
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
        offset2_col3,
        op1_base_fp_col4,
        ap_update_add_1_col5,
        mem1_base_col6,
        next_pc_id_col7,
        msb_col8,
        mid_limbs_set_col9,
        next_pc_limb_0_col10,
        next_pc_limb_1_col11,
        next_pc_limb_2_col12,
        remainder_bits_col13,
        partial_limb_msb_col14,
        enabler_col15,
    ] = input.try_into().unwrap();

    let [
        decode_instruction_ba944_output_tmp_62dfc_5_offset2,
        decode_instruction_ba944_output_tmp_62dfc_5_op1_base_ap,
    ] = decode_instruction_ba944::accumulate_constraints(
        &[
            eval!(context, input_pc_col0),
            eval!(context, offset2_col3),
            eval!(context, op1_base_fp_col4),
            eval!(context, ap_update_add_1_col5),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    //mem1_base.
    let constraint_1_value = eval!(
        context,
        (mem1_base_col6)
            - (((op1_base_fp_col4) * (input_fp_col2))
                + ((decode_instruction_ba944_output_tmp_62dfc_5_op1_base_ap) * (input_ap_col1)))
    );
    acc.add_constraint(context, constraint_1_value);

    let [read_small_output_tmp_62dfc_15_limb_0] = read_small::accumulate_constraints(
        &[
            eval!(
                context,
                (mem1_base_col6) + (decode_instruction_ba944_output_tmp_62dfc_5_offset2)
            ),
            eval!(context, next_pc_id_col7),
            eval!(context, msb_col8),
            eval!(context, mid_limbs_set_col9),
            eval!(context, next_pc_limb_0_col10),
            eval!(context, next_pc_limb_1_col11),
            eval!(context, next_pc_limb_2_col12),
            eval!(context, remainder_bits_col13),
            eval!(context, partial_limb_msb_col14),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    //Enabler is a bit.
    let constraint_3_value = eval!(context, ((enabler_col15) * (enabler_col15)) - (enabler_col15));
    acc.add_constraint(context, constraint_3_value);

    // Use Opcodes.
    let tuple_4 = &[
        eval!(context, 428564188),
        eval!(context, input_pc_col0),
        eval!(context, input_ap_col1),
        eval!(context, input_fp_col2),
    ];
    let numerator_4 = eval!(context, enabler_col15);
    acc.add_to_relation(context, numerator_4, tuple_4);

    // Yield Opcodes.
    let tuple_5 = &[
        eval!(context, 428564188),
        eval!(context, (input_pc_col0) + (read_small_output_tmp_62dfc_15_limb_0)),
        eval!(context, (input_ap_col1) + (ap_update_add_1_col5)),
        eval!(context, input_fp_col2),
    ];
    let numerator_5 = eval!(context, -(enabler_col15));
    acc.add_to_relation(context, numerator_5, tuple_5);
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
        ];
        let interaction_columns = [
            qm31_from_u32s(1005168032, 79980996, 1847888101, 1941984119),
            qm31_from_u32s(1072277211, 214198724, 1914996965, 1941984119),
            qm31_from_u32s(1139386390, 348416452, 1982105829, 1941984119),
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
        assert_eq!(result_value, JUMP_OPCODE_REL_SAMPLE_EVAL_RESULT)
    }
}
