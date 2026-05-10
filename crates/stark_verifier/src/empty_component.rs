use crate::constraint_eval::{
    CircuitEval, ComponentDataTrait, CompositionConstraintAccumulator, RelationUse,
};
use crate::order_hash_map::OrderedHashMap;
use circuits::{context::Context, ivalue::IValue};
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

// An empty component that can be used when a component is disabled.
pub struct EmptyComponent {}
impl<Value: IValue> CircuitEval<Value> for EmptyComponent {
    fn name(&self) -> String {
        "empty_component".to_string()
    }

    fn trace_columns(&self) -> usize {
        0
    }

    fn interaction_columns(&self) -> usize {
        0
    }

    fn relation_uses_per_row(&self) -> &[RelationUse] {
        &[]
    }

    fn log_size(
        &self,
        _preprocessed_column_log_sizes: OrderedHashMap<PreProcessedColumnId, u32>,
    ) -> Option<u32> {
        None
    }

    fn evaluate(
        &self,
        _context: &mut Context<Value>,
        _component_data: &dyn ComponentDataTrait<Value>,
        _acc: &mut CompositionConstraintAccumulator,
    ) {
    }
}
