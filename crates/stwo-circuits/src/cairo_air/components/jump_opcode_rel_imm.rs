// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 13;
pub const N_INTERACTION_COLUMNS: usize = 12;

pub const RELATION_USES_PER_ROW: [RelationUse; 4] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 1 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 1 },
    RelationUse { relation_id: "Opcodes", uses: 1 },
    RelationUse { relation_id: "VerifyInstruction", uses: 1 },
];

#[allow(unused_variables)]
pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [
        input_pc_col0,
        input_ap_col1,
        input_fp_col2,
        ap_update_add_1_col3,
        next_pc_id_col4,
        msb_col5,
        mid_limbs_set_col6,
        next_pc_limb_0_col7,
        next_pc_limb_1_col8,
        next_pc_limb_2_col9,
        remainder_bits_col10,
        partial_limb_msb_col11,
        enabler,
    ] = input.try_into().unwrap();
    let enabler_constraint_value = eval!(context, ((enabler) * (enabler)) - (enabler));
    acc.add_constraint(context, enabler_constraint_value);

    decode_instruction_7ebc4::accumulate_constraints(
        &[eval!(context, input_pc_col0), eval!(context, ap_update_add_1_col3)],
        context,
        component_data,
        acc,
    );

    let [read_small_output_tmp_81a39_13_limb_0] = read_small::accumulate_constraints(
        &[
            eval!(context, (input_pc_col0) + (1)),
            eval!(context, next_pc_id_col4),
            eval!(context, msb_col5),
            eval!(context, mid_limbs_set_col6),
            eval!(context, next_pc_limb_0_col7),
            eval!(context, next_pc_limb_1_col8),
            eval!(context, next_pc_limb_2_col9),
            eval!(context, remainder_bits_col10),
            eval!(context, partial_limb_msb_col11),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    // Use Opcodes.
    let tuple_2 = &[
        eval!(context, 428564188),
        eval!(context, input_pc_col0),
        eval!(context, input_ap_col1),
        eval!(context, input_fp_col2),
    ];
    let numerator_2 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_2, tuple_2);

    // Yield Opcodes.
    let tuple_3 = &[
        eval!(context, 428564188),
        eval!(context, (input_pc_col0) + (read_small_output_tmp_81a39_13_limb_0)),
        eval!(context, (input_ap_col1) + (ap_update_add_1_col3)),
        eval!(context, input_fp_col2),
    ];
    let numerator_3 = eval!(context, -(enabler));
    acc.add_to_relation(context, numerator_3, tuple_3);
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
