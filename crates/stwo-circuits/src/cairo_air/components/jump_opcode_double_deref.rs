// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 21;
pub const N_INTERACTION_COLUMNS: usize = 16;

pub const RELATION_USES_PER_ROW: [RelationUse; 4] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 2 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 2 },
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
    let _ = component_data;
    let _ = acc;
    let [
        input_pc_col0,
        input_ap_col1,
        input_fp_col2,
        offset1_col3,
        offset2_col4,
        op0_base_fp_col5,
        ap_update_add_1_col6,
        mem0_base_col7,
        mem1_base_id_col8,
        mem1_base_limb_0_col9,
        mem1_base_limb_1_col10,
        mem1_base_limb_2_col11,
        mem1_base_limb_3_col12,
        partial_limb_msb_col13,
        next_pc_id_col14,
        next_pc_limb_0_col15,
        next_pc_limb_1_col16,
        next_pc_limb_2_col17,
        next_pc_limb_3_col18,
        partial_limb_msb_col19,
        enabler,
    ] = input.try_into().unwrap();
    let enabler_constraint_value = eval!(context, ((enabler) * (enabler)) - (enabler));
    acc.add_constraint(context, enabler_constraint_value);

    let [
        decode_instruction_9bd86_output_tmp_22134_6_offset1,
        decode_instruction_9bd86_output_tmp_22134_6_offset2,
    ] = decode_instruction_9bd86::accumulate_constraints(
        &[
            eval!(context, input_pc_col0),
            eval!(context, offset1_col3),
            eval!(context, offset2_col4),
            eval!(context, op0_base_fp_col5),
            eval!(context, ap_update_add_1_col6),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    //mem0_base.
    let constraint_1_value = eval!(
        context,
        (mem0_base_col7)
            - (((op0_base_fp_col5) * (input_fp_col2))
                + (((1) - (op0_base_fp_col5)) * (input_ap_col1)))
    );
    acc.add_constraint(context, constraint_1_value);

    read_positive_num_bits_29::accumulate_constraints(
        &[
            eval!(
                context,
                (mem0_base_col7) + (decode_instruction_9bd86_output_tmp_22134_6_offset1)
            ),
            eval!(context, mem1_base_id_col8),
            eval!(context, mem1_base_limb_0_col9),
            eval!(context, mem1_base_limb_1_col10),
            eval!(context, mem1_base_limb_2_col11),
            eval!(context, mem1_base_limb_3_col12),
            eval!(context, partial_limb_msb_col13),
        ],
        context,
        component_data,
        acc,
    );

    read_positive_num_bits_29::accumulate_constraints(
        &[
            eval!(
                context,
                ((((mem1_base_limb_0_col9) + ((mem1_base_limb_1_col10) * (512)))
                    + ((mem1_base_limb_2_col11) * (262144)))
                    + ((mem1_base_limb_3_col12) * (134217728)))
                    + (decode_instruction_9bd86_output_tmp_22134_6_offset2)
            ),
            eval!(context, next_pc_id_col14),
            eval!(context, next_pc_limb_0_col15),
            eval!(context, next_pc_limb_1_col16),
            eval!(context, next_pc_limb_2_col17),
            eval!(context, next_pc_limb_3_col18),
            eval!(context, partial_limb_msb_col19),
        ],
        context,
        component_data,
        acc,
    );

    // Use Opcodes.
    let tuple_4 = &[
        eval!(context, 428564188),
        eval!(context, input_pc_col0),
        eval!(context, input_ap_col1),
        eval!(context, input_fp_col2),
    ];
    let numerator_4 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_4, tuple_4);

    // Yield Opcodes.
    let tuple_5 = &[
        eval!(context, 428564188),
        eval!(
            context,
            (((next_pc_limb_0_col15) + ((next_pc_limb_1_col16) * (512)))
                + ((next_pc_limb_2_col17) * (262144)))
                + ((next_pc_limb_3_col18) * (134217728))
        ),
        eval!(context, (input_ap_col1) + (ap_update_add_1_col6)),
        eval!(context, input_fp_col2),
    ];
    let numerator_5 = eval!(context, -(enabler));
    acc.add_to_relation(context, numerator_5, tuple_5);
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
