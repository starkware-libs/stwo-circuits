// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 9;
pub const N_INTERACTION_COLUMNS: usize = 12;

pub const RELATION_USES_PER_ROW: [RelationUse; 4] = [
    RelationUse { relation_id: "RangeCheck_9_9", uses: 1 },
    RelationUse { relation_id: "RangeCheck_9_9_B", uses: 1 },
    RelationUse { relation_id: "RangeCheck_9_9_C", uses: 1 },
    RelationUse { relation_id: "RangeCheck_9_9_D", uses: 1 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let _ = component_data;
    let _ = acc;
    let [
        memory_id_to_small_output_col0,
        memory_id_to_small_output_col1,
        memory_id_to_small_output_col2,
        memory_id_to_small_output_col3,
        memory_id_to_small_output_col4,
        memory_id_to_small_output_col5,
        memory_id_to_small_output_col6,
        memory_id_to_small_output_col7,
        multiplicity_0,
    ] = input.try_into().unwrap();
    let seq = seq_of_component_size(context, component_data, acc);

    range_check_mem_value_n_8::accumulate_constraints(
        &[
            eval!(context, memory_id_to_small_output_col0),
            eval!(context, memory_id_to_small_output_col1),
            eval!(context, memory_id_to_small_output_col2),
            eval!(context, memory_id_to_small_output_col3),
            eval!(context, memory_id_to_small_output_col4),
            eval!(context, memory_id_to_small_output_col5),
            eval!(context, memory_id_to_small_output_col6),
            eval!(context, memory_id_to_small_output_col7),
        ],
        context,
        component_data,
        acc,
    );

    // Yield MemoryIdToBig.
    let tuple_1 = &[
        eval!(context, 1662111297),
        eval!(context, seq),
        eval!(context, memory_id_to_small_output_col0),
        eval!(context, memory_id_to_small_output_col1),
        eval!(context, memory_id_to_small_output_col2),
        eval!(context, memory_id_to_small_output_col3),
        eval!(context, memory_id_to_small_output_col4),
        eval!(context, memory_id_to_small_output_col5),
        eval!(context, memory_id_to_small_output_col6),
        eval!(context, memory_id_to_small_output_col7),
    ];
    let numerator_1 = eval!(context, -(multiplicity_0));
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
