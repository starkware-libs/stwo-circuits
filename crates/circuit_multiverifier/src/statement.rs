use circuit_verifier::{
    blake2s_consts::blake2s_initial_state,
    relations::{BLAKE_STATE_RELATION_ID, GATE_RELATION_ID},
};
use circuits::{
    blake::HashValue,
    context::{Context, Var},
    eval,
    ivalue::IValue,
    ops::div,
    simd::Simd,
    wrappers::M31Wrapper,
};
use circuits_stark_verifier::{
    constraint_eval::CircuitEval,
    logup::{combine_term, logup_use_term},
    statement::Statement,
};
use indexmap::IndexMap;
use itertools::zip_eq;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

pub struct SubCircuitStatement<Value: IValue> {
    pub components: IndexMap<&'static str, Box<dyn CircuitEval<Value>>>,
    /// The variable indices (addresses) of the output gates.
    pub output_addresses: Vec<M31Wrapper<Var>>,
    /// The values of the output gates.
    pub output_values: Vec<Var>,
    /// The log number of blake gates in the circuit.
    pub n_blake_gates_pow_two: M31Wrapper<Var>,
    /// Preprocessed column ids in the exact order used by the prover's preprocessed trace.
    pub preprocessed_column_ids: Vec<PreProcessedColumnId>,
    /// The preprocessed trace root.
    pub preprocessed_root: HashValue<Var>,
}
// impl<Value: IValue> SubCircuitStatement<Value> {
//     pub fn new(
//         context: &mut Context<Value>,
//         output_addresses: &[usize],
//         output_values: &[QM31],
//         n_blake_gates: usize,
//         preprocessed_column_ids: Vec<PreProcessedColumnId>,
//         preprocessed_root: HashValue<QM31>,
//     ) -> Self {
//         let output_addresses = output_addresses
//             .iter()
//             .map(|&addr| M31Wrapper::new_unsafe(context.constant(addr.into())))
//             .collect_vec();
//         let output_values =
//             output_values.iter().map(|value|
// Value::from_qm31(*value).guess(context)).collect_vec();         Self {
//             components: all_circuit_components(),
//             output_addresses,
//             output_values,
//             n_blake_gates,
//             preprocessed_column_ids,
//             preprocessed_root,
//         }
//     }
// }

impl<Value: IValue> Statement<Value> for SubCircuitStatement<Value> {
    fn claims_to_mix(&self, _context: &mut Context<Value>) -> Vec<Vec<Var>> {
        vec![self.output_values.clone()]
    }

    fn get_components(&self) -> &IndexMap<&'static str, Box<dyn CircuitEval<Value>>> {
        &self.components
    }

    fn public_logup_sum(
        &self,
        context: &mut Context<Value>,
        interaction_elements: [Var; 2],
    ) -> Var {
        let mut sum = context.zero();

        // Output gates public logup sum contribution.
        let gate_relation_id = context.constant(GATE_RELATION_ID.into());
        for (output_address, output_value) in zip_eq(&self.output_addresses, &self.output_values) {
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

        // Blake IV public logup sum contribution.
        let initial_state = blake2s_initial_state();
        let blake_state_relation_id = context.constant(BLAKE_STATE_RELATION_ID.into());
        let iv_state_address = context.zero();
        let mut logup_terms = vec![blake_state_relation_id, iv_state_address];
        for &word in &initial_state {
            let low = context.constant((word & 0xffff).into());
            let high = context.constant((word >> 16).into());
            logup_terms.push(low);
            logup_terms.push(high);
        }
        let blake_iv_denom = combine_term(context, &logup_terms, interaction_elements);

        // There are `self.n_blake_gates.next_power_of_two()` BlakeOutput rows, each one uses
        // the same IV state, either indirectly through a blakeGate or directly in padding rows
        // of the BlakeOutput component.
        let n_blakes = *self.n_blake_gates_pow_two.get();
        let blake_iv_yield = div(context, n_blakes, blake_iv_denom);
        sum = eval!(context, (sum) - (blake_iv_yield));

        sum
    }

    fn get_preprocessed_column_ids(&self) -> Vec<PreProcessedColumnId> {
        self.preprocessed_column_ids.clone()
    }

    fn get_preprocessed_root(&self, _context: &mut Context<Value>) -> HashValue<Var> {
        self.preprocessed_root
    }
}
