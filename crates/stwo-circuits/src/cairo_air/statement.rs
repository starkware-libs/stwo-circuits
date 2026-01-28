use std::array;
use std::collections::HashMap;

use crate::cairo_air::components;
use crate::circuits::blake::blake_qm31;
use crate::circuits::ops::Guess;
use crate::eval;
use crate::stark_verifier::extract_bits::extract_bits;
use crate::stark_verifier::logup::logup_use_term;
use crate::stark_verifier::proof_from_stark_proof::pack_into_qm31s;
use cairo_air::relations::{
    MEMORY_ADDRESS_TO_ID_RELATION_ID, MEMORY_ID_TO_BIG_RELATION_ID, OPCODES_RELATION_ID,
};
use itertools::{Itertools, chain, izip, zip_eq};
use stwo::core::fields::qm31::{QM31, SECURE_EXTENSION_DEGREE};
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
const N_SAFE_CALL_IDS: usize = 2;

// A memory value is stored as 28 9bit limbs.
pub const MEMORY_VALUES_LIMBS: usize = 28;
pub const PUB_MEMORY_VALUE_LEN: usize = 1 + MEMORY_VALUES_LIMBS;
const PUB_MEMORY_VALUE_M31_LEN: usize = 2;
const STATE_LEN: usize = 3;
pub const PUBLIC_DATA_LEN: usize =
    2 * STATE_LEN + 2 * PUB_MEMORY_VALUE_M31_LEN * N_SEGMENTS + N_SAFE_CALL_IDS;

const LIMB_BITS: usize = 9;
const SMALL_VALUE_BITS: u32 = 27;

#[derive(Clone)]
pub struct PubMemoryValue<T> {
    pub id: T,
    pub value: [T; MEMORY_VALUES_LIMBS],
}

impl PubMemoryValue<Var> {
    /// Computes the address to id logup term for the public memory value.
    pub fn logup_term(
        &self,
        context: &mut Context<impl IValue>,
        interaction_elements: [Var; 2],
    ) -> Var {
        id_to_big_logup_term(context, self.id, self.value.iter().copied(), interaction_elements)
    }
}

pub struct CasmState<T> {
    pub pc: T,
    pub ap: T,
    pub fp: T,
}
impl CasmState<Var> {
    pub fn logup_term(
        &self,
        context: &mut Context<impl IValue>,
        interaction_elements: [Var; 2],
    ) -> Var {
        let Self { pc, ap, fp } = self;
        let elements = [context.constant(OPCODES_RELATION_ID.into()), *pc, *ap, *fp];
        logup_use_term(context, &elements, interaction_elements)
    }
}

// A public memory value that fits in 27bits.
pub struct PubMemoryM31Value<T> {
    pub id: T,
    pub value: T,
}

pub fn split_27bit_to_9bit_limbs(context: &mut Context<impl IValue>, value: Var) -> [Var; 3] {
    let simd = Simd::from_packed(vec![value], 1);
    let extracted_bits = extract_bits(context, &simd, SMALL_VALUE_BITS);

    let mut limbs_iter = extracted_bits.chunks(LIMB_BITS).map(|limb_bits| {
        let limb = Simd::combine_bits(context, limb_bits);
        Simd::unpack(context, &limb)[0]
    });
    array::from_fn(|_| limbs_iter.next().unwrap())
}

impl PubMemoryM31Value<Var> {
    /// Computes the address to id logup term for the public memory value.
    pub fn logup_term(
        &self,
        context: &mut Context<impl IValue>,
        interaction_elements: [Var; 2],
    ) -> Var {
        let limbs = split_27bit_to_9bit_limbs(context, self.value);
        id_to_big_logup_term(context, self.id, limbs.into_iter(), interaction_elements)
    }
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
    pub program_ids: Vec<T>,
}

pub struct PublicData<T> {
    pub initial_state: CasmState<T>,
    pub final_state: CasmState<T>,
    pub public_memory: PublicMemory<T>,
}

impl PublicData<Var> {
    /// Parses the public data from a slice of variables.
    pub fn parse_from_vars(data: &[Var], output_len: usize, program_len: usize) -> Self {
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
        let output_iter = iter.by_ref().take(output_len * PUB_MEMORY_VALUE_LEN);
        for mut chunk in output_iter.chunks(PUB_MEMORY_VALUE_LEN).into_iter() {
            output.push(PubMemoryValue {
                id: *chunk.next().unwrap(),
                value: array::from_fn(|_| *chunk.next().unwrap()),
            });
        }

        let program_ids = iter.cloned().collect_vec();
        assert_eq!(program_ids.len(), program_len);

        Self {
            initial_state,
            final_state,
            public_memory: PublicMemory { segement_ranges, safe_call_ids, output, program_ids },
        }
    }
}

pub struct CairoStatement<Value: IValue> {
    pub components: Vec<Box<dyn CircuitEval<Value>>>,
    pub packed_public_data: Vec<Var>,
    pub public_data: PublicData<Var>,
    pub program: Vec<[M31; MEMORY_VALUES_LIMBS]>,
}

impl<Value: IValue> CairoStatement<Value> {
    pub fn new(
        context: &mut Context<Value>,
        public_data: Vec<M31>,
        output_len: usize,
        program: Vec<[M31; MEMORY_VALUES_LIMBS]>,
    ) -> Self {
        let mut packed_public_data = pack_into_qm31s(public_data.iter().cloned())
            .into_iter()
            .map(|qm31| Value::from_qm31(qm31).guess(context))
            .collect_vec();

        let simd_public_data = Simd::from_packed(packed_public_data.clone(), public_data.len());
        // Note that we don't enforce anything on the padding M31 in packed_public_data.
        let unpacked_simd = Simd::unpack(context, &simd_public_data);

        let public_data =
            PublicData::<Var>::parse_from_vars(&unpacked_simd[..], output_len, program.len());

        let flat_program = pack_into_qm31s(program.iter().flatten().cloned());
        let program_hash = blake_qm31(
            &flat_program,
            (program.len() * MEMORY_VALUES_LIMBS).div_ceil(SECURE_EXTENSION_DEGREE) * 16,
        );
        packed_public_data.push(context.constant(program_hash.0));
        packed_public_data.push(context.constant(program_hash.1));

        Self {
            packed_public_data,
            public_data,
            program,
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
        &self.packed_public_data
    }

    fn public_logup_sum(
        &self,
        context: &mut Context<Value>,
        interaction_elements: [Var; 2],
        _claim: &Claim<Var>,
    ) -> Var {
        public_logup_sum(context, &self.public_data, &self.program[..], interaction_elements)
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

pub fn address_to_id_logup_term(
    context: &mut Context<impl IValue>,
    address: Var,
    id: Var,
    interaction_elements: [Var; 2],
) -> Var {
    let memory_address_to_id_relation_id =
        context.constant(MEMORY_ADDRESS_TO_ID_RELATION_ID.into());

    logup_use_term(context, &[memory_address_to_id_relation_id, address, id], interaction_elements)
}

/// Calculates the logup term for a provided id and its associated value limbs.
/// Each value limb is 9 bits wide, with the least significant limb appearing first.
pub fn id_to_big_logup_term(
    context: &mut Context<impl IValue>,
    id: Var,
    value_limbs: impl Iterator<Item = Var>,
    interaction_elements: [Var; 2],
) -> Var {
    let memory_id_to_big_relation_id = context.constant(MEMORY_ID_TO_BIG_RELATION_ID.into());
    let elements = chain!([memory_id_to_big_relation_id, id], value_limbs).collect_vec();
    logup_use_term(context, &elements, interaction_elements)
}

pub fn segment_range_logup_sum(
    context: &mut Context<impl IValue>,
    interaction_elements: [Var; 2],
    segement_ranges: &[SegmentRange<Var>; N_SEGMENTS],
    mut argument_address: Var,
    mut return_value_address: Var,
) -> Var {
    let one = context.one();
    let mut sum = context.zero();
    for (i, segment_range) in segement_ranges.iter().enumerate() {
        if i != 0 {
            argument_address = eval!(context, (argument_address) + (one));
            return_value_address = eval!(context, (return_value_address) + (one));
        }

        let arg_address_to_id_logup_term = address_to_id_logup_term(
            context,
            argument_address,
            segment_range.start.id,
            interaction_elements,
        );
        let return_value_to_id_logup_term = address_to_id_logup_term(
            context,
            return_value_address,
            segment_range.end.id,
            interaction_elements,
        );

        sum = eval!(context, (sum) + (arg_address_to_id_logup_term));
        sum = eval!(context, (sum) + (return_value_to_id_logup_term));

        sum =
            eval!(context, (sum) + (segment_range.start.logup_term(context, interaction_elements)));
        sum = eval!(context, (sum) + (segment_range.end.logup_term(context, interaction_elements)));
    }

    sum
}

fn safe_call_id_logup_term(
    context: &mut Context<impl IValue>,
    interaction_elements: [Var; 2],
    address: Var,
    id: Var,
    value_limbs: &[Var],
) -> Var {
    let address_to_id_logup_term =
        address_to_id_logup_term(context, address, id, interaction_elements);

    let id_to_value_logup_term =
        id_to_big_logup_term(context, id, value_limbs.iter().cloned(), interaction_elements);
    eval!(context, (address_to_id_logup_term) + (id_to_value_logup_term))
}

pub fn memory_segments_logup_sum(
    context: &mut Context<impl IValue>,
    interaction_elements: [Var; 2],
    mut start_address: Var,
    memory_values: &[PubMemoryValue<Var>],
) -> Var {
    let one = context.one();
    let mut sum = context.zero();

    for (i, memory_value) in memory_values.iter().enumerate() {
        if i != 0 {
            start_address = eval!(context, (start_address) + (one));
        }

        let address_to_id_logup_term =
            address_to_id_logup_term(context, start_address, memory_value.id, interaction_elements);
        sum = eval!(context, (sum) + (address_to_id_logup_term));

        let id_to_value_logup_term = memory_value.logup_term(context, interaction_elements);
        sum = eval!(context, (sum) + (id_to_value_logup_term));
    }

    sum
}

pub fn public_logup_sum(
    context: &mut Context<impl IValue>,
    public_data: &PublicData<Var>,
    program: &[[M31; MEMORY_VALUES_LIMBS]],
    interaction_elements: [Var; 2],
) -> Var {
    let PublicData {
        initial_state,
        final_state,
        public_memory: PublicMemory { segement_ranges, safe_call_ids, output, program_ids },
    } = public_data;
    let initial_ap = initial_state.ap;
    let final_ap = final_state.ap;
    let final_state_logup_term = public_data.final_state.logup_term(context, interaction_elements);
    let initial_state_logup_term =
        public_data.initial_state.logup_term(context, interaction_elements);
    let mut sum = eval!(context, (final_state_logup_term) - (initial_state_logup_term));

    let one = context.one();
    let safe_call_addresses = vec![
        eval!(context, (initial_ap) - (context.constant(QM31::from(2)))),
        eval!(context, (initial_ap) - (one)),
    ];

    let split_initial_ap = split_27bit_to_9bit_limbs(context, initial_ap);
    let safe_call_values = [split_initial_ap.as_slice(), &[]];
    // Handle the safe call memory section::
    // memory[initial_ap - 2] = (safe_call_id0, initial_ap)
    // memory[initial_ap - 1] = (safe_call_id1, 0).
    for (address, id, value_limbs) in izip!(safe_call_addresses, safe_call_ids, safe_call_values) {
        let logup_term =
            safe_call_id_logup_term(context, interaction_elements, address, *id, value_limbs);
        sum = eval!(context, (sum) + (logup_term));
    }

    let argument_address = initial_ap;
    let return_value_address =
        eval!(context, (final_ap) - (context.constant(QM31::from(N_SEGMENTS as u32))));
    let segment_ranges_logup_sum = segment_range_logup_sum(
        context,
        interaction_elements,
        segement_ranges,
        argument_address,
        return_value_address,
    );
    sum = eval!(context, (sum) + (segment_ranges_logup_sum));

    let output_logup_sum = memory_segments_logup_sum(
        context,
        interaction_elements,
        segement_ranges[0].start.value,
        output,
    );
    sum = eval!(context, (sum) + (output_logup_sum));

    let program_with_ids = zip_eq(program, program_ids)
        .map(|(program_word, &id)| PubMemoryValue {
            id,
            value: program_word.map(|word| context.constant(word.into())),
        })
        .collect_vec();

    let program_logup_sum = memory_segments_logup_sum(
        context,
        interaction_elements,
        initial_state.pc,
        &program_with_ids,
    );
    sum = eval!(context, (sum) + (program_logup_sum));

    sum
}
