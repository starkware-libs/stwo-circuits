// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 9;
pub const N_INTERACTION_COLUMNS: usize = 12;

pub const RELATION_USES_PER_ROW: [RelationUse; 3] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 2 },
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
        offset0_col3,
        dst_base_fp_col4,
        ap_update_add_1_col5,
        mem_dst_base_col6,
        dst_id_col7,
        enabler,
    ] = input.try_into().unwrap();
    let enabler_constraint_value = eval!(context, ((enabler) * (enabler)) - (enabler));
    acc.add_constraint(context, enabler_constraint_value);

    let [decode_instruction_161c9_output_tmp_bb09e_5_offset0] =
        decode_instruction_161c9::accumulate_constraints(
            &[
                eval!(context, input_pc_col0),
                eval!(context, offset0_col3),
                eval!(context, dst_base_fp_col4),
                eval!(context, ap_update_add_1_col5),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    //mem_dst_base.
    let constraint_1_value = eval!(
        context,
        (mem_dst_base_col6)
            - (((dst_base_fp_col4) * (input_fp_col2))
                + (((1) - (dst_base_fp_col4)) * (input_ap_col1)))
    );
    acc.add_constraint(context, constraint_1_value);

    mem_verify_equal::accumulate_constraints(
        &[
            eval!(
                context,
                (mem_dst_base_col6) + (decode_instruction_161c9_output_tmp_bb09e_5_offset0)
            ),
            eval!(context, (input_pc_col0) + (1)),
            eval!(context, dst_id_col7),
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
        eval!(context, (input_pc_col0) + (2)),
        eval!(context, (input_ap_col1) + (ap_update_add_1_col5)),
        eval!(context, input_fp_col2),
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
