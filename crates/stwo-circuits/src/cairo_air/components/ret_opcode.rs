// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 16;
pub const N_INTERACTION_COLUMNS: usize = 16;

pub const RELATION_USES_PER_ROW: [RelationUse; 4] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 2 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 2 },
    RelationUse { relation_id: "Opcodes", uses: 1 },
    RelationUse { relation_id: "VerifyInstruction", uses: 1 },
];

pub fn accumulate_constraints(
    input: &[Var],
    context: &mut Context<impl IValue>,
    component_data: &ComponentData<'_>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let _ = component_data;
    let [
        input_pc_col0,
        input_ap_col1,
        input_fp_col2,
        next_pc_id_col3,
        next_pc_limb_0_col4,
        next_pc_limb_1_col5,
        next_pc_limb_2_col6,
        next_pc_limb_3_col7,
        partial_limb_msb_col8,
        next_fp_id_col9,
        next_fp_limb_0_col10,
        next_fp_limb_1_col11,
        next_fp_limb_2_col12,
        next_fp_limb_3_col13,
        partial_limb_msb_col14,
        enabler,
    ] = input.try_into().unwrap();
    let enabler_constraint_value = eval!(context, ((enabler) * (enabler)) - (enabler));
    acc.add_constraint(context, enabler_constraint_value);

    decode_instruction_15a61::accumulate_constraints(
        &[eval!(context, input_pc_col0)],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_29::accumulate_constraints(
        &[
            eval!(context, (input_fp_col2) - (1)),
            eval!(context, next_pc_id_col3),
            eval!(context, next_pc_limb_0_col4),
            eval!(context, next_pc_limb_1_col5),
            eval!(context, next_pc_limb_2_col6),
            eval!(context, next_pc_limb_3_col7),
            eval!(context, partial_limb_msb_col8),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_29::accumulate_constraints(
        &[
            eval!(context, (input_fp_col2) - (2)),
            eval!(context, next_fp_id_col9),
            eval!(context, next_fp_limb_0_col10),
            eval!(context, next_fp_limb_1_col11),
            eval!(context, next_fp_limb_2_col12),
            eval!(context, next_fp_limb_3_col13),
            eval!(context, partial_limb_msb_col14),
        ],
        context,
        component_data,
        acc,
    );

    // Use Opcodes.
    let tuple_3 = &[
        eval!(context, 428564188),
        eval!(context, input_pc_col0),
        eval!(context, input_ap_col1),
        eval!(context, input_fp_col2),
    ];
    let numerator_3 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_3, tuple_3);

    // Yield Opcodes.
    let tuple_4 = &[
        eval!(context, 428564188),
        eval!(
            context,
            (((next_pc_limb_0_col4) + ((next_pc_limb_1_col5) * (512)))
                + ((next_pc_limb_2_col6) * (262144)))
                + ((next_pc_limb_3_col7) * (134217728))
        ),
        eval!(context, input_ap_col1),
        eval!(
            context,
            (((next_fp_limb_0_col10) + ((next_fp_limb_1_col11) * (512)))
                + ((next_fp_limb_2_col12) * (262144)))
                + ((next_fp_limb_3_col13) * (134217728))
        ),
    ];
    let numerator_4 = eval!(context, -(enabler));
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
