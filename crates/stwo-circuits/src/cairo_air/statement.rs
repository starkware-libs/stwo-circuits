use std::array;
use std::collections::HashMap;

use crate::cairo_air::components;
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

const OUTPUT_LEN: usize = 1;
const PROGRAM_LEN: usize = 2048;
const N_SEGMENTS: usize = 11;
pub const PUBLIC_DATA_LEN: usize = N_SEGMENTS * 2 + 2 + 29 * (OUTPUT_LEN + PROGRAM_LEN);

#[derive(Clone)]
pub struct PubMemoryValue<T> {
    pub id: T,
    pub value: [T; 28],
}
impl Default for PubMemoryValue<M31> {
    fn default() -> Self {
        Self { id: M31::zero(), value: [M31::zero(); 28] }
    }
}

#[derive(Default)]
pub struct PublicMemory<T> {
    pub segement_starts: [T; N_SEGMENTS],
    pub segement_ends: [T; N_SEGMENTS],
    pub segement_start_ids: [T; N_SEGMENTS],
    pub segement_end_ids: [T; N_SEGMENTS],

    pub safe_call_ids: [T; 2],

    pub output: Vec<PubMemoryValue<T>>,
    pub program: Vec<PubMemoryValue<T>>,
}

pub struct PublicData<T> {
    pub public_memory: PublicMemory<T>,
    pub initial_state: [T; 3],
    pub final_state: [T; 3],
}

impl PublicData<Var> {
    pub fn unpack<Value: IValue>(data: &[Var]) -> Self {
        let mut iter = data.iter();
        let mut segement_ranges = vec![];

        let read_segement_data = |iter: &mut std::slice::Iter<'_, Var>| -> [Var; N_SEGMENTS] {
            array::from_fn(|_| *iter.next().unwrap())
        };

        let segement_starts = read_segement_data(&mut iter);
        let segement_ends = read_segement_data(&mut iter);
        let segement_start_ids = read_segement_data(&mut iter);
        let segement_end_ids = read_segement_data(&mut iter);

        for _ in 0..N_SEGMENTS {
            let start = iter.next().unwrap();
            let end = iter.next().unwrap();
            segement_ranges.push((*start, *end));
        }
        let initial_state = [*iter.next().unwrap(), *iter.next().unwrap(), *iter.next().unwrap()];
        let final_state = [*iter.next().unwrap(), *iter.next().unwrap(), *iter.next().unwrap()];

        let get_public_mem_value = |iter: &mut std::slice::Iter<'_, Var>| PubMemoryValue {
            id: *iter.next().unwrap(),
            value: array::from_fn(|_| *iter.next().unwrap()),
        };

        let output = vec![get_public_mem_value(&mut iter)];
        let mut program = vec![];
        while iter.len() > 0 {
            program.push(get_public_mem_value(&mut iter));
        }
        Self {
            public_memory: PublicMemory {
                segement_starts,
                segement_ends,
                segement_start_ids,
                segement_end_ids,
                output,
                program,
                safe_call_ids: [*iter.next().unwrap(), *iter.next().unwrap()],
            },
            initial_state,
            final_state,
        }
    }
}

pub struct CairoStatement<Value: IValue> {
    pub components: Vec<Box<dyn CircuitEval<Value>>>,
    pub packed_public_data: Simd,
    pub public_data: PublicData<Var>,
}

impl<Value: IValue> CairoStatement<Value> {
    pub fn new(
        context: &mut Context<Value>,
        packed_public_data: Vec<Var>,
        public_data_len: usize,
    ) -> Self {
        let packed_public_data = Simd::from_packed(packed_public_data, public_data_len);
        let unpacked_simd = Simd::unpack(context, &packed_public_data);
        let public_data = PublicData::<Var>::unpack::<Value>(&unpacked_simd[..]);

        Self {
            packed_public_data,
            public_data,
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
        }
    }
}

impl<Value: IValue> Statement<Value> for CairoStatement<Value> {
    fn get_components(&self) -> &[Box<dyn CircuitEval<Value>>] {
        &self.components
    }

    fn packed_public_data(&self) -> &[Var] {
        self.packed_public_data.get_packed()
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
        let segement_starts = self.public_data.public_memory.segement_starts;
        let public_params: HashMap<String, Var> = HashMap::from_iter(
            [
                ("output_start_ptr", segement_starts[0]),
                ("pedersen_start_ptr", segement_starts[1]),
                ("range_check_128_start_ptr", segement_starts[2]),
                ("ecdsa_start_ptr", segement_starts[3]),
                ("bitwise_start_ptr", segement_starts[4]),
                ("ec_op_start_ptr", segement_starts[5]),
                ("keccak_start_ptr", segement_starts[6]),
                ("poseidon_start_ptr", segement_starts[7]),
                ("range_check_96_start_ptr", segement_starts[8]),
                ("add_mod_start_ptr", segement_starts[9]),
                ("mul_mod_start_ptr", segement_starts[10]),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v)),
        );
        public_params
    }
}
