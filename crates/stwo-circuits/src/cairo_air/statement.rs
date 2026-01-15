use crate::cairo_air::components;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::cairo_air::preprocessed_columns::PREPROCESSED_COLUMNS_ORDER;
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::stark_verifier::constraint_eval::CircuitEval;
use crate::stark_verifier::proof::Claim;
use crate::stark_verifier::statement::Statement;

pub struct CairoStatement<Value: IValue> {
    pub components: Vec<Box<dyn CircuitEval<Value>>>,
}
impl<Value: IValue> Default for CairoStatement<Value> {
    fn default() -> Self {
        Self {
            components: vec![
                Box::new(components::add_ap_opcode::Component {}),
                Box::new(components::assert_eq_opcode::Component {}),
                Box::new(components::jnz_opcode_non_taken::Component {}),
                Box::new(components::jnz_opcode_taken::Component {}),
                Box::new(components::jump_opcode_rel_imm::Component {}),
                Box::new(components::range_check_11::Component {}),
                Box::new(components::range_check_18::Component {}),
                Box::new(components::range_check_4_3::Component {}),
                Box::new(components::range_check_7_2_5::Component {}),
                Box::new(components::ret_opcode::Component {}),
                Box::new(components::verify_instruction::Component {}),
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
