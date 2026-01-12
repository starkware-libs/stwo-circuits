use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::stark_verifier::constraint_eval::CircuitEval;
use crate::stark_verifier::statement::Statement;

pub struct CircuitStatement<Value: IValue> {
    pub components: Vec<Box<dyn CircuitEval<Value>>>,
}
impl<Value: IValue> Statement<Value> for CircuitStatement<Value> {
    fn get_components(&self) -> &[Box<dyn CircuitEval<Value>>] {
        &self.components
    }

    fn public_logup_sum(
        &self,
        context: &mut Context<Value>,
        _interaction_elements: [Var; 2],
    ) -> Var {
        context.zero()
    }
}
