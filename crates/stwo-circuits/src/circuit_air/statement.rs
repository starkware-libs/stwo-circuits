use crate::circuit_air::components::{eq, qm31_ops};
use crate::circuit_air::relations::GATE_RELATION_ID;
use crate::circuits::simd::Simd;
use crate::circuits::wrappers::M31Wrapper;
use crate::eval;
use itertools::Itertools;
use stwo::core::fields::m31::M31;
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

/// Represents an output of the circuit with its variable address and QM31 value (see
/// [crate::circuits::circuit::Output]).
pub struct Output {
    /// The variable index (address) of the output.
    pub address: M31Wrapper<Var>,
    /// The 4 M31 components of the QM31 value at this address.
    pub value: [M31Wrapper<Var>; 4],
}

pub struct CircuitStatement<Value: IValue> {
    pub components: Vec<Box<dyn CircuitEval<Value>>>,
    pub outputs: Vec<Output>,
}
impl<Value: IValue> CircuitStatement<Value> {
    pub fn new(context: &mut Context<Value>, outputs: &[(M31, [M31; 4])]) -> Self {
        let outputs = outputs
            .iter()
            .map(|&(addr, [v0, v1, v2, v3])| Output {
                address: M31Wrapper::new_unsafe(context.new_var(value)),
                value: [
                    M31Wrapper::new_unsafe(context.new_var(v0.into())),
                    M31Wrapper::new_unsafe(context.new_var(v1.into())),
                    M31Wrapper::new_unsafe(context.new_var(v2.into())),
                    M31Wrapper::new_unsafe(context.new_var(v3.into())),
                ],
            })
            .collect();

        Self {
            components: vec![
                Box::new(eq::CircuitEqComponent { preprocessed_column_indices: [0, 1] }),
                Box::new(qm31_ops::CircuitQm31OpsComponent {
                    preprocessed_column_indices: [2, 3, 4, 5, 6, 7, 8, 9],
                }),
            ],
            outputs,
        }
    }
}
impl<Value: IValue> Statement<Value> for CircuitStatement<Value> {
    fn claims_to_mix(&self, context: &mut Context<Value>) -> Vec<Vec<Var>> {
        let packed_outputs = Simd::pack(
            context,
            &self
                .outputs
                .iter()
                .flat_map(|output| {
                    [
                        &output.address,
                        &output.value[0],
                        &output.value[1],
                        &output.value[2],
                        &output.value[3],
                    ]
                })
                .cloned()
                .collect_vec(),
        );
        vec![packed_outputs.get_packed().to_vec()]
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
        for output in &self.outputs {
            let term = logup_use_term(
                context,
                &[
                    gate_relation_id,
                    *output.address.get(),
                    *output.value[0].get(),
                    *output.value[1].get(),
                    *output.value[2].get(),
                    *output.value[3].get(),
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
