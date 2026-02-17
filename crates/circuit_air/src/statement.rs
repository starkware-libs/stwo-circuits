use crate::components::{eq, qm31_ops};
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::preprocessed_columns::PREPROCESSED_COLUMNS_ORDER;
use circuits::context::{Context, Var};
use circuits::ivalue::IValue;
use circuits_stark_verifier::proof::Claim;

use circuits_stark_verifier::constraint_eval::CircuitEval;
use circuits_stark_verifier::statement::Statement;

// TODO(ilya): Update this to to correct values.
pub const INTERACTION_POW_BITS: u32 = 8;

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
    fn claims_to_mix(&self, _context: &mut Context<Value>) -> Vec<Vec<Var>> {
        vec![vec![]]
    }

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

    fn get_preprocessed_column_ids(&self) -> Vec<PreProcessedColumnId> {
        PREPROCESSED_COLUMNS_ORDER
            .iter()
            .map(|id| PreProcessedColumnId { id: id.to_string() })
            .collect()
    }
}
