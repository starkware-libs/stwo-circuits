// This file was created by the AIR team.

use super::prelude::*;

pub const N_TRACE_COLUMNS: usize = 1;
pub const N_INTERACTION_COLUMNS: usize = 4;

pub const RELATION_USES_PER_ROW: [RelationUse; 0] = [];

#[allow(unused_variables)]
pub fn accumulate_constraints<Value: IValue>(
    input: &[Var],
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) {
    let [multiplicity_0_col0] = input.try_into().unwrap();
    let bitwise_xor_7_0 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "bitwise_xor_7_0".to_owned() });
    let bitwise_xor_7_1 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "bitwise_xor_7_1".to_owned() });
    let bitwise_xor_7_2 =
        acc.get_preprocessed_column(&PreProcessedColumnId { id: "bitwise_xor_7_2".to_owned() });

    // Yield VerifyBitwiseXor_7.
    let tuple_0 = &[
        eval!(context, 62225763),
        eval!(context, bitwise_xor_7_0),
        eval!(context, bitwise_xor_7_1),
        eval!(context, bitwise_xor_7_2),
    ];
    let numerator_0 = eval!(context, -(multiplicity_0_col0));
    acc.add_to_relation(context, numerator_0, tuple_0);
}

pub struct Component {}
impl<Value: IValue> CircuitEval<Value> for Component {
    fn name(&self) -> String {
        "verify_bitwise_xor_7".to_string()
    }

    fn evaluate(
        &self,
        context: &mut Context<Value>,
        component_data: &dyn ComponentDataTrait<Value>,
        acc: &mut CompositionConstraintAccumulator,
    ) {
        accumulate_constraints(component_data.trace_columns(), context, component_data, acc);
        // Verify this component has 2 ** 14 rows
        let size_bit = component_data.get_n_instances_bit(context, 14);
        eq(context, size_bit, context.one());
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
