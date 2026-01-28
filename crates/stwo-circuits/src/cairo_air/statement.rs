use std::array;
use std::collections::HashMap;

use crate::cairo_air::components;
use itertools::Itertools;
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
    + PUB_MEMORY_VALUE_LEN * OUTPUT_LEN;

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

pub struct CasmState<T> {
    pub pc: T,
    pub ap: T,
    pub fp: T,
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
    /// Output must be the last thing as it is variable length.
    pub output: Vec<PubMemoryValue<T>>,
}

pub struct PublicData<T> {
    pub initial_state: CasmState<T>,
    pub final_state: CasmState<T>,
    pub public_memory: PublicMemory<T>,
}

impl PublicData<Var> {
    /// Parses the public data from a slice of variables.
    pub fn parse_from_vars(data: &[Var]) -> Self {
        let mut iter = data.iter();

        let initial_state = CasmState {
            pc: *iter.next().unwrap(),
            ap: *iter.next().unwrap(),
            fp: *iter.next().unwrap(),
        };
        let final_state = CasmState {
            pc: *iter.next().unwrap(),
            ap: *iter.next().unwrap(),
            fp: *iter.next().unwrap(),
        };

        let segement_ranges = array::from_fn(|_| SegmentRange {
            start: PubMemoryM31Value { id: *iter.next().unwrap(), value: *iter.next().unwrap() },
            end: PubMemoryM31Value { id: *iter.next().unwrap(), value: *iter.next().unwrap() },
        });

        let safe_call_ids = [*iter.next().unwrap(), *iter.next().unwrap()];

        let mut output = vec![];
        for mut chunk in iter.chunks(PUB_MEMORY_VALUE_LEN).into_iter() {
            output.push(PubMemoryValue {
                id: *chunk.next().unwrap(),
                value: array::from_fn(|_| *chunk.next().unwrap()),
            });
        }
        Self {
            initial_state,
            final_state,
            public_memory: PublicMemory { segement_ranges, safe_call_ids, output },
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
                Box::new(components::call_opcode_rel_imm::Component {}),
                Box::new(components::jnz_opcode_non_taken::Component {}),
                Box::new(components::jnz_opcode_taken::Component {}),
                Box::new(components::jump_opcode_rel_imm::Component {}),
                Box::new(components::ret_opcode::Component {}),
                Box::new(components::verify_instruction::Component {}),
                Box::new(components::memory_address_to_id::Component {}),
                Box::new(components::memory_id_to_big::Component { index: 0 }),
                Box::new(components::memory_id_to_big::Component { index: 1 }),
                Box::new(components::memory_id_to_big::Component { index: 2 }),
                Box::new(components::memory_id_to_big::Component { index: 3 }),
                Box::new(components::memory_id_to_small::Component {}),
                Box::new(components::range_check_11::Component {}),
                Box::new(components::range_check_18::Component {}),
                Box::new(components::range_check_4_3::Component {}),
                Box::new(components::range_check_9_9::Component {}),
                Box::new(components::range_check_7_2_5::Component {}),
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
        let segement_ranges = &self.public_data.public_memory.segement_ranges;
        let public_params: HashMap<String, Var> = HashMap::from_iter(
            [
                ("output_start_ptr", &segement_ranges[0]),
                ("pedersen_start_ptr", &segement_ranges[1]),
                ("range_check_128_start_ptr", &segement_ranges[2]),
                ("ecdsa_start_ptr", &segement_ranges[3]),
                ("bitwise_start_ptr", &segement_ranges[4]),
                ("ec_op_start_ptr", &segement_ranges[5]),
                ("keccak_start_ptr", &segement_ranges[6]),
                ("poseidon_start_ptr", &segement_ranges[7]),
                ("range_check_96_start_ptr", &segement_ranges[8]),
                ("add_mod_start_ptr", &segement_ranges[9]),
                ("mul_mod_start_ptr", &segement_ranges[10]),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.start.value)),
        );
        public_params
    }
}
