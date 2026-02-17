// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 12;
pub const N_INTERACTION_COLUMNS: usize = 8;

pub const RELATION_USES_PER_ROW: [RelationUse; 1] = [RelationUse { relation_id: "Gate", uses: 2 }];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [
        input_op0_limb0_col0,
        input_op0_limb1_col1,
        input_op0_limb2_col2,
        input_op0_limb3_col3,
        input_op1_limb0_col4,
        input_op1_limb1_col5,
        input_op1_limb2_col6,
        input_op1_limb3_col7,
        input_dst_limb0_col8,
        input_dst_limb1_col9,
        input_dst_limb2_col10,
        input_dst_limb3_col11,
    ] = input.try_into().unwrap();
    let add_flag = acc.get_preprocessed_column(&PreProcessedColumnId { id: "add_flag".to_owned() });
    let dst_addr = acc.get_preprocessed_column(&PreProcessedColumnId { id: "dst_addr".to_owned() });
    let mul_flag = acc.get_preprocessed_column(&PreProcessedColumnId { id: "mul_flag".to_owned() });
    let op0_addr = acc.get_preprocessed_column(&PreProcessedColumnId { id: "op0_addr".to_owned() });
    let op1_addr = acc.get_preprocessed_column(&PreProcessedColumnId { id: "op1_addr".to_owned() });
    let pointwise_mul_flag =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "pointwise_mul_flag".to_owned() });
    let qm31_ops_multiplicity = acc
        .get_preprocessed_column(&PreProcessedColumnId { id: "qm31_ops_multiplicity".to_owned() });
    let sub_flag = acc.get_preprocessed_column(&PreProcessedColumnId { id: "sub_flag".to_owned() });

    //all flags sum to 1.
    let constraint_0_value =
        eval!(context, ((((add_flag) + (sub_flag)) + (mul_flag)) + (pointwise_mul_flag)) - (1));
    acc.add_constraint(context, constraint_0_value);

    //add_flag is a bit.
    let constraint_1_value = eval!(context, (add_flag) * ((add_flag) - (1)));
    acc.add_constraint(context, constraint_1_value);

    //sub_flag is a bit.
    let constraint_2_value = eval!(context, (sub_flag) * ((sub_flag) - (1)));
    acc.add_constraint(context, constraint_2_value);

    //mul_flag is a bit.
    let constraint_3_value = eval!(context, (mul_flag) * ((mul_flag) - (1)));
    acc.add_constraint(context, constraint_3_value);

    //pointwise_mul_flag is a bit.
    let constraint_4_value = eval!(context, (pointwise_mul_flag) * ((pointwise_mul_flag) - (1)));
    acc.add_constraint(context, constraint_4_value);

    let constraint_5_value = eval!(
        context,
        (input_dst_limb0_col8)
            - ((((((((((input_op0_limb0_col0) * (input_op1_limb0_col4))
                - ((input_op0_limb1_col1) * (input_op1_limb1_col5)))
                + ((2)
                    * (((input_op0_limb2_col2) * (input_op1_limb2_col6))
                        - ((input_op0_limb3_col3) * (input_op1_limb3_col7)))))
                - ((input_op0_limb2_col2) * (input_op1_limb3_col7)))
                - ((input_op0_limb3_col3) * (input_op1_limb2_col6)))
                * (mul_flag))
                + (((input_op0_limb0_col0) + (input_op1_limb0_col4)) * (add_flag)))
                + (((input_op1_limb0_col4) + (input_op0_limb0_col0)) * (sub_flag)))
                + (((input_op0_limb0_col0) * (input_op1_limb0_col4)) * (pointwise_mul_flag)))
    );
    acc.add_constraint(context, constraint_5_value);

    let constraint_6_value = eval!(
        context,
        (input_dst_limb1_col9)
            - ((((((((((input_op0_limb0_col0) * (input_op1_limb1_col5))
                + ((input_op0_limb1_col1) * (input_op1_limb0_col4)))
                + ((2)
                    * (((input_op0_limb2_col2) * (input_op1_limb3_col7))
                        + ((input_op0_limb3_col3) * (input_op1_limb2_col6)))))
                + ((input_op0_limb2_col2) * (input_op1_limb2_col6)))
                - ((input_op0_limb3_col3) * (input_op1_limb3_col7)))
                * (mul_flag))
                + (((input_op0_limb1_col1) + (input_op1_limb1_col5)) * (add_flag)))
                + (((input_op1_limb1_col5) + (input_op0_limb1_col1)) * (sub_flag)))
                + (((input_op0_limb1_col1) * (input_op1_limb1_col5)) * (pointwise_mul_flag)))
    );
    acc.add_constraint(context, constraint_6_value);

    let constraint_7_value = eval!(
        context,
        (input_dst_limb2_col10)
            - (((((((((input_op0_limb0_col0) * (input_op1_limb2_col6))
                - ((input_op0_limb1_col1) * (input_op1_limb3_col7)))
                + ((input_op0_limb2_col2) * (input_op1_limb0_col4)))
                - ((input_op0_limb3_col3) * (input_op1_limb1_col5)))
                * (mul_flag))
                + (((input_op0_limb2_col2) + (input_op1_limb2_col6)) * (add_flag)))
                + (((input_op1_limb2_col6) + (input_op0_limb2_col2)) * (sub_flag)))
                + (((input_op0_limb2_col2) * (input_op1_limb2_col6)) * (pointwise_mul_flag)))
    );
    acc.add_constraint(context, constraint_7_value);

    let constraint_8_value = eval!(
        context,
        (input_dst_limb3_col11)
            - (((((((((input_op0_limb0_col0) * (input_op1_limb3_col7))
                + ((input_op0_limb1_col1) * (input_op1_limb2_col6)))
                + ((input_op0_limb2_col2) * (input_op1_limb1_col5)))
                + ((input_op0_limb3_col3) * (input_op1_limb0_col4)))
                * (mul_flag))
                + (((input_op0_limb3_col3) + (input_op1_limb3_col7)) * (add_flag)))
                + (((input_op1_limb3_col7) + (input_op0_limb3_col3)) * (sub_flag)))
                + (((input_op0_limb3_col3) * (input_op1_limb3_col7)) * (pointwise_mul_flag)))
    );
    acc.add_constraint(context, constraint_8_value);

    // Use Gate.
    let tuple_9 = &[
        eval!(context, 378353459),
        eval!(context, op0_addr),
        eval!(context, input_op0_limb0_col0),
        eval!(context, input_op0_limb1_col1),
        eval!(context, input_op0_limb2_col2),
        eval!(context, input_op0_limb3_col3),
    ];
    let numerator_9 = eval!(context, 1);
    acc.add_to_relation(context, numerator_9, tuple_9);

    // Use Gate.
    let tuple_10 = &[
        eval!(context, 378353459),
        eval!(context, op1_addr),
        eval!(context, input_op1_limb0_col4),
        eval!(context, input_op1_limb1_col5),
        eval!(context, input_op1_limb2_col6),
        eval!(context, input_op1_limb3_col7),
    ];
    let numerator_10 = eval!(context, 1);
    acc.add_to_relation(context, numerator_10, tuple_10);

    // Yield Gate.
    let tuple_11 = &[
        eval!(context, 378353459),
        eval!(context, dst_addr),
        eval!(context, input_dst_limb0_col8),
        eval!(context, input_dst_limb1_col9),
        eval!(context, input_dst_limb2_col10),
        eval!(context, input_dst_limb3_col11),
    ];
    let numerator_11 = eval!(context, -(qm31_ops_multiplicity));
    acc.add_to_relation(context, numerator_11, tuple_11);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &dyn ComponentDataTrait<Value>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        accumulate_constraints(component_data.trace_columns(), context, component_data, acc);
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
