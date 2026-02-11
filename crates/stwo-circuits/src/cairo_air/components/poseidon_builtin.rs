// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 6;
pub const N_INTERACTION_COLUMNS: usize = 16;

pub const RELATION_USES_PER_ROW: [RelationUse; 2] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 6 },
    RelationUse { relation_id: "PoseidonAggregator", uses: 1 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [
        input_state_0_id_col0,
        input_state_1_id_col1,
        input_state_2_id_col2,
        output_state_0_id_col3,
        output_state_1_id_col4,
        output_state_2_id_col5,
    ] = input.try_into().unwrap();
    let seq = seq_of_component_size(context, component_data, acc);
    let poseidon_builtin_segment_start =
        *acc.public_params.get("poseidon_builtin_segment_start").unwrap();

    let instance_addr_tmp_51986_0 =
        eval!(context, ((seq) * (6)) + (poseidon_builtin_segment_start));

    read_id::accumulate_constraints(
        &[eval!(context, instance_addr_tmp_51986_0), eval!(context, input_state_0_id_col0)],
        context,
        component_data,
        acc,
    );

    read_id::accumulate_constraints(
        &[eval!(context, (instance_addr_tmp_51986_0) + (1)), eval!(context, input_state_1_id_col1)],
        context,
        component_data,
        acc,
    );

    read_id::accumulate_constraints(
        &[eval!(context, (instance_addr_tmp_51986_0) + (2)), eval!(context, input_state_2_id_col2)],
        context,
        component_data,
        acc,
    );

    read_id::accumulate_constraints(
        &[
            eval!(context, (instance_addr_tmp_51986_0) + (3)),
            eval!(context, output_state_0_id_col3),
        ],
        context,
        component_data,
        acc,
    );

    read_id::accumulate_constraints(
        &[
            eval!(context, (instance_addr_tmp_51986_0) + (4)),
            eval!(context, output_state_1_id_col4),
        ],
        context,
        component_data,
        acc,
    );

    read_id::accumulate_constraints(
        &[
            eval!(context, (instance_addr_tmp_51986_0) + (5)),
            eval!(context, output_state_2_id_col5),
        ],
        context,
        component_data,
        acc,
    );

    // Use PoseidonAggregator.
    let tuple_7 = &[
        eval!(context, 1551892206),
        eval!(context, input_state_0_id_col0),
        eval!(context, input_state_1_id_col1),
        eval!(context, input_state_2_id_col2),
        eval!(context, output_state_0_id_col3),
        eval!(context, output_state_1_id_col4),
        eval!(context, output_state_2_id_col5),
    ];
    let numerator_7 = eval!(context, 1);
    acc.add_to_relation(context, numerator_7, tuple_7);
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
