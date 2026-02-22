use crate::circuit_eval_components::{
    blake_g, blake_gate, blake_output, blake_round, blake_round_sigma, range_check_15,
    range_check_16, triple_xor_32, verify_bitwise_xor_12, verify_bitwise_xor_4,
    verify_bitwise_xor_7, verify_bitwise_xor_8, verify_bitwise_xor_9,
};
use crate::components::{eq, qm31_ops};
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::preprocessed_columns::PREPROCESSED_COLUMNS_ORDER;
use circuits::context::{Context, Var};
use circuits::eval;
use circuits::extract_bits::extract_bits;
use circuits::ivalue::IValue;
use circuits::simd::Simd;
use circuits_stark_verifier::logup::logup_use_term;
use circuits_stark_verifier::proof::Claim;
use circuits_stark_verifier::verify::LOG_SIZE_BITS;

use circuits_stark_verifier::constraint_eval::CircuitEval;
use circuits_stark_verifier::statement::Statement;
use stwo::core::fields::m31::M31;

// TODO(ilya): Update this to to correct values.
pub const INTERACTION_POW_BITS: u32 = 8;

pub struct CircuitStatement<Value: IValue> {
    pub components: Vec<Box<dyn CircuitEval<Value>>>,
    pub preprocessed_column_ids: Vec<PreProcessedColumnId>,
}
impl<Value: IValue> Default for CircuitStatement<Value> {
    fn default() -> Self {
        Self {
            components: vec![
                Box::new(eq::CircuitEqComponent {}),
                Box::new(qm31_ops::CircuitQm31OpsComponent {}),
                Box::new(blake_gate::Component {}),
                Box::new(blake_round::Component {}),
                Box::new(blake_round_sigma::Component {}),
                Box::new(blake_g::Component {}),
                Box::new(blake_output::Component {}),
                Box::new(triple_xor_32::Component {}),
                Box::new(verify_bitwise_xor_8::Component {}),
                Box::new(verify_bitwise_xor_12::Component {}),
                Box::new(verify_bitwise_xor_4::Component {}),
                Box::new(verify_bitwise_xor_7::Component {}),
                Box::new(verify_bitwise_xor_9::Component {}),
                Box::new(range_check_15::Component {}),
                Box::new(range_check_16::Component {}),
            ],
            preprocessed_column_ids: vec![],
        }
    }
}
impl<Value: IValue> CircuitStatement<Value> {
    pub fn with_component_log_sizes(
        component_log_sizes: &[u32; crate::components::N_COMPONENTS],
    ) -> Self {
        let eq_log_size = component_log_sizes[crate::components::ComponentList::Eq as usize];
        let qm31_ops_log_size =
            component_log_sizes[crate::components::ComponentList::Qm31Ops as usize];
        let blake_gate_log_size =
            component_log_sizes[crate::components::ComponentList::BlakeGate as usize];
        let blake_output_log_size =
            component_log_sizes[crate::components::ComponentList::BlakeOutput as usize];

        let mut by_size_then_original = PREPROCESSED_COLUMNS_ORDER
            .iter()
            .enumerate()
            .map(|(idx, id)| {
                let size = match *id {
                    "eq_in0_address" | "eq_in1_address" => eq_log_size,
                    "qm31_ops_add_flag"
                    | "qm31_ops_sub_flag"
                    | "qm31_ops_mul_flag"
                    | "qm31_ops_pointwise_mul_flag"
                    | "qm31_ops_in0_address"
                    | "qm31_ops_in1_address"
                    | "qm31_ops_out_address"
                    | "qm31_ops_mults" => qm31_ops_log_size,
                    "t0"
                    | "t1"
                    | "finalize_flag"
                    | "state_before_addr"
                    | "state_after_addr"
                    | "message0_addr"
                    | "message1_addr"
                    | "message2_addr"
                    | "message3_addr"
                    | "compress_enabler" => blake_gate_log_size,
                    "final_state_addr"
                    | "blake_output0_addr"
                    | "blake_output1_addr"
                    | "blake_output0_mults"
                    | "blake_output1_mults" => blake_output_log_size,
                    _ if id.starts_with("blake_sigma_") => 4,
                    _ if id.starts_with("seq_") => id.strip_prefix("seq_").unwrap().parse().unwrap(),
                    _ if id.starts_with("bitwise_xor_4_") => 8,
                    _ if id.starts_with("bitwise_xor_7_") => 14,
                    _ if id.starts_with("bitwise_xor_8_") => 16,
                    _ if id.starts_with("bitwise_xor_9_") => 18,
                    _ if id.starts_with("bitwise_xor_10_") => 20,
                    _ => panic!("Unsupported preprocessed column id: {id}"),
                };
                (idx, *id, size)
            })
            .collect::<Vec<_>>();
        by_size_then_original.sort_by_key(|(idx, _, size)| (*size, *idx));

        let mut statement = Self::default();
        statement.preprocessed_column_ids = by_size_then_original
            .into_iter()
            .map(|(_, id, _)| PreProcessedColumnId { id: id.to_string() })
            .collect();
        statement
    }

    pub fn with_preprocessed_trace_sizes(preprocessed_trace_sizes: &[u32]) -> Self {
        assert_eq!(
            preprocessed_trace_sizes.len(),
            PREPROCESSED_COLUMNS_ORDER.len(),
            "Invalid number of preprocessed columns"
        );

        let mut by_size_then_original = PREPROCESSED_COLUMNS_ORDER
            .iter()
            .zip(preprocessed_trace_sizes.iter().copied())
            .enumerate()
            .map(|(idx, (id, size))| (idx, *id, size))
            .collect::<Vec<_>>();
        by_size_then_original.sort_by_key(|(idx, _, size)| (*size, *idx));

        let mut statement = Self::default();
        statement.preprocessed_column_ids = by_size_then_original
            .into_iter()
            .map(|(_, id, _)| PreProcessedColumnId { id: id.to_string() })
            .collect();
        statement
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
        interaction_elements: [Var; 2],
        claim: &Claim<Var>,
    ) -> Var {
        let initial_state = [
            1795745351u32,
            3144134277,
            1013904242,
            2773480762,
            1359893119,
            2600822924,
            528734635,
            1541459225,
        ];
        let state_id = context.constant(M31::from(1061955672).into());
        let zero = context.zero();

        let component_log_sizes =
            Simd::from_packed(claim.packed_component_log_sizes.clone(), crate::components::N_COMPONENTS);
        let component_log_size_bits = extract_bits(context, &component_log_sizes, LOG_SIZE_BITS);
        let component_sizes = Simd::pow2(context, &component_log_size_bits);
        let n_blake_gates =
            Simd::unpack_idx(context, &component_sizes, crate::components::ComponentList::BlakeGate as usize);
        Simd::mark_partly_used(context, &component_log_sizes);
        for bit_simd in &component_log_size_bits {
            Simd::mark_partly_used(context, bit_simd);
        }
        Simd::mark_partly_used(context, &component_sizes);

        let mut element = vec![state_id, zero];
        for word in initial_state {
            element.push(context.constant(M31::from(word & 0xffff).into()));
            element.push(context.constant(M31::from((word >> 16) & 0xffff).into()));
        }

        let iv_use_term = logup_use_term(context, &element, interaction_elements);
        eval!(context, -((n_blake_gates) * (iv_use_term)))
    }

    fn get_preprocessed_column_ids(&self) -> Vec<PreProcessedColumnId> {
        self.preprocessed_column_ids.clone()
    }
}
