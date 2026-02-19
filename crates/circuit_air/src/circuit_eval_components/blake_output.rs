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
        id: "blake_output0_mults".to_owned(),
    });
    let blake_output1_addr =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_output1_addr".to_owned() });
    let blake_output1_multiplicity = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "blake_output1_mults".to_owned(),
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
