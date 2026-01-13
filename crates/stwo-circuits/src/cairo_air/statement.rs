use crate::cairo_air::components;
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::stark_verifier::constraint_eval::CircuitEval;
use crate::stark_verifier::statement::Statement;

pub struct CairoStatement<Value: IValue> {
    pub components: Vec<Box<dyn CircuitEval<Value>>>,
}
impl<Value: IValue> Default for CairoStatement<Value> {
    fn default() -> Self {
        Self {
            components: vec![
                Box::new(components::jnz_opcode_taken::Component {}),
                Box::new(components::jnz_opcode_non_taken::Component {}),
            ],
        }
    }
}


impl<Value: IValue> Statement<Value> for CairoStatement<Value> {
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
