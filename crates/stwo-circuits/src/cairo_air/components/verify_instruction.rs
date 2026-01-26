// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 17;
pub const N_INTERACTION_COLUMNS: usize = 12;

pub const RELATION_USES_PER_ROW: [RelationUse; 4] = [
    RelationUse { relation_id: "MemoryAddressToId", uses: 1 },
    RelationUse { relation_id: "MemoryIdToBig", uses: 1 },
    RelationUse { relation_id: "RangeCheck_4_3", uses: 1 },
    RelationUse { relation_id: "RangeCheck_7_2_5", uses: 1 },
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
        input_offset0_col1,
        input_offset1_col2,
        input_offset2_col3,
        input_inst_felt5_high_col4,
        input_inst_felt6_col5,
        input_opcode_extension_col6,
        offset0_low_col7,
        offset0_mid_col8,
        offset1_low_col9,
        offset1_mid_col10,
        offset1_high_col11,
        offset2_low_col12,
        offset2_mid_col13,
        offset2_high_col14,
        instruction_id_col15,
        multiplicity_0,
    ] = input.try_into().unwrap();

    let [encode_offsets_output_tmp_16a4f_8_limb_1, encode_offsets_output_tmp_16a4f_8_limb_3] =
        encode_offsets::accumulate_constraints(
            &[
                eval!(context, input_offset0_col1),
                eval!(context, input_offset1_col2),
                eval!(context, input_offset2_col3),
                eval!(context, offset0_low_col7),
                eval!(context, offset0_mid_col8),
                eval!(context, offset1_low_col9),
                eval!(context, offset1_mid_col10),
                eval!(context, offset1_high_col11),
                eval!(context, offset2_low_col12),
                eval!(context, offset2_mid_col13),
                eval!(context, offset2_high_col14),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    mem_verify::accumulate_constraints(
        &[
            eval!(context, input_pc_col0),
            eval!(context, offset0_low_col7),
            eval!(context, encode_offsets_output_tmp_16a4f_8_limb_1),
            eval!(context, offset1_mid_col10),
            eval!(context, encode_offsets_output_tmp_16a4f_8_limb_3),
            eval!(context, offset2_mid_col13),
            eval!(context, (offset2_high_col14) + (input_inst_felt5_high_col4)),
            eval!(context, input_inst_felt6_col5),
            eval!(context, input_opcode_extension_col6),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, instruction_id_col15),
        ],
        context,
        component_data,
        acc,
    );

    // Yield VerifyInstruction.
    let tuple_2 = &[
        eval!(context, 1719106205),
        eval!(context, input_pc_col0),
        eval!(context, input_offset0_col1),
        eval!(context, input_offset1_col2),
        eval!(context, input_offset2_col3),
        eval!(context, input_inst_felt5_high_col4),
        eval!(context, input_inst_felt6_col5),
        eval!(context, input_opcode_extension_col6),
    ];
    let numerator_2 = eval!(context, -(multiplicity_0));
    acc.add_to_relation(context, numerator_2, tuple_2);
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
