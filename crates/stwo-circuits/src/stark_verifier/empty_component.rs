use crate::{
    cairo_air::component_utils::RelationUse,
    circuits::{context::Context, ivalue::IValue},
    stark_verifier::constraint_eval::{
        CircuitEval, ComponentDataTrait, CompositionConstraintAccumulator,
    },
};

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

    fn evaluate(
        &self,
        _context: &mut Context<Value>,
        _component_data: &dyn ComponentDataTrait<Value>,
        _acc: &mut CompositionConstraintAccumulator,
    ) {
    }
}
