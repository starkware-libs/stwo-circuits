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
    let range_check_4_4_4_4_column_0 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "range_check_4_4_4_4_column_0".to_owned(),
    });
    let range_check_4_4_4_4_column_1 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "range_check_4_4_4_4_column_1".to_owned(),
    });
    let range_check_4_4_4_4_column_2 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "range_check_4_4_4_4_column_2".to_owned(),
    });
    let range_check_4_4_4_4_column_3 = acc.get_preprocessed_column(&PreProcessedColumnId {
        id: "range_check_4_4_4_4_column_3".to_owned(),
    });

    // Yield RangeCheck_4_4_4_4.
    let tuple_0 = &[
        eval!(context, 1027333874),
        eval!(context, range_check_4_4_4_4_column_0),
        eval!(context, range_check_4_4_4_4_column_1),
        eval!(context, range_check_4_4_4_4_column_2),
        eval!(context, range_check_4_4_4_4_column_3),
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
        // Verify this component has 2 ** 16 rows
        let size_bit = component_data.get_n_instances_bit(context, 16);
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
