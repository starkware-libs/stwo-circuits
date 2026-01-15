use crate::circuit_air::components::{eq, qm31_ops};
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;

use crate::stark_verifier::constraint_eval::CircuitEval;
use crate::stark_verifier::proof::Claim;
use crate::stark_verifier::statement::Statement;

pub struct CircuitStatement<Value: IValue> {
    pub components: Vec<Box<dyn CircuitEval<Value>>>,
}
impl<Value: IValue> Default for CircuitStatement<Value> {
    fn default() -> Self {
        Self {
            components: vec![
                Box::new(eq::CircuitEqComponent { preprocessed_column_indices: [0, 1] }),
                Box::new(qm31_ops::CircuitQm31OpsComponent {
                    preprocessed_column_indices: [2, 3, 4, 5, 6, 7, 8, 9],
                }),
            ],
        }
    }
}
impl<Value: IValue> Statement<Value> for CircuitStatement<Value> {
    fn get_components(&self) -> &[Box<dyn CircuitEval<Value>>] {
        &self.components
    }

    fn public_logup_sum(
        &self,
        context: &mut Context<Value>,
        _interaction_elements: [Var; 2],
        _claim: &Claim<Var>,
    ) -> Var {
        context.zero()
    }
}
