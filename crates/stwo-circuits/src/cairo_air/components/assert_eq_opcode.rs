// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 12;
pub const N_INTERACTION_COLUMNS: usize = 12;

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
        offset0_col3,
        offset2_col4,
        dst_base_fp_col5,
        op1_base_fp_col6,
        ap_update_add_1_col7,
        mem_dst_base_col8,
        mem1_base_col9,
        dst_id_col10,
        enabler,
    ] = input.try_into().unwrap();
    let enabler_constraint_value = eval!(context, ((enabler) * (enabler)) - (enabler));
    acc.add_constraint(context, enabler_constraint_value);

    let [
        decode_instruction_fe864_output_tmp_d6f03_7_offset0,
        decode_instruction_fe864_output_tmp_d6f03_7_offset2,
        decode_instruction_fe864_output_tmp_d6f03_7_op1_base_ap,
    ] = decode_instruction_fe864::accumulate_constraints(
        &[
            eval!(context, input_pc_col0),
            eval!(context, offset0_col3),
            eval!(context, offset2_col4),
            eval!(context, dst_base_fp_col5),
            eval!(context, op1_base_fp_col6),
            eval!(context, ap_update_add_1_col7),
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
        (mem_dst_base_col8)
            - (((dst_base_fp_col5) * (input_fp_col2))
                + (((1) - (dst_base_fp_col5)) * (input_ap_col1)))
    );
    acc.add_constraint(context, constraint_1_value);

    //mem1_base.
    let constraint_2_value = eval!(
        context,
        (mem1_base_col9)
            - (((op1_base_fp_col6) * (input_fp_col2))
                + ((decode_instruction_fe864_output_tmp_d6f03_7_op1_base_ap) * (input_ap_col1)))
    );
    acc.add_constraint(context, constraint_2_value);

    mem_verify_equal::accumulate_constraints(
        &[
            eval!(
                context,
                (mem_dst_base_col8) + (decode_instruction_fe864_output_tmp_d6f03_7_offset0)
            ),
            eval!(
                context,
                (mem1_base_col9) + (decode_instruction_fe864_output_tmp_d6f03_7_offset2)
            ),
            eval!(context, dst_id_col10),
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
        eval!(context, (input_pc_col0) + (1)),
        eval!(context, (input_ap_col1) + (ap_update_add_1_col7)),
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
}
