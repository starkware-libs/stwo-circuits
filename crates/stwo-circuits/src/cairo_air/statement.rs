use std::collections::HashMap;

use crate::cairo_air::components;
use crate::circuits::ops::Guess;
use crate::circuits::wrappers::M31Wrapper;
use crate::stark_verifier::proof::Claim;
use cairo_air::air::{PublicData, PublicSegmentRanges};
use num_traits::Zero;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::cairo_air::preprocessed_columns::PREPROCESSED_COLUMNS_ORDER;
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::stark_verifier::constraint_eval::CircuitEval;
use crate::stark_verifier::statement::Statement;

pub struct PublicInput {
    pub public_data: PublicData,
    
}

pub struct CairoStatement<Value: IValue> {
    pub components: Vec<Box<dyn CircuitEval<Value>>>,
    pub public_params: HashMap<String, Var>,
    pub packed_initial_state_values: Var,
    pub packed_final_state_values: Var,
}
impl<Value: IValue> CairoStatement<Value> {
    pub fn new(context: &mut Context<Value>, public_input: PublicInput) -> Self
    where
        M31Wrapper<Value>: std::convert::From<M31>,
    {
        let PublicData { public_memory, initial_state, final_state } = public_input.public_data;
        let PublicSegmentRanges {
            output,
            pedersen: Some(pedersen),
            range_check_128: Some(range_check_128),
            ecdsa: Some(ecdsa),
            bitwise: Some(bitwise),
            ec_op: Some(ec_op),
            keccak: Some(keccak),
            poseidon: Some(poseidon),
            range_check_96: Some(range_check_96),
            add_mod: Some(add_mod),
            mul_mod: Some(mul_mod),
        } = public_memory.public_segments
        else {
            panic!("Public segment ranges are not present");
        };

        let state_as_qm31 = |[pc, ap, fp]: [M31; 3]| -> QM31 {
            QM31::from_m31(pc, ap, fp, M31::zero())
        };

        let packed_initial_state_values = Value::from_qm31(state_as_qm31(initial_state.values())).guess(context);
        let packed_final_state_values = Value::from_qm31(state_as_qm31(final_state.values())).guess(context);

        let public_params: HashMap<String, Var> = HashMap::from_iter(
            [
                ("output_start_ptr", output.start_ptr.value),
                ("pedersen_start_ptr", pedersen.start_ptr.value),
                ("range_check_128_start_ptr", range_check_128.start_ptr.value),
                ("ecdsa_start_ptr", ecdsa.start_ptr.value),
                ("bitwise_start_ptr", bitwise.start_ptr.value),
                ("ec_op_start_ptr", ec_op.start_ptr.value),
                ("keccak_start_ptr", keccak.start_ptr.value),
                ("poseidon_start_ptr", poseidon.start_ptr.value),
                ("range_check_96_start_ptr", range_check_96.start_ptr.value),
                ("add_mod_start_ptr", add_mod.start_ptr.value),
                ("mul_mod_start_ptr", mul_mod.start_ptr.value),
            ]
            .into_iter()
            .map(|(k, v)| {
                let m31_wrapper: M31Wrapper<Value> = M31::from_u32_unchecked(v).into();
                (k.to_string(), *m31_wrapper.guess(context).get())
            }),
        );

        Self {
            public_params,
            components: vec![
                Box::new(components::add_ap_opcode::Component {}),
                Box::new(components::assert_eq_opcode::Component {}),
                Box::new(components::jnz_opcode_non_taken::Component {}),
                Box::new(components::jnz_opcode_taken::Component {}),
                Box::new(components::jump_opcode_rel_imm::Component {}),
                Box::new(components::memory_address_to_id::Component {}),
                Box::new(components::memory_id_to_big::Component { index: 0 }),
                Box::new(components::memory_id_to_big::Component { index: 1 }),
                Box::new(components::memory_id_to_big::Component { index: 2 }),
                Box::new(components::memory_id_to_big::Component { index: 3 }),
                Box::new(components::memory_id_to_small::Component {}),
                Box::new(components::range_check_11::Component {}),
                Box::new(components::range_check_18::Component {}),
                Box::new(components::range_check_4_3::Component {}),
                Box::new(components::range_check_7_2_5::Component {}),
                Box::new(components::ret_opcode::Component {}),
                Box::new(components::verify_instruction::Component {}),
            ],
            packed_initial_state_values,
            packed_final_state_values,
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

    fn public_params(&self, _context: &mut Context<Value>) -> HashMap<String, Var> {
       self.public_params.clone()
    }
}
