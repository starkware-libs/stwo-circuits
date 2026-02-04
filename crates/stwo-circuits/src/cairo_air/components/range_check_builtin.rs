// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 17;
pub const N_INTERACTION_COLUMNS: usize = 4;

pub const RELATION_USES_PER_ROW: [RelationUse; 2] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 1 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 1 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [
        value_id_col0,
        value_limb_0_col1,
        value_limb_1_col2,
        value_limb_2_col3,
        value_limb_3_col4,
        value_limb_4_col5,
        value_limb_5_col6,
        value_limb_6_col7,
        value_limb_7_col8,
        value_limb_8_col9,
        value_limb_9_col10,
        value_limb_10_col11,
        value_limb_11_col12,
        value_limb_12_col13,
        value_limb_13_col14,
        value_limb_14_col15,
        partial_limb_msb_col16,
    ] = input.try_into().unwrap();
    let seq = seq_of_component_size(context, component_data, acc);
    let range_check_builtin_segment_start =
        *acc.public_params.get("range_check_builtin_segment_start").unwrap();

    read_positive_num_bits_128::accumulate_constraints(
        &[
            eval!(context, (range_check_builtin_segment_start) + (seq)),
            eval!(context, value_id_col0),
            eval!(context, value_limb_0_col1),
            eval!(context, value_limb_1_col2),
            eval!(context, value_limb_2_col3),
            eval!(context, value_limb_3_col4),
            eval!(context, value_limb_4_col5),
            eval!(context, value_limb_5_col6),
            eval!(context, value_limb_6_col7),
            eval!(context, value_limb_7_col8),
            eval!(context, value_limb_8_col9),
            eval!(context, value_limb_9_col10),
            eval!(context, value_limb_10_col11),
            eval!(context, value_limb_11_col12),
            eval!(context, value_limb_12_col13),
            eval!(context, value_limb_13_col14),
            eval!(context, value_limb_14_col15),
            eval!(context, partial_limb_msb_col16),
        ],
        context,
        component_data,
        acc,
    );
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
