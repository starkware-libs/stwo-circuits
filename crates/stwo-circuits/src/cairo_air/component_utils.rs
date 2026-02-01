use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::circuits::context::*;
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::eq;
use crate::eval;
use crate::stark_verifier::constraint_eval::*;

#[derive(Debug)]
pub struct RelationUse {
    pub relation_id: &'static str,
    pub uses: u64,
}

// Create a variable with the evaluation of seq_k where k is the log-height of
// the component. The height is taken from component_data.get_n_instances_bits.
pub fn seq_of_component_size<Value: IValue>(
    context: &mut Context<Value>,
    component_data: &dyn ComponentDataTrait<Value>,
    acc: &mut CompositionConstraintAccumulator,
) -> Var {
    // Compute:
    //      sum_bits = size_bits[0] + size_bits[1] + ... + size_bits[MAX_BITS]
    //      result = sum_k size_bits[k] * seq_k  (for all seq_k columns in the preprocessed trace)
    let mut sum_bits = context.zero();
    let mut result = context.zero();

    for log_size in 0..component_data.max_component_size_bits() {
        let seq_name = PreProcessedColumnId { id: format!("seq_{log_size}") };
        if !acc.preprocessed_columns.contains_key(&seq_name) {
            // Our preprocessed trace doesn't contain a seq column of this size
            continue;
        }

        let bit = component_data.get_n_instances_bit(context, log_size);
        let seq_value = acc.get_preprocessed_column(&seq_name);

        sum_bits = eval!(context, (sum_bits) + (bit));
        result = eval!(context, (result) + ((bit) * (seq_value)))
    }

    // Assert that the component size was one of the supported sizes
    eq(context, sum_bits, context.one());

    result
}
