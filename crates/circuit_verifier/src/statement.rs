use crate::blake2s_consts::blake2s_initial_state;
use crate::components::{
    blake_g, blake_gate, blake_output, blake_round, blake_round_sigma, eq::CircuitEqComponent,
    m_31_to_u_32, qm31_ops::CircuitQm31OpsComponent, range_check_15, range_check_16, triple_xor_32,
    verify_bitwise_xor_4, verify_bitwise_xor_7, verify_bitwise_xor_8, verify_bitwise_xor_9,
    verify_bitwise_xor_12,
};

mod component_log_size {
    pub const BLAKE_ROUND_SIGMA: u32 = 4;
    pub const VERIFY_BITWISE_XOR_8: u32 = 16;
    pub const VERIFY_BITWISE_XOR_12: u32 = super::verify_bitwise_xor_12::LOG_SIZE;
    pub const VERIFY_BITWISE_XOR_4: u32 = 8;
    pub const VERIFY_BITWISE_XOR_7: u32 = 14;
    pub const VERIFY_BITWISE_XOR_9: u32 = 18;
    pub const RANGE_CHECK_15: u32 = 15;
    pub const RANGE_CHECK_16: u32 = 16;
}
use crate::relations::{BLAKE_STATE_RELATION_ID, GATE_RELATION_ID};
use crate::verify::CircuitConfig;
use circuits::blake::HashValue;
use circuits::context::{Context, Var};
use circuits::eval;
use circuits::ivalue::IValue;
use circuits::ops::{Guess, div};
use circuits::simd::Simd;
use circuits::wrappers::M31Wrapper;
use circuits_stark_verifier::constraint_eval::CircuitEval;
use circuits_stark_verifier::logup::{combine_term, logup_use_term};
use circuits_stark_verifier::proof_from_stark_proof::pack_into_qm31s;
use circuits_stark_verifier::statement::Statement;
use indexmap::IndexMap;
use itertools::{Itertools, zip_eq};
use stwo::core::fields::qm31::QM31;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

// TODO(ilya): Update this to the correct values.
pub const INTERACTION_POW_BITS: u32 = 20;

const N_LANES: usize = 16;

pub struct CircuitStatement<Value: IValue> {
    pub components: IndexMap<&'static str, Box<dyn CircuitEval<Value>>>,
    /// The variable indices (addresses) of the output gates.
    pub output_addresses: Vec<M31Wrapper<Var>>,
    /// The values of the output gates.
    pub output_values: Vec<Var>,
    /// The number of blake gates in the circuit.
    pub n_blake_gates: usize,
    /// Per-component trace log sizes packed as a [`Simd`].
    pub component_log_sizes: Simd,
    /// Preprocessed column ids in the exact order used by the prover's preprocessed trace.
    pub preprocessed_column_ids: Vec<PreProcessedColumnId>,
    /// Log size of each preprocessed column, in the same order as `preprocessed_column_ids`.
    pub preprocessed_column_log_sizes: Vec<u32>,
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
            n_blake_gates,
            n_blake_compress,
            preprocessed_column_ids,
            preprocessed_column_log_sizes,
            preprocessed_root,
        } = circuit_config;

        let output_addresses = output_addresses
            .iter()
            .map(|&addr| M31Wrapper::new_unsafe(context.constant(addr.into())))
            .collect_vec();
        let output_values =
            output_values.iter().map(|value| Value::from_qm31(*value).guess(context)).collect_vec();

        let component_log_sizes_u32 = component_log_sizes(
            *n_blake_compress,
            preprocessed_column_ids,
            preprocessed_column_log_sizes,
        );

        let n_components = component_log_sizes_u32.len();
        let packed_log_sizes = pack_into_qm31s(component_log_sizes_u32.iter().cloned())
            .into_iter()
            .map(|qm31| Value::from_qm31(qm31).guess(context))
            .collect_vec();
        let component_log_sizes = Simd::from_packed(packed_log_sizes, n_components);

        Self {
            components: all_circuit_components(),
            output_addresses,
            output_values,
            n_blake_gates: *n_blake_gates,
            component_log_sizes,
            preprocessed_column_ids: preprocessed_column_ids.clone(),
            preprocessed_column_log_sizes: preprocessed_column_log_sizes.clone(),
            preprocessed_root: *preprocessed_root,
        }
    }
}
/// Computes the per-component trace log sizes in the order expected by the prover.
pub fn component_log_sizes(
    n_blake_compress: usize,
    preprocessed_column_ids: &[PreProcessedColumnId],
    preprocessed_column_log_sizes: &[u32],
) -> [u32; 16] {
    let get_preprocessed_column_log_size = |column_name: &str| -> u32 {
        let column_id = PreProcessedColumnId { id: column_name.to_owned() };
        let idx = preprocessed_column_ids
            .iter()
            .position(|id| id == &column_id)
            .unwrap_or_else(|| panic!("Preprocessed column '{column_name}' not found"));
        preprocessed_column_log_sizes[idx]
    };

    // Trace size of the blake_gate component, in M31 rows. blake_gate is sized by the number of
    // blake compression blocks, padded up to the next power of two (and at least N_LANES).
    let blake_gate_n_rows = std::cmp::max(n_blake_compress.next_power_of_two(), N_LANES) as u32;
    let blake_gate_log_size = blake_gate_n_rows.ilog2();
    // blake_round receives 10 sub-component inputs per blake_gate (unpadded) row.
    let blake_round_log_size = ((10 * n_blake_compress) as u32).next_power_of_two().ilog2();
    // blake_g receives 8 sub-component inputs per (resized) blake_round row.
    let blake_g_log_size = blake_round_log_size + 3;
    // triple_xor_32 receives 8 sub-component inputs per blake_gate (unpadded) row.
    let triple_xor_32_log_size = ((8 * n_blake_compress) as u32).next_power_of_two().ilog2();

    [
        get_preprocessed_column_log_size("eq_in0_address"), // eq
        get_preprocessed_column_log_size("qm31_ops_in0_address"), // qm31_ops
        blake_gate_log_size,                                // blake_gate
        blake_round_log_size,                               // blake_round
        component_log_size::BLAKE_ROUND_SIGMA,              // blake_round_sigma
        blake_g_log_size,                                   // blake_g
        get_preprocessed_column_log_size("final_state_addr"), // blake_output
        triple_xor_32_log_size,                             // triple_xor_32
        get_preprocessed_column_log_size("m31_to_u32_input_addr"), // m_31_to_u_32
        component_log_size::VERIFY_BITWISE_XOR_8,           // verify_bitwise_xor_8
        component_log_size::VERIFY_BITWISE_XOR_12,          // verify_bitwise_xor_12
        component_log_size::VERIFY_BITWISE_XOR_4,           // verify_bitwise_xor_4
        component_log_size::VERIFY_BITWISE_XOR_7,           // verify_bitwise_xor_7
        component_log_size::VERIFY_BITWISE_XOR_9,           // verify_bitwise_xor_9
        component_log_size::RANGE_CHECK_15,                 // range_check_15
        component_log_size::RANGE_CHECK_16,                 // range_check_16
    ]
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

        // Blake IV public logup sum contribution.
        if self.n_blake_gates > 0 {
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
            let n_iv_uses = self.n_blake_gates.next_power_of_two();

            let n_blakes = context.constant((n_iv_uses as u32).into());
            let blake_iv_yield = div(context, n_blakes, blake_iv_denom);
            sum = eval!(context, (sum) - (blake_iv_yield));
        }

        sum
    }

    fn get_preprocessed_column_ids(&self) -> Vec<PreProcessedColumnId> {
        self.preprocessed_column_ids.clone()
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
        ("blake_gate", Box::new(blake_gate::Component {}) as Box<dyn CircuitEval<Value>>),
        ("blake_round", Box::new(blake_round::Component {}) as Box<dyn CircuitEval<Value>>),
        (
            "blake_round_sigma",
            Box::new(blake_round_sigma::Component {}) as Box<dyn CircuitEval<Value>>,
        ),
        ("blake_g", Box::new(blake_g::Component {}) as Box<dyn CircuitEval<Value>>),
        ("blake_output", Box::new(blake_output::Component {}) as Box<dyn CircuitEval<Value>>),
        ("triple_xor_32", Box::new(triple_xor_32::Component {}) as Box<dyn CircuitEval<Value>>),
        ("m_31_to_u_32", Box::new(m_31_to_u_32::Component {}) as Box<dyn CircuitEval<Value>>),
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
        ("range_check_15", Box::new(range_check_15::Component {}) as Box<dyn CircuitEval<Value>>),
        ("range_check_16", Box::new(range_check_16::Component {}) as Box<dyn CircuitEval<Value>>),
    ])
}
