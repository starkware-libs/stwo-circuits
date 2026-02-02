// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 3;
pub const N_INTERACTION_COLUMNS: usize = 8;

pub const RELATION_USES_PER_ROW: [RelationUse; 2] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 3 },
    RelationUse { relation_id: "PedersenAggregatorWindowBits9", uses: 1 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [input_state_0_id_col0, input_state_1_id_col1, output_state_id_col2] =
        input.try_into().unwrap();
    let seq = seq_of_component_size(context, component_data, acc);
    let pedersen_builtin_segment_start =
        *acc.public_params.get("pedersen_builtin_segment_start").unwrap();

    let instance_addr_tmp_adb38_0 =
        eval!(context, ((seq) * (3)) + (pedersen_builtin_segment_start));

    read_id::accumulate_constraints(
        &[eval!(context, instance_addr_tmp_adb38_0), eval!(context, input_state_0_id_col0)],
        context,
        component_data,
        acc,
    );

    read_id::accumulate_constraints(
        &[eval!(context, (instance_addr_tmp_adb38_0) + (1)), eval!(context, input_state_1_id_col1)],
        context,
        component_data,
        acc,
    );

    read_id::accumulate_constraints(
        &[eval!(context, (instance_addr_tmp_adb38_0) + (2)), eval!(context, output_state_id_col2)],
        context,
        component_data,
        acc,
    );

    // Use PedersenAggregatorWindowBits9.
    let tuple_4 = &[
        eval!(context, 194336987),
        eval!(context, input_state_0_id_col0),
        eval!(context, input_state_1_id_col1),
        eval!(context, output_state_id_col2),
    ];
    let numerator_4 = eval!(context, 1);
    acc.add_to_relation(context, numerator_4, tuple_4);
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
