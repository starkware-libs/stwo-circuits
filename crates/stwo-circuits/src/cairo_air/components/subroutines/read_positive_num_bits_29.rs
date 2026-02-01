// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

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
) -> Vec<Var> {
    let _ = component_data;
    let _ = acc;
    let [
        read_positive_num_bits_29_input,
        id_col0,
        value_limb_0_col1,
        value_limb_1_col2,
        value_limb_2_col3,
        value_limb_3_col4,
        partial_limb_msb_col5,
    ] = input.try_into().unwrap();

    read_id::accumulate_constraints(
        &[eval!(context, read_positive_num_bits_29_input), eval!(context, id_col0)],
        context,
        component_data,
        acc,
    );

    read_positive_known_id_num_bits_29::accumulate_constraints(
        &[
            eval!(context, id_col0),
            eval!(context, value_limb_0_col1),
            eval!(context, value_limb_1_col2),
            eval!(context, value_limb_2_col3),
            eval!(context, value_limb_3_col4),
            eval!(context, partial_limb_msb_col5),
        ],
        context,
        component_data,
        acc,
    );
    vec![]
}
