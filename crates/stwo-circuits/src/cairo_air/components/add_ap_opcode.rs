// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 17;
pub const N_INTERACTION_COLUMNS: usize = 16;

pub const RELATION_USES_PER_ROW: [RelationUse; 6] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 1 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 1 },
    RelationUse { relation_id: "Opcodes", uses: 1 },
    RelationUse { relation_id: "RangeCheck_11", uses: 1 },
    RelationUse { relation_id: "RangeCheck_18", uses: 1 },
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
        offset2_col3,
        op1_imm_col4,
        op1_base_fp_col5,
        mem1_base_col6,
        op1_id_col7,
        msb_col8,
        mid_limbs_set_col9,
        op1_limb_0_col10,
        op1_limb_1_col11,
        op1_limb_2_col12,
        remainder_bits_col13,
        partial_limb_msb_col14,
        range_check_29_bot11bits_col15,
        enabler,
    ] = input.try_into().unwrap();
    let enabler_constraint_value = eval!(context, ((enabler) * (enabler)) - (enabler));
    acc.add_constraint(context, enabler_constraint_value);

    let [
        decode_instruction_d2a10_output_tmp_c921e_6_offset2,
        decode_instruction_d2a10_output_tmp_c921e_6_op1_base_ap,
    ] = decode_instruction_d2a10::accumulate_constraints(
        &[
            eval!(context, input_pc_col0),
            eval!(context, offset2_col3),
            eval!(context, op1_imm_col4),
            eval!(context, op1_base_fp_col5),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    //if imm then offset2 is 1.
    let constraint_1_value = eval!(
        context,
        (op1_imm_col4) * ((1) - (decode_instruction_d2a10_output_tmp_c921e_6_offset2))
    );
    acc.add_constraint(context, constraint_1_value);

    //mem1_base.
    let constraint_2_value = eval!(
        context,
        (mem1_base_col6)
            - ((((op1_imm_col4) * (input_pc_col0)) + ((op1_base_fp_col5) * (input_fp_col2)))
                + ((decode_instruction_d2a10_output_tmp_c921e_6_op1_base_ap) * (input_ap_col1)))
    );
    acc.add_constraint(context, constraint_2_value);

    let [read_small_output_tmp_c921e_16_limb_0] = read_small::accumulate_constraints(
        &[
            eval!(
                context,
                (mem1_base_col6) + (decode_instruction_d2a10_output_tmp_c921e_6_offset2)
            ),
            eval!(context, op1_id_col7),
            eval!(context, msb_col8),
            eval!(context, mid_limbs_set_col9),
            eval!(context, op1_limb_0_col10),
            eval!(context, op1_limb_1_col11),
            eval!(context, op1_limb_2_col12),
            eval!(context, remainder_bits_col13),
            eval!(context, partial_limb_msb_col14),
        ],
        context,
        component_data,
        acc,
    )
    .try_into()
    .unwrap();

    let next_ap_tmp_c921e_17 =
        eval!(context, (input_ap_col1) + (read_small_output_tmp_c921e_16_limb_0));

    range_check_29::accumulate_constraints(
        &[eval!(context, next_ap_tmp_c921e_17), eval!(context, range_check_29_bot11bits_col15)],
        context,
        component_data,
        acc,
    );

    // Use Opcodes.
    let tuple_6 = &[
        eval!(context, 428564188),
        eval!(context, input_pc_col0),
        eval!(context, input_ap_col1),
        eval!(context, input_fp_col2),
    ];
    let numerator_6 = eval!(context, enabler);
    acc.add_to_relation(context, numerator_6, tuple_6);

    // Yield Opcodes.
    let tuple_7 = &[
        eval!(context, 428564188),
        eval!(context, (input_pc_col0) + ((1) + (op1_imm_col4))),
        eval!(context, next_ap_tmp_c921e_17),
        eval!(context, input_fp_col2),
    ];
    let numerator_7 = eval!(context, -(enabler));
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
