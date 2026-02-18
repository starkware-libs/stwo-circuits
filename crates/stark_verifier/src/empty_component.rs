use crate::constraint_eval::{
    CircuitEval, ComponentDataTrait, CompositionConstraintAccumulator, RelationUse,
};
use circuits::{context::Context, ivalue::IValue};

// An empty component that can be used when a component is disabled.
pub struct EmptyComponent {}
impl<Value: IValue> CircuitEval<Value> for EmptyComponent {
    fn trace_columns(&self) -> usize {
        0
    }

    fn interaction_columns(&self) -> usize {
        0
    }

    fn relation_uses_per_row(&self) -> &[RelationUse] {
        &[]
    }

    fn evaluate(
        &self,
        _context: &mut Context<Value>,
        _component_data: &dyn ComponentDataTrait<Value>,
        _acc: &mut CompositionConstraintAccumulator,
    ) {
    }
}
