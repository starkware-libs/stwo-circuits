// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 2;
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
    let [multiplicity_0, multiplicity_1] = input.try_into().unwrap();
    let seq_18 = acc.get_preprocessed_column(&PreProcessedColumnId { id: "seq_18".to_owned() });

    // Yield RangeCheck_18.
    let tuple_0 = &[eval!(context, 1109051422), eval!(context, seq_18)];
    let numerator_0 = eval!(context, -(multiplicity_0));
    acc.add_to_relation(context, numerator_0, tuple_0);

    // Yield RangeCheck_18_B.
    let tuple_1 = &[eval!(context, 1424798916), eval!(context, seq_18)];
    let numerator_1 = eval!(context, -(multiplicity_1));
    acc.add_to_relation(context, numerator_1, tuple_1);
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
        // Verify this component has 2 ** 18 rows
        let size_bit = component_data.get_n_instances_bit(context, 18);
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
