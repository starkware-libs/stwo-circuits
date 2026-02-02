// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 1;
pub const N_INTERACTION_COLUMNS: usize = 4;

pub const RELATION_USES_PER_ROW: [RelationUse; 0] = [];

#[allow(unused_variables)]
pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let _ = component_data;
    let _ = acc;
    let [multiplicity_0] = input.try_into().unwrap();
    let seq_4 = acc.get_preprocessed_column(&PreProcessedColumnId { id: "seq_4".to_owned() });
    let blake_sigma_0 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_0".to_owned() });
    let blake_sigma_1 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_1".to_owned() });
    let blake_sigma_2 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_2".to_owned() });
    let blake_sigma_3 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_3".to_owned() });
    let blake_sigma_4 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_4".to_owned() });
    let blake_sigma_5 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_5".to_owned() });
    let blake_sigma_6 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_6".to_owned() });
    let blake_sigma_7 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_7".to_owned() });
    let blake_sigma_8 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_8".to_owned() });
    let blake_sigma_9 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_9".to_owned() });
    let blake_sigma_10 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_10".to_owned() });
    let blake_sigma_11 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_11".to_owned() });
    let blake_sigma_12 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_12".to_owned() });
    let blake_sigma_13 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_13".to_owned() });
    let blake_sigma_14 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_14".to_owned() });
    let blake_sigma_15 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "blake_sigma_15".to_owned() });

    // Yield BlakeRoundSigma.
    let tuple_0 = &[
        eval!(context, 1805967942),
        eval!(context, seq_4),
        eval!(context, blake_sigma_0),
        eval!(context, blake_sigma_1),
        eval!(context, blake_sigma_2),
        eval!(context, blake_sigma_3),
        eval!(context, blake_sigma_4),
        eval!(context, blake_sigma_5),
        eval!(context, blake_sigma_6),
        eval!(context, blake_sigma_7),
        eval!(context, blake_sigma_8),
        eval!(context, blake_sigma_9),
        eval!(context, blake_sigma_10),
        eval!(context, blake_sigma_11),
        eval!(context, blake_sigma_12),
        eval!(context, blake_sigma_13),
        eval!(context, blake_sigma_14),
        eval!(context, blake_sigma_15),
    ];
    let numerator_0 = eval!(context, -(multiplicity_0));
    acc.add_to_relation(context, numerator_0, tuple_0);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &ComponentData<'_>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        accumulate_constraints(component_data.trace_columns, context, component_data, acc);
        // Verify this component has 2 ** 4 rows
        let size_bit = component_data.get_n_instances_bit(context, 4);
        eq(context, size_bit, context.one());
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
