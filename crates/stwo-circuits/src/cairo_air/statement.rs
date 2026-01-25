use std::array;
use std::collections::HashMap;

use crate::cairo_air::components;
use crate::cairo_air::components::memory_address_to_id::MEMORY_ADDRESS_TO_ID_RELATION_ID;
use crate::circuits::ops::div;
use crate::eval;
use crate::stark_verifier::extract_bits::extract_bits;
use crate::stark_verifier::logup::combine_term;
use itertools::{Itertools, chain};
use num_traits::Zero;
use stwo::core::fields::qm31::QM31;
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

const MEMRORY_ID_TO_VALUE_RELATION_ID: u32 = 1662111297;

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

impl PubMemoryM31Value<Var> {
    /// Computes the address to id logup term for the public memory value.
    pub fn logup_term(
        &self,
        context: &mut Context<impl IValue>,
        interaction_elements: [Var; 2],
    ) -> Var {
        let simd = Simd::from_packed(vec![self.value], 1);
        // TODO(ilya): Consider using the logup itself for the range check instead of extracting
        // bits.
        let extracted_bits = extract_bits(context, &simd, 27);

        let bits = extracted_bits.iter().map(|bits| Simd::unpack(context, bits)[0]).collect_vec();

        let element = chain!(
            [context.constant(MEMRORY_ID_TO_VALUE_RELATION_ID.into()), self.id],
            bits.chunks(9).map(|limb_bits| combine_bits(context, limb_bits))
        )
        .collect_vec();

        let combined = combine_term(context, &element, interaction_elements);
        div(context, context.one(), combined)
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
        for var in
            unpacked_simd.iter().skip(2 * STATE_LEN + 2 * PUB_MEMORY_VALUE_M31_LEN * N_SEGMENTS)
        {
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

pub fn combine_bits(context: &mut Context<impl IValue>, bits: &[Var]) -> Var {
    let mut iter = bits.iter().rev();
    let mut res = *iter.next().unwrap();
    let two = context.constant(QM31::from(2));
    for bit in iter {
        res = eval!(context, ((res) * (two)) + (*bit));
    }
    res
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
        interaction_elements: [Var; 2],
        _claim: &Claim<Var>,
    ) -> Var {
        public_logup_sum(context, &self.public_data, interaction_elements)
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

    let address_to_id_logup_denominator = combine_term(
        context,
        &[memory_address_to_id_relation_id, address, id],
        interaction_elements,
    );
    div(context, context.one(), address_to_id_logup_denominator)
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

pub fn public_logup_sum(
    context: &mut Context<impl IValue>,
    public_data: &PublicData<Var>,
    interaction_elements: [Var; 2],
) -> Var {
    let initial_ap = public_data.initial_state.ap;
    context.mark_as_unused(public_data.initial_state.pc);
    context.mark_as_unused(public_data.final_state.pc);
    context.mark_as_unused(public_data.initial_state.fp);
    context.mark_as_unused(public_data.final_state.fp);

    let argument_address = initial_ap;
    let return_value_address = eval!(
        context,
        (public_data.final_state.ap) - (context.constant(QM31::from(N_SEGMENTS as u32)))
    );
    segment_range_logup_sum(
        context,
        interaction_elements,
        &public_data.public_memory.segement_ranges,
        argument_address,
        return_value_address,
    )

    // TODO(ilya): Add missing logup terms.
}
