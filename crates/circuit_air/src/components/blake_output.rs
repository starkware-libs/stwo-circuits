// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 24;
pub const N_INTERACTION_COLUMNS: usize = 8;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] =
    [RelationUse { relation_id: "BlakeOutput", uses: 1 }];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [
        input_final_state_limb0_limb_0_col0,
        input_final_state_limb0_limb_1_col1,
        input_final_state_limb1_limb_0_col2,
        input_final_state_limb1_limb_1_col3,
        input_final_state_limb2_limb_0_col4,
        input_final_state_limb2_limb_1_col5,
        input_final_state_limb3_limb_0_col6,
        input_final_state_limb3_limb_1_col7,
        input_final_state_limb4_limb_0_col8,
        input_final_state_limb4_limb_1_col9,
        input_final_state_limb5_limb_0_col10,
        input_final_state_limb5_limb_1_col11,
        input_final_state_limb6_limb_0_col12,
        input_final_state_limb6_limb_1_col13,
        input_final_state_limb7_limb_0_col14,
        input_final_state_limb7_limb_1_col15,
        output_limb0_col16,
        output_limb1_col17,
        output_limb2_col18,
        output_limb3_col19,
        output_limb4_col20,
        output_limb5_col21,
        output_limb6_col22,
        output_limb7_col23,
    ] = input.try_into().unwrap();
    let blake_output0_addr =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_output0_addr".to_owned() });
    let blake_output0_multiplicity = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "blake_output0_multiplicity".to_owned(),
    });
    let blake_output1_addr =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_output1_addr".to_owned() });
    let blake_output1_multiplicity = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "blake_output1_multiplicity".to_owned(),
    });
    let final_state_addr =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "final_state_addr".to_owned() });

    // Use BlakeOutput.
    let tuple_0 = &[
        eval!(context, 1061955672),
        eval!(context, final_state_addr),
        eval!(context, input_final_state_limb0_limb_0_col0),
        eval!(context, input_final_state_limb0_limb_1_col1),
        eval!(context, input_final_state_limb1_limb_0_col2),
        eval!(context, input_final_state_limb1_limb_1_col3),
        eval!(context, input_final_state_limb2_limb_0_col4),
        eval!(context, input_final_state_limb2_limb_1_col5),
        eval!(context, input_final_state_limb3_limb_0_col6),
        eval!(context, input_final_state_limb3_limb_1_col7),
        eval!(context, input_final_state_limb4_limb_0_col8),
        eval!(context, input_final_state_limb4_limb_1_col9),
        eval!(context, input_final_state_limb5_limb_0_col10),
        eval!(context, input_final_state_limb5_limb_1_col11),
        eval!(context, input_final_state_limb6_limb_0_col12),
        eval!(context, input_final_state_limb6_limb_1_col13),
        eval!(context, input_final_state_limb7_limb_0_col14),
        eval!(context, input_final_state_limb7_limb_1_col15),
    ];
    let numerator_0 = eval!(context, 1);
    acc.add_to_relation(context, numerator_0, tuple_0);

    // Yield Gate.
    let tuple_1 = &[
        eval!(context, 378353459),
        eval!(context, blake_output0_addr),
        eval!(context, output_limb0_col16),
        eval!(context, output_limb1_col17),
        eval!(context, output_limb2_col18),
        eval!(context, output_limb3_col19),
    ];
    let numerator_1 = eval!(context, -(blake_output0_multiplicity));
    acc.add_to_relation(context, numerator_1, tuple_1);

    // Yield Gate.
    let tuple_2 = &[
        eval!(context, 378353459),
        eval!(context, blake_output1_addr),
        eval!(context, output_limb4_col20),
        eval!(context, output_limb5_col21),
        eval!(context, output_limb6_col22),
        eval!(context, output_limb7_col23),
    ];
    let numerator_2 = eval!(context, -(blake_output1_multiplicity));
    acc.add_to_relation(context, numerator_2, tuple_2);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "blake_output".to_string()
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
                PreProcessedColumnId { id: "final_state_addr".to_owned() },
                context.constant(qm31_from_u32s(2105944151, 1569844783, 443153106, 72036792)),
            ),
            (
                PreProcessedColumnId { id: "blake_output0_addr".to_owned() },
                context.constant(qm31_from_u32s(545782490, 1513343251, 1892890443, 1820346206)),
            ),
            (
                PreProcessedColumnId { id: "blake_output0_multiplicity".to_owned() },
                context.constant(qm31_from_u32s(1831188742, 1224750087, 635578300, 657734190)),
            ),
            (
                PreProcessedColumnId { id: "blake_output1_addr".to_owned() },
                context.constant(qm31_from_u32s(332029597, 1058897354, 1974660943, 1310732131)),
            ),
            (
                PreProcessedColumnId { id: "blake_output1_multiplicity".to_owned() },
                context.constant(qm31_from_u32s(2104778872, 1327615922, 1640829775, 781691366)),
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
        assert_eq!(result_value, BLAKE_OUTPUT_SAMPLE_EVAL_RESULT)
    }
}
