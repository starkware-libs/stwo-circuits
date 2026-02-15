use crate::circuit_air::components::{eq, qm31_ops};
use crate::circuit_air::relations::GATE_RELATION_ID;
use crate::circuits::ops::Guess;
use crate::circuits::simd::Simd;
use crate::circuits::wrappers::M31Wrapper;
use crate::eval;
use itertools::{Itertools, zip_eq};
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::circuit_air::preprocessed_columns::PREPROCESSED_COLUMNS_ORDER;
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::stark_verifier::logup::logup_use_term;
use crate::stark_verifier::proof::Claim;

use crate::stark_verifier::constraint_eval::CircuitEval;
use crate::stark_verifier::statement::Statement;

// TODO(ilya): Update this to to correct values.
pub const INTERACTION_POW_BITS: u32 = 8;

pub struct CircuitStatement<Value: IValue> {
    pub components: Vec<Box<dyn CircuitEval<Value>>>,
    /// The variable indices (addresses) of the output gates.
    pub output_addresses: Vec<M31Wrapper<Var>>,
    /// The values of the output gates.
    pub output_values: Vec<Var>,
}
impl<Value: IValue> CircuitStatement<Value> {
    pub fn new(
        context: &mut Context<Value>,
        output_addresses: &[M31],
        output_values: &[QM31],
    ) -> Self {
        let output_addresses = output_addresses
            .iter()
            .map(|&addr| M31Wrapper::new_unsafe(context.constant(addr.into())))
            .collect_vec();
        let output_values =
            output_values.iter().map(|value| Value::from_qm31(*value).guess(context)).collect_vec();
        Self {
            components: vec![
                Box::new(eq::CircuitEqComponent { preprocessed_column_indices: [0, 1] }),
                Box::new(qm31_ops::CircuitQm31OpsComponent {
                    preprocessed_column_indices: [2, 3, 4, 5, 6, 7, 8, 9],
                }),
            ],
            output_addresses,
            output_values,
        }
    }
}
impl<Value: IValue> Statement<Value> for CircuitStatement<Value> {
    fn claims_to_mix(&self, _context: &mut Context<Value>) -> Vec<Vec<Var>> {
        vec![self.output_values.clone()]
    }

    fn get_components(&self) -> &[Box<dyn CircuitEval<Value>>] {
        &self.components
    }

    fn public_logup_sum(
        &self,
        context: &mut Context<Value>,
        interaction_elements: [Var; 2],
        _claim: &Claim<Var>,
    ) -> Var {
        let mut sum = context.zero();

        let gate_relation_id = context.constant(GATE_RELATION_ID.into());
        for (output_address, output_value) in
            zip_eq(self.output_addresses.iter(), self.output_values.iter())
        {
            let [output_value_0, output_value_1, output_value_2, output_value_3] =
                Simd::unpack(context, &Simd::from_packed(vec![*output_value], 4))
                    .try_into()
                    .unwrap();
            let term = logup_use_term(
                context,
                &[
                    gate_relation_id,
                    *output_address.get(),
                    output_value_0,
                    output_value_1,
                    output_value_2,
                    output_value_3,
                ],
                interaction_elements,
            );
            sum = eval!(context, (sum) + (term));
        }
        sum
    }

    fn get_preprocessed_column_ids(&self) -> Vec<PreProcessedColumnId> {
        PREPROCESSED_COLUMNS_ORDER
            .iter()
            .map(|id| PreProcessedColumnId { id: id.to_string() })
            .collect()
    }
}
