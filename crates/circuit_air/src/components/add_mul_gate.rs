// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 12;
pub const N_INTERACTION_COLUMNS: usize = 8;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] = [RelationUse { relation_id: "Gate", uses: 2 }];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [
        input_op0_limb0_col0,
        input_op0_limb1_col1,
        input_op0_limb2_col2,
        input_op0_limb3_col3,
        input_op1_limb0_col4,
        input_op1_limb1_col5,
        input_op1_limb2_col6,
        input_op1_limb3_col7,
        input_dst_limb0_col8,
        input_dst_limb1_col9,
        input_dst_limb2_col10,
        input_dst_limb3_col11,
    ] = input.try_into().unwrap();
    let add_flag = acc.get_preprocessed_column(&PreProcessedColumnId { id: "add_flag".to_owned() });
    let dst_addr = acc.get_preprocessed_column(&PreProcessedColumnId { id: "dst_addr".to_owned() });
    let mul_flag = acc.get_preprocessed_column(&PreProcessedColumnId { id: "mul_flag".to_owned() });
    let op0_addr = acc.get_preprocessed_column(&PreProcessedColumnId { id: "op0_addr".to_owned() });
    let op1_addr = acc.get_preprocessed_column(&PreProcessedColumnId { id: "op1_addr".to_owned() });
    let pointwise_mul_flag =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pointwise_mul_flag".to_owned() });
    let qm31_ops_multiplicity = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "qm31_ops_multiplicity".to_owned() });
    let sub_flag = acc.get_preprocessed_column(&PreProcessedColumnId { id: "sub_flag".to_owned() });

    //all flags sum to 1.
    let constraint_0_value =
        eval!(context, ((((add_flag) + (sub_flag)) + (mul_flag)) + (pointwise_mul_flag)) - (1));
    acc.add_constraint(context, constraint_0_value);

    //add_flag is a bit.
    let constraint_1_value = eval!(context, (add_flag) * ((add_flag) - (1)));
    acc.add_constraint(context, constraint_1_value);

    //sub_flag is a bit.
    let constraint_2_value = eval!(context, (sub_flag) * ((sub_flag) - (1)));
    acc.add_constraint(context, constraint_2_value);

    //mul_flag is a bit.
    let constraint_3_value = eval!(context, (mul_flag) * ((mul_flag) - (1)));
    acc.add_constraint(context, constraint_3_value);

    //pointwise_mul_flag is a bit.
    let constraint_4_value = eval!(context, (pointwise_mul_flag) * ((pointwise_mul_flag) - (1)));
    acc.add_constraint(context, constraint_4_value);

    let constraint_5_value = eval!(
        context,
        (input_dst_limb0_col8)
            - ((((((((((input_op0_limb0_col0) * (input_op1_limb0_col4))
                - ((input_op0_limb1_col1) * (input_op1_limb1_col5)))
                + ((2)
                    * (((input_op0_limb2_col2) * (input_op1_limb2_col6))
                        - ((input_op0_limb3_col3) * (input_op1_limb3_col7)))))
                - ((input_op0_limb2_col2) * (input_op1_limb3_col7)))
                - ((input_op0_limb3_col3) * (input_op1_limb2_col6)))
                * (mul_flag))
                + (((input_op0_limb0_col0) + (input_op1_limb0_col4)) * (add_flag)))
                + (((input_op0_limb0_col0) - (input_op1_limb0_col4)) * (sub_flag)))
                + (((input_op0_limb0_col0) * (input_op1_limb0_col4)) * (pointwise_mul_flag)))
    );
    acc.add_constraint(context, constraint_5_value);

    let constraint_6_value = eval!(
        context,
        (input_dst_limb1_col9)
            - ((((((((((input_op0_limb0_col0) * (input_op1_limb1_col5))
                + ((input_op0_limb1_col1) * (input_op1_limb0_col4)))
                + ((2)
                    * (((input_op0_limb2_col2) * (input_op1_limb3_col7))
                        + ((input_op0_limb3_col3) * (input_op1_limb2_col6)))))
                + ((input_op0_limb2_col2) * (input_op1_limb2_col6)))
                - ((input_op0_limb3_col3) * (input_op1_limb3_col7)))
                * (mul_flag))
                + (((input_op0_limb1_col1) + (input_op1_limb1_col5)) * (add_flag)))
                + (((input_op0_limb1_col1) - (input_op1_limb1_col5)) * (sub_flag)))
                + (((input_op0_limb1_col1) * (input_op1_limb1_col5)) * (pointwise_mul_flag)))
    );
    acc.add_constraint(context, constraint_6_value);

    let constraint_7_value = eval!(
        context,
        (input_dst_limb2_col10)
            - (((((((((input_op0_limb0_col0) * (input_op1_limb2_col6))
                - ((input_op0_limb1_col1) * (input_op1_limb3_col7)))
                + ((input_op0_limb2_col2) * (input_op1_limb0_col4)))
                - ((input_op0_limb3_col3) * (input_op1_limb1_col5)))
                * (mul_flag))
                + (((input_op0_limb2_col2) + (input_op1_limb2_col6)) * (add_flag)))
                + (((input_op0_limb2_col2) - (input_op1_limb2_col6)) * (sub_flag)))
                + (((input_op0_limb2_col2) * (input_op1_limb2_col6)) * (pointwise_mul_flag)))
    );
    acc.add_constraint(context, constraint_7_value);

    let constraint_8_value = eval!(
        context,
        (input_dst_limb3_col11)
            - (((((((((input_op0_limb0_col0) * (input_op1_limb3_col7))
                + ((input_op0_limb1_col1) * (input_op1_limb2_col6)))
                + ((input_op0_limb2_col2) * (input_op1_limb1_col5)))
                + ((input_op0_limb3_col3) * (input_op1_limb0_col4)))
                * (mul_flag))
                + (((input_op0_limb3_col3) + (input_op1_limb3_col7)) * (add_flag)))
                + (((input_op0_limb3_col3) - (input_op1_limb3_col7)) * (sub_flag)))
                + (((input_op0_limb3_col3) * (input_op1_limb3_col7)) * (pointwise_mul_flag)))
    );
    acc.add_constraint(context, constraint_8_value);

    // Use Gate.
    let tuple_9 = &[
        eval!(context, 378353459),
        eval!(context, op0_addr),
        eval!(context, input_op0_limb0_col0),
        eval!(context, input_op0_limb1_col1),
        eval!(context, input_op0_limb2_col2),
        eval!(context, input_op0_limb3_col3),
    ];
    let numerator_9 = eval!(context, 1);
    acc.add_to_relation(context, numerator_9, tuple_9);

    // Use Gate.
    let tuple_10 = &[
        eval!(context, 378353459),
        eval!(context, op1_addr),
        eval!(context, input_op1_limb0_col4),
        eval!(context, input_op1_limb1_col5),
        eval!(context, input_op1_limb2_col6),
        eval!(context, input_op1_limb3_col7),
    ];
    let numerator_10 = eval!(context, 1);
    acc.add_to_relation(context, numerator_10, tuple_10);

    // Yield Gate.
    let tuple_11 = &[
        eval!(context, 378353459),
        eval!(context, dst_addr),
        eval!(context, input_dst_limb0_col8),
        eval!(context, input_dst_limb1_col9),
        eval!(context, input_dst_limb2_col10),
        eval!(context, input_dst_limb3_col11),
    ];
    let numerator_11 = eval!(context, -(qm31_ops_multiplicity));
    acc.add_to_relation(context, numerator_11, tuple_11);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "add_mul_gate".to_string()
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
            qm31_from_u32s(700269555, 307766862, 1685683780, 745982081),
        ];
        let interaction_columns = [
            qm31_from_u32s(1005168032, 79980996, 1847888101, 1941984119),
            qm31_from_u32s(1072277211, 214198724, 1914996965, 1941984119),
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
                PreProcessedColumnId { id: "add_flag".to_owned() },
                context.constant(qm31_from_u32s(287801007, 1639185429, 739213960, 1937617367)),
            ),
            (
                PreProcessedColumnId { id: "mul_flag".to_owned() },
                context.constant(qm31_from_u32s(1385251064, 979665308, 1813087576, 707332196)),
            ),
            (
                PreProcessedColumnId { id: "pointwise_mul_flag".to_owned() },
                context.constant(qm31_from_u32s(370130236, 248815775, 2064198532, 946492407)),
            ),
            (
                PreProcessedColumnId { id: "sub_flag".to_owned() },
                context.constant(qm31_from_u32s(959120445, 1991939391, 771350898, 270607940)),
            ),
            (
                PreProcessedColumnId { id: "op0_addr".to_owned() },
                context.constant(qm31_from_u32s(391264576, 808421283, 2141129359, 1577246560)),
            ),
            (
                PreProcessedColumnId { id: "op1_addr".to_owned() },
                context.constant(qm31_from_u32s(1847617402, 1292397090, 2087950641, 1380563370)),
            ),
            (
                PreProcessedColumnId { id: "dst_addr".to_owned() },
                context.constant(qm31_from_u32s(727072817, 1271914452, 2073396652, 1845474033)),
            ),
            (
                PreProcessedColumnId { id: "qm31_ops_multiplicity".to_owned() },
                context.constant(qm31_from_u32s(271587272, 1324294110, 104931299, 958140170)),
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
        assert_eq!(result_value, ADD_MUL_GATE_SAMPLE_EVAL_RESULT)
    }
}
