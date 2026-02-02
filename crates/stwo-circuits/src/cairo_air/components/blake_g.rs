// This file was created by the AIR team.

use crate::cairo_air::components::prelude::*;

pub const N_TRACE_COLUMNS: usize = 53;
pub const N_INTERACTION_COLUMNS: usize = 36;

pub const RELATION_USES_PER_ROW: [RelationUse; 6] = [
    RelationUse { relation_id: "VerifyBitwiseXor_12", uses: 2 },
    RelationUse { relation_id: "VerifyBitwiseXor_4", uses: 2 },
    RelationUse { relation_id: "VerifyBitwiseXor_7", uses: 2 },
    RelationUse { relation_id: "VerifyBitwiseXor_8", uses: 4 },
    RelationUse { relation_id: "VerifyBitwiseXor_8_B", uses: 4 },
    RelationUse { relation_id: "VerifyBitwiseXor_9", uses: 2 },
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
        input_limb_0_col0,
        input_limb_1_col1,
        input_limb_2_col2,
        input_limb_3_col3,
        input_limb_4_col4,
        input_limb_5_col5,
        input_limb_6_col6,
        input_limb_7_col7,
        input_limb_8_col8,
        input_limb_9_col9,
        input_limb_10_col10,
        input_limb_11_col11,
        triple_sum32_res_limb_0_col12,
        triple_sum32_res_limb_1_col13,
        ms_8_bits_col14,
        ms_8_bits_col15,
        ms_8_bits_col16,
        ms_8_bits_col17,
        xor_col18,
        xor_col19,
        xor_col20,
        xor_col21,
        triple_sum32_res_limb_0_col22,
        triple_sum32_res_limb_1_col23,
        ms_4_bits_col24,
        ms_4_bits_col25,
        ms_4_bits_col26,
        ms_4_bits_col27,
        xor_col28,
        xor_col29,
        xor_col30,
        xor_col31,
        triple_sum32_res_limb_0_col32,
        triple_sum32_res_limb_1_col33,
        ms_8_bits_col34,
        ms_8_bits_col35,
        ms_8_bits_col36,
        ms_8_bits_col37,
        xor_col38,
        xor_col39,
        xor_col40,
        xor_col41,
        triple_sum32_res_limb_0_col42,
        triple_sum32_res_limb_1_col43,
        ms_9_bits_col44,
        ms_9_bits_col45,
        ms_9_bits_col46,
        ms_9_bits_col47,
        xor_col48,
        xor_col49,
        xor_col50,
        xor_col51,
        enabler,
    ] = input.try_into().unwrap();
    let enabler_constraint_value = eval!(context, ((enabler) * (enabler)) - (enabler));
    acc.add_constraint(context, enabler_constraint_value);

    triple_sum_32::accumulate_constraints(
        &[
            eval!(context, input_limb_0_col0),
            eval!(context, input_limb_1_col1),
            eval!(context, input_limb_2_col2),
            eval!(context, input_limb_3_col3),
            eval!(context, input_limb_8_col8),
            eval!(context, input_limb_9_col9),
            eval!(context, triple_sum32_res_limb_0_col12),
            eval!(context, triple_sum32_res_limb_1_col13),
        ],
        context,
        component_data,
        acc,
    );

    let [xor_rot_32_r_16_output_tmp_f72c8_21_limb_0, xor_rot_32_r_16_output_tmp_f72c8_21_limb_1] =
        xor_rot_32_r_16::accumulate_constraints(
            &[
                eval!(context, triple_sum32_res_limb_0_col12),
                eval!(context, triple_sum32_res_limb_1_col13),
                eval!(context, input_limb_6_col6),
                eval!(context, input_limb_7_col7),
                eval!(context, ms_8_bits_col14),
                eval!(context, ms_8_bits_col15),
                eval!(context, ms_8_bits_col16),
                eval!(context, ms_8_bits_col17),
                eval!(context, xor_col18),
                eval!(context, xor_col19),
                eval!(context, xor_col20),
                eval!(context, xor_col21),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    triple_sum_32::accumulate_constraints(
        &[
            eval!(context, input_limb_4_col4),
            eval!(context, input_limb_5_col5),
            eval!(context, xor_rot_32_r_16_output_tmp_f72c8_21_limb_0),
            eval!(context, xor_rot_32_r_16_output_tmp_f72c8_21_limb_1),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, triple_sum32_res_limb_0_col22),
            eval!(context, triple_sum32_res_limb_1_col23),
        ],
        context,
        component_data,
        acc,
    );

    let [xor_rot_32_r_12_output_tmp_f72c8_43_limb_0, xor_rot_32_r_12_output_tmp_f72c8_43_limb_1] =
        xor_rot_32_r_12::accumulate_constraints(
            &[
                eval!(context, input_limb_2_col2),
                eval!(context, input_limb_3_col3),
                eval!(context, triple_sum32_res_limb_0_col22),
                eval!(context, triple_sum32_res_limb_1_col23),
                eval!(context, ms_4_bits_col24),
                eval!(context, ms_4_bits_col25),
                eval!(context, ms_4_bits_col26),
                eval!(context, ms_4_bits_col27),
                eval!(context, xor_col28),
                eval!(context, xor_col29),
                eval!(context, xor_col30),
                eval!(context, xor_col31),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    triple_sum_32::accumulate_constraints(
        &[
            eval!(context, triple_sum32_res_limb_0_col12),
            eval!(context, triple_sum32_res_limb_1_col13),
            eval!(context, xor_rot_32_r_12_output_tmp_f72c8_43_limb_0),
            eval!(context, xor_rot_32_r_12_output_tmp_f72c8_43_limb_1),
            eval!(context, input_limb_10_col10),
            eval!(context, input_limb_11_col11),
            eval!(context, triple_sum32_res_limb_0_col32),
            eval!(context, triple_sum32_res_limb_1_col33),
        ],
        context,
        component_data,
        acc,
    );

    let [xor_rot_32_r_8_output_tmp_f72c8_65_limb_0, xor_rot_32_r_8_output_tmp_f72c8_65_limb_1] =
        xor_rot_32_r_8::accumulate_constraints(
            &[
                eval!(context, triple_sum32_res_limb_0_col32),
                eval!(context, triple_sum32_res_limb_1_col33),
                eval!(context, xor_rot_32_r_16_output_tmp_f72c8_21_limb_0),
                eval!(context, xor_rot_32_r_16_output_tmp_f72c8_21_limb_1),
                eval!(context, ms_8_bits_col34),
                eval!(context, ms_8_bits_col35),
                eval!(context, ms_8_bits_col36),
                eval!(context, ms_8_bits_col37),
                eval!(context, xor_col38),
                eval!(context, xor_col39),
                eval!(context, xor_col40),
                eval!(context, xor_col41),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    triple_sum_32::accumulate_constraints(
        &[
            eval!(context, triple_sum32_res_limb_0_col22),
            eval!(context, triple_sum32_res_limb_1_col23),
            eval!(context, xor_rot_32_r_8_output_tmp_f72c8_65_limb_0),
            eval!(context, xor_rot_32_r_8_output_tmp_f72c8_65_limb_1),
            eval!(context, 0),
            eval!(context, 0),
            eval!(context, triple_sum32_res_limb_0_col42),
            eval!(context, triple_sum32_res_limb_1_col43),
        ],
        context,
        component_data,
        acc,
    );

    let [xor_rot_32_r_7_output_tmp_f72c8_87_limb_0, xor_rot_32_r_7_output_tmp_f72c8_87_limb_1] =
        xor_rot_32_r_7::accumulate_constraints(
            &[
                eval!(context, xor_rot_32_r_12_output_tmp_f72c8_43_limb_0),
                eval!(context, xor_rot_32_r_12_output_tmp_f72c8_43_limb_1),
                eval!(context, triple_sum32_res_limb_0_col42),
                eval!(context, triple_sum32_res_limb_1_col43),
                eval!(context, ms_9_bits_col44),
                eval!(context, ms_9_bits_col45),
                eval!(context, ms_9_bits_col46),
                eval!(context, ms_9_bits_col47),
                eval!(context, xor_col48),
                eval!(context, xor_col49),
                eval!(context, xor_col50),
                eval!(context, xor_col51),
            ],
            context,
            component_data,
            acc,
        )
        .try_into()
        .unwrap();

    // Yield BlakeG.
    let tuple_8 = &[
        eval!(context, 1139985212),
        eval!(context, input_limb_0_col0),
        eval!(context, input_limb_1_col1),
        eval!(context, input_limb_2_col2),
        eval!(context, input_limb_3_col3),
        eval!(context, input_limb_4_col4),
        eval!(context, input_limb_5_col5),
        eval!(context, input_limb_6_col6),
        eval!(context, input_limb_7_col7),
        eval!(context, input_limb_8_col8),
        eval!(context, input_limb_9_col9),
        eval!(context, input_limb_10_col10),
        eval!(context, input_limb_11_col11),
        eval!(context, triple_sum32_res_limb_0_col32),
        eval!(context, triple_sum32_res_limb_1_col33),
        eval!(context, xor_rot_32_r_7_output_tmp_f72c8_87_limb_0),
        eval!(context, xor_rot_32_r_7_output_tmp_f72c8_87_limb_1),
        eval!(context, triple_sum32_res_limb_0_col42),
        eval!(context, triple_sum32_res_limb_1_col43),
        eval!(context, xor_rot_32_r_8_output_tmp_f72c8_65_limb_0),
        eval!(context, xor_rot_32_r_8_output_tmp_f72c8_65_limb_1),
    ];
    let numerator_8 = eval!(context, -(enabler));
    acc.add_to_relation(context, numerator_8, tuple_8);
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
