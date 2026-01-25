use crate::cairo_air::components;
use crate::stark_verifier::proof_from_stark_proof::pack_into_qm31s;
use num_traits::Zero;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::cairo_air::preprocessed_columns::PREPROCESSED_COLUMNS_ORDER;
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::simd::Simd;
use crate::stark_verifier::constraint_eval::CircuitEval;
use crate::stark_verifier::proof::Claim;
use crate::stark_verifier::statement::Statement;
use stwo::core::fields::m31::M31;

const N_SEGMENTS: usize = 11;

#[derive(Default)]
pub struct PublicMemory<T> {
    segement_ranges: [(T, T); 11],
}

pub struct PublicData<T> {
    pub public_memory: PublicMemory<T>,
    pub initial_state: [T; 3],
    pub final_state: [T; 3],
}

pub struct FlatClaim<T> {
    public_data: PublicData<T>,
}

impl Default for FlatClaim<M31> {
    fn default() -> Self {
        Self {
            public_data: PublicData {
                public_memory: PublicMemory { segement_ranges: [(M31::zero(), M31::zero()); 11] },
                initial_state: [M31::zero(), M31::zero(), M31::zero()],
                final_state: [M31::zero(), M31::zero(), M31::zero()],
            },
        }
    }
}

impl FlatClaim<M31> {
    pub fn pack<Value: IValue>(&self) -> Vec<Value> {
        let mut data = vec![];
        for (start, end) in &self.public_data.public_memory.segement_ranges {
            data.push(*start);
            data.push(*end);
        }
        data.push(self.public_data.initial_state[0]);
        data.push(self.public_data.initial_state[1]);
        data.push(self.public_data.initial_state[2]);
        data.push(self.public_data.final_state[0]);
        data.push(self.public_data.final_state[1]);
        data.push(self.public_data.final_state[2]);

        pack_into_qm31s(data.into_iter()).into_iter().map(|v| Value::from_qm31(v)).collect()
    }
}

impl FlatClaim<Var> {
    pub fn unpack<Value: IValue>(data: &[Var]) -> Self {
        let mut iter = data.iter();
        let mut segement_ranges = vec![];
        for _ in 0..N_SEGMENTS {
            let start = iter.next().unwrap();
            let end = iter.next().unwrap();
            segement_ranges.push((*start, *end));
        }
        let initial_state = [*iter.next().unwrap(), *iter.next().unwrap(), *iter.next().unwrap()];
        let final_state = [*iter.next().unwrap(), *iter.next().unwrap(), *iter.next().unwrap()];
        Self {
            public_data: PublicData {
                public_memory: PublicMemory {
                    segement_ranges: segement_ranges.try_into().unwrap(),
                },
                initial_state,
                final_state,
            },
        }
    }
}

pub struct CairoStatement<Value: IValue> {
    pub components: Vec<Box<dyn CircuitEval<Value>>>,
    pub packed_public_data: Simd,
    pub flat_claim: FlatClaim<Var>,
}

impl<Value: IValue> CairoStatement<Value> {
    pub fn new(context: &mut Context<Value>, packed_semgent_ranges: Vec<Var>) -> Self {
        let packed_public_data = Simd::from_packed(packed_semgent_ranges, N_SEGMENTS * 2);
        let unpacked_simd = Simd::unpack(context, &packed_public_data);
        let flat_claim = FlatClaim::<Var>::unpack::<Value>(&unpacked_simd[..]);

        Self {
            packed_public_data,
            flat_claim,
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
