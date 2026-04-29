use crate::components::{
    blake_g_gate, eq::CircuitEqComponent, m_31_to_u_32, qm31_ops::CircuitQm31OpsComponent,
    range_check_16, triple_xor, verify_bitwise_xor_4, verify_bitwise_xor_7, verify_bitwise_xor_8,
    verify_bitwise_xor_9, verify_bitwise_xor_12,
};
use crate::relations::GATE_RELATION_ID;
use crate::verify::CircuitConfig;
use circuits::blake::HashValue;
use circuits::context::{Context, Var};
use circuits::eval;
use circuits::ivalue::IValue;
use circuits::ops::Guess;
use circuits::simd::Simd;
use circuits::wrappers::M31Wrapper;
use circuits_stark_verifier::constraint_eval::CircuitEval;
use circuits_stark_verifier::logup::logup_use_term;
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use circuits_stark_verifier::proof_from_stark_proof::pack_into_qm31s;
use circuits_stark_verifier::statement::Statement;
use indexmap::IndexMap;
use itertools::{Itertools, zip_eq};
use stwo::core::fields::qm31::QM31;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

// TODO(ilya): Update this to the correct values.
pub const INTERACTION_POW_BITS: u32 = 20;

pub struct CircuitStatement<Value: IValue> {
    pub components: IndexMap<&'static str, Box<dyn CircuitEval<Value>>>,
    /// The variable indices (addresses) of the output gates.
    pub output_addresses: Vec<M31Wrapper<Var>>,
    /// The values of the output gates.
    pub output_values: Vec<Var>,
    /// Per-component trace log sizes packed as a [`Simd`].
    pub component_log_sizes: Simd,
    /// Maps preprocessed column ids to their log sizes.
    /// The order of the keys is the same as the order of the columns in the prover's preprocessed
    /// trace.
    pub preprocessed_column_log_sizes: OrderedHashMap<PreProcessedColumnId, u32>,
    /// The preprocessed trace root.
    pub preprocessed_root: HashValue<QM31>,
}
impl<Value: IValue> CircuitStatement<Value> {
    pub fn new(
        context: &mut Context<Value>,
        circuit_config: &CircuitConfig,
        output_values: &[QM31],
    ) -> Self {
        let CircuitConfig {
            config: _,
            output_addresses,
            preprocessed_column_log_sizes,
            preprocessed_root,
        } = circuit_config;

        let output_addresses = output_addresses
            .iter()
            .map(|&addr| M31Wrapper::new_unsafe(context.constant(addr.into())))
            .collect_vec();
        let output_values =
            output_values.iter().map(|value| Value::from_qm31(*value).guess(context)).collect_vec();

        let components = all_circuit_components::<Value>();
        let component_log_sizes = components
            .values()
            .map(|c| {
                c.log_size(preprocessed_column_log_sizes)
                    .expect("The circuit components can't have a dynamic log_size.")
            })
            .collect_vec();

        let n_components = component_log_sizes.len();
        let packed_log_sizes = pack_into_qm31s(component_log_sizes.iter().cloned())
            .into_iter()
            .map(|qm31| Value::from_qm31(qm31).guess(context))
            .collect_vec();
        let component_log_sizes = Simd::from_packed(packed_log_sizes, n_components);

        Self {
            components,
            output_addresses,
            output_values,
            component_log_sizes,
            preprocessed_column_log_sizes: preprocessed_column_log_sizes.clone(),
            preprocessed_root: *preprocessed_root,
        }
    }
}

impl<Value: IValue> Statement<Value> for CircuitStatement<Value> {
    fn claims_to_mix(&self, _context: &mut Context<Value>) -> Vec<Vec<Var>> {
        vec![self.output_values.clone()]
    }

    fn get_components(&self) -> &IndexMap<&'static str, Box<dyn CircuitEval<Value>>> {
        &self.components
    }

    fn get_component_log_sizes(&self) -> &Simd {
        &self.component_log_sizes
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

        sum
    }

    fn get_preprocessed_column_ids(&self) -> Vec<PreProcessedColumnId> {
        self.preprocessed_column_log_sizes.keys().cloned().collect()
    }

    fn get_preprocessed_root(&self, context: &mut Context<Value>) -> HashValue<Var> {
        HashValue(
            context.constant(self.preprocessed_root.0),
            context.constant(self.preprocessed_root.1),
        )
    }
}

pub fn all_circuit_components<Value: IValue>() -> IndexMap<&'static str, Box<dyn CircuitEval<Value>>>
{
    IndexMap::from([
        ("eq", Box::new(CircuitEqComponent {}) as Box<dyn CircuitEval<Value>>),
        ("qm31_ops", Box::new(CircuitQm31OpsComponent {}) as Box<dyn CircuitEval<Value>>),
        ("triple_xor", Box::new(triple_xor::Component {}) as Box<dyn CircuitEval<Value>>),
        ("m_31_to_u_32", Box::new(m_31_to_u_32::Component {}) as Box<dyn CircuitEval<Value>>),
        ("blake_g_gate", Box::new(blake_g_gate::Component {}) as Box<dyn CircuitEval<Value>>),
        (
            "verify_bitwise_xor_8",
            Box::new(verify_bitwise_xor_8::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "verify_bitwise_xor_12",
            Box::new(verify_bitwise_xor_12::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "verify_bitwise_xor_4",
            Box::new(verify_bitwise_xor_4::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "verify_bitwise_xor_7",
            Box::new(verify_bitwise_xor_7::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        (
            "verify_bitwise_xor_9",
            Box::new(verify_bitwise_xor_9::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        ("range_check_16", Box::new(range_check_16::Component {}) as Box<dyn CircuitEval<Value>>),
    ])
}
