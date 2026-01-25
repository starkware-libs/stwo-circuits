use std::array;

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
const N_SAFE_CALL_IDS: usize = 2;

// A memory value is stored as 28 9bit limbs.
const MEMORY_VALUES_LIMBS: usize = 28;
const PUB_MEMORY_VALUE_LEN: usize = 1 + MEMORY_VALUES_LIMBS;
const PUB_MEMORY_VALUE_M31_LEN: usize = 2;
const STATE_LEN: usize = 3;
pub const PUBLIC_DATA_LEN: usize = 2 * STATE_LEN
    + 2 * PUB_MEMORY_VALUE_M31_LEN * N_SEGMENTS
    + N_SAFE_CALL_IDS
    + PUB_MEMORY_VALUE_LEN * (OUTPUT_LEN + PROGRAM_LEN);

#[derive(Clone)]
pub struct PubMemoryValue<T> {
    pub id: T,
    pub value: [T; MEMORY_VALUES_LIMBS],
}
impl Default for PubMemoryValue<M31> {
    fn default() -> Self {
        Self { id: M31::zero(), value: [M31::zero(); MEMORY_VALUES_LIMBS] }
    }
}

// A public memory value that fits in 27bits.
pub struct PubMemoryM31Value<T> {
    pub id: T,
    pub value: T,
}

pub struct SegmentRange<T> {
    pub start: PubMemoryM31Value<T>,
    pub end: PubMemoryM31Value<T>,
}

pub struct PublicMemory<T> {
    pub segement_ranges: [SegmentRange<T>; N_SEGMENTS],
    pub safe_call_ids: [T; 2],

    pub output: Vec<PubMemoryValue<T>>,
    pub program: Vec<PubMemoryValue<T>>,
}

pub struct PublicData<T> {
    pub initial_state: [T; 3],
    pub final_state: [T; 3],
    pub public_memory: PublicMemory<T>,
}

impl PublicData<Var> {
    /// Parses the public data from a slice of variables.
    pub fn parse_from_vars(data: &[Var]) -> Self {
        let mut iter = data.iter();

        let initial_state = [*iter.next().unwrap(), *iter.next().unwrap(), *iter.next().unwrap()];
        let final_state = [*iter.next().unwrap(), *iter.next().unwrap(), *iter.next().unwrap()];

        let read_segement_data =
            |iter: &mut std::slice::Iter<'_, Var>| -> [SegmentRange<Var>; N_SEGMENTS] {
                array::from_fn(|_| SegmentRange {
                    start: PubMemoryM31Value {
                        id: *iter.next().unwrap(),
                        value: *iter.next().unwrap(),
                    },
                    end: PubMemoryM31Value {
                        id: *iter.next().unwrap(),
                        value: *iter.next().unwrap(),
                    },
                })
            };

        let segement_ranges = read_segement_data(&mut iter);

        let get_public_mem_value = |iter: &mut std::slice::Iter<'_, Var>| PubMemoryValue {
            id: *iter.next().unwrap(),
            value: array::from_fn(|_| *iter.next().unwrap()),
        };

        let safe_call_ids = [*iter.next().unwrap(), *iter.next().unwrap()];

        let output = vec![get_public_mem_value(&mut iter)];
        let mut program = vec![];
        while iter.len() > 0 {
            program.push(get_public_mem_value(&mut iter));
        }
        Self {
            initial_state,
            final_state,
            public_memory: PublicMemory { segement_ranges, safe_call_ids, output, program },
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

        // TODO(ilya): Remove once we handle the public data properly.
        for var in unpacked_simd.iter() {
            context.mark_as_unused(*var);
        }
        let public_data = PublicData::<Var>::parse_from_vars(&unpacked_simd[..]);

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
