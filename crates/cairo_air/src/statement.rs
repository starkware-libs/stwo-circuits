use std::array;
use std::collections::HashMap;
use std::sync::Arc;

use cairo_air::PreProcessedTraceVariant;
use cairo_air::components::memory_address_to_id::MEMORY_ADDRESS_TO_ID_SPLIT;
use cairo_air::relations::{
    MEMORY_ADDRESS_TO_ID_RELATION_ID, MEMORY_ID_TO_BIG_RELATION_ID, OPCODES_RELATION_ID,
};
use circuits::blake::{HashValue, blake};
use circuits::eval;
use circuits::extract_bits::extract_bits;
use circuits::ops::{Guess, eq, output};
use circuits::wrappers::M31Wrapper;
use circuits_stark_verifier::logup::logup_use_term;
use circuits_stark_verifier::proof_from_stark_proof::pack_into_qm31s;
use circuits_stark_verifier::verify::RELATION_USES_NUM_ROWS_SHIFT;
use itertools::{Itertools, chain, izip, zip_eq};
use stwo::core::fields::qm31::QM31;
use stwo_cairo_common::builtins::{
    BITWISE_BUILTIN_MEMORY_CELLS, PEDERSEN_BUILTIN_MEMORY_CELLS, POSEIDON_BUILTIN_MEMORY_CELLS,
    RANGE_CHECK_BUILTIN_MEMORY_CELLS,
};
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::all_components::all_components;
use crate::preprocessed_columns::MAX_SEQUENCE_LOG_SIZE;
use circuits::context::{Context, Var};
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::simd::Simd;
use circuits_stark_verifier::constraint_eval::CircuitEval;
use circuits_stark_verifier::statement::Statement;
use stwo::core::fields::m31::M31;

#[cfg(test)]
#[path = "statement_test.rs"]
pub mod test;

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
    pub segment_ranges: [SegmentRange<T>; N_SEGMENTS],
    pub safe_call_ids: [T; 2],
    pub output_ids: Vec<T>,
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

        let segment_ranges = array::from_fn(|_| SegmentRange {
            start: PubMemoryM31Value { id: *iter.next().unwrap(), value: *iter.next().unwrap() },
            end: PubMemoryM31Value { id: *iter.next().unwrap(), value: *iter.next().unwrap() },
        });

        let safe_call_ids = [*iter.next().unwrap(), *iter.next().unwrap()];
        let output_ids = iter.by_ref().take(output_len).cloned().collect_vec();
        let program_ids = iter.cloned().collect_vec();
        assert_eq!(program_ids.len(), program_len);

        Self {
            initial_state,
            final_state,
            public_memory: PublicMemory { segment_ranges, safe_call_ids, output_ids, program_ids },
        }
    }
}

pub struct CairoStatement<Value: IValue> {
    pub components: Vec<Box<dyn CircuitEval<Value>>>,
    pub packed_public_data: Simd,
    pub public_data: PublicData<Var>,
    pub program: Arc<[[M31; MEMORY_VALUES_LIMBS]]>,
    pub packed_outputs: Simd,
    pub preprocessed_root: HashValue<QM31>,
    pub preprocessed_trace_variant: PreProcessedTraceVariant,
}

impl<Value: IValue> CairoStatement<Value> {
    /// Verifies the builtins.
    ///
    /// Assumes that the start and end addresses of the segment ranges are less than 2^27 (this is
    /// guaranteed by `segment_ranges_logup_sum`).
    pub fn verify_builtins(&self, context: &mut Context<Value>, component_sizes: &[Var]) {
        let [output_segment_range, builtin_segment_ranges @ ..] =
            &self.public_data.public_memory.segment_ranges;

        // Validate the output segment range.
        let diff =
            eval!(context, (output_segment_range.end.value) - (output_segment_range.start.value));
        let n_outputs = context.constant((self.packed_outputs.len() / MEMORY_VALUES_LIMBS).into());
        eq(context, diff, n_outputs);

        let [
            pedersen_segment_range,
            range_check_128_segment_range,
            ecdsa_segment_range,
            bitwise_segment_range,
            ec_op_segment_range,
            keccak_segment_range,
            poseidon_segment_range,
            range_check96_segment_range,
            add_mod_segment_range,
            mul_mod_segment_range,
        ] = builtin_segment_ranges;

        // Each builtin is either supported (Some with component name + instance size) or not
        // supported (None, meaning its segment must be empty).
        let all_builtins: [_; N_SEGMENTS - 1] = [
            (
                pedersen_segment_range,
                Some(("pedersen_builtin_narrow_windows", PEDERSEN_BUILTIN_MEMORY_CELLS)),
            ),
            (
                range_check_128_segment_range,
                Some(("range_check_builtin", RANGE_CHECK_BUILTIN_MEMORY_CELLS)),
            ),
            (ecdsa_segment_range, None),
            (bitwise_segment_range, Some(("bitwise_builtin", BITWISE_BUILTIN_MEMORY_CELLS))),
            (ec_op_segment_range, None),
            (keccak_segment_range, None),
            (poseidon_segment_range, Some(("poseidon_builtin", POSEIDON_BUILTIN_MEMORY_CELLS))),
            (range_check96_segment_range, None),
            (add_mod_segment_range, None),
            (mul_mod_segment_range, None),
        ];

        // Enforce not-supported builtins have empty segments, collect supported builtins.
        let mut supported_builtins: Vec<(&SegmentRange<Var>, &str, usize)> = vec![];
        for (segment_range, maybe_builtin) in all_builtins {
            match maybe_builtin {
                Some((name, size)) => supported_builtins.push((segment_range, name, size)),
                None => {
                    eq(context, segment_range.end.value, segment_range.start.value);
                }
            }
        }

        // Compute n_uses for all supported builtins.
        let start_addresses = Simd::pack(
            context,
            &supported_builtins
                .iter()
                .map(|(segment_range, _, _)| M31Wrapper::new_unsafe(segment_range.start.value))
                .collect_vec(),
        );
        let end_addresses = Simd::pack(
            context,
            &supported_builtins
                .iter()
                .map(|(segment_range, _, _)| M31Wrapper::new_unsafe(segment_range.end.value))
                .collect_vec(),
        );
        let diff = Simd::sub(context, &end_addresses, &start_addresses);

        let builtin_instance_size_inverses = pack_into_qm31s(
            supported_builtins.iter().map(|(_, _, size)| M31::from(*size as u32).inverse()),
        )
        .into_iter()
        .map(|qm31| context.constant(qm31))
        .collect();
        let packed_instance_size_inverses =
            Simd::from_packed(builtin_instance_size_inverses, supported_builtins.len());

        let n_uses_simd = Simd::mul(context, &diff, &packed_instance_size_inverses);

        // Range-check the number of times each builtin is used.
        // n_uses = (end - start) / instance_size, which implies end = start + n_uses *
        // instance_size (mod M31_P). Since all values are less than 2^27, this equality
        // also holds over the integers.
        extract_bits(context, &n_uses_simd, SMALL_VALUE_BITS);
        let max_instance_size = *supported_builtins.iter().map(|(_, _, size)| size).max().unwrap();
        assert!(
            (max_instance_size as u32).ilog2() < (31 - SMALL_VALUE_BITS),
            "max_builtin_memory_cell * n_uses might exceed M31_P"
        );

        let actual_uses_iter = Simd::unpack(context, &n_uses_simd).into_iter();
        let mut range_checks = vec![];
        let all_components = all_components::<Value>();

        for ((_, name, _), actual_uses) in zip_eq(&supported_builtins, actual_uses_iter) {
            let index = all_components.get_index_of(*name).unwrap();
            if self.components[index].is_disabled() {
                // Component is disabled - actual_uses must be 0.
                eq(context, actual_uses, context.zero());
            }

            let component_size = component_sizes[index];
            // Check that 0 <= component_size - actual_uses < 2^27 => actual_uses <= component_size.
            let diff = eval!(context, (component_size) - (actual_uses));
            range_checks.push(M31Wrapper::new_unsafe(diff));
        }

        let rc_simd = Simd::pack(context, &range_checks);
        extract_bits(context, &rc_simd, SMALL_VALUE_BITS);
    }
}

impl<Value: IValue> CairoStatement<Value> {
    pub fn new(
        context: &mut Context<Value>,
        public_data: Vec<M31>,
        outputs: Vec<[M31; MEMORY_VALUES_LIMBS]>,
        program: Arc<[[M31; MEMORY_VALUES_LIMBS]]>,
        preprocessed_root: HashValue<QM31>,
        preprocessed_trace_variant: PreProcessedTraceVariant,
    ) -> Self {
        let components = all_components().into_values().collect_vec();
        Self::new_ex(
            context,
            public_data,
            outputs,
            program,
            components,
            preprocessed_root,
            preprocessed_trace_variant,
        )
    }

    pub fn new_ex(
        context: &mut Context<Value>,
        public_data: Vec<M31>,
        outputs: Vec<[M31; MEMORY_VALUES_LIMBS]>,
        program: Arc<[[M31; MEMORY_VALUES_LIMBS]]>,
        components: Vec<Box<dyn CircuitEval<Value>>>,
        preprocessed_root: HashValue<QM31>,
        preprocessed_trace_variant: PreProcessedTraceVariant,
    ) -> Self {
        let packed_public_data = pack_into_qm31s(public_data.iter().cloned())
            .into_iter()
            .map(|qm31| Value::from_qm31(qm31).guess(context))
            .collect_vec();

        let packed_public_data = Simd::from_packed(packed_public_data, public_data.len());
        // Note that we don't enforce anything on the padding M31 in packed_public_data.
        let unpacked_simd = Simd::unpack(context, &packed_public_data);

        let public_data =
            PublicData::<Var>::parse_from_vars(&unpacked_simd[..], outputs.len(), program.len());

        let n_outputs = outputs.len();
        let packed_outputs = pack_into_qm31s(outputs.into_iter().flatten())
            .into_iter()
            .map(|qm31| Value::from_qm31(qm31).guess(context))
            .collect_vec();
        let packed_outputs = Simd::from_packed(packed_outputs, n_outputs * MEMORY_VALUES_LIMBS);

        Self {
            packed_public_data,
            public_data,
            program,
            packed_outputs,
            components,
            preprocessed_root,
            preprocessed_trace_variant,
        }
    }
}

impl<Value: IValue> Statement<Value> for CairoStatement<Value> {
    fn get_components(&self) -> &[Box<dyn CircuitEval<Value>>] {
        &self.components
    }

    fn claims_to_mix(&self, context: &mut Context<Value>) -> Vec<Vec<Var>> {
        let Self {
            components: _components,
            packed_public_data,
            public_data: _public_data,
            program,
            packed_outputs,
            preprocessed_root: _preprocessed_root,
            preprocessed_trace_variant: _preprocessed_trace_variant,
        } = self;
        let program_len = context.constant(qm31_from_u32s(program.len() as u32, 0, 0, 0));

        let output_hash = blake(context, packed_outputs.get_packed(), 4 * packed_outputs.len());

        // output the output hash.
        output(context, output_hash.0);
        output(context, output_hash.1);

        let flat_program = pack_into_qm31s(program.iter().flatten().cloned());
        let program_hash = IValue::blake(&flat_program, flat_program.len() * 16);
        vec![
            vec![program_len],
            packed_public_data.get_packed().to_vec(),
            vec![output_hash.0, output_hash.1],
            vec![context.constant(program_hash.0), context.constant(program_hash.1)],
        ]
    }

    fn public_logup_sum(
        &self,
        context: &mut Context<Value>,
        interaction_elements: [Var; 2],
    ) -> Var {
        let program_as_constants = self
            .program
            .iter()
            .map(|value_limbs| {
                value_limbs.map(|limb| M31Wrapper::new_unsafe(context.constant(limb.into())))
            })
            .collect_vec();

        let unpacked = Simd::unpack(context, &self.packed_outputs);
        let outputs: Vec<[M31Wrapper<Var>; MEMORY_VALUES_LIMBS]> = unpacked
            .chunks(MEMORY_VALUES_LIMBS)
            .map(|chunk| array::from_fn(|i| M31Wrapper::new_unsafe(chunk[i])))
            .collect_vec();

        public_logup_sum(
            context,
            &self.public_data,
            &program_as_constants,
            &outputs,
            interaction_elements,
        )
    }

    fn get_preprocessed_column_ids(&self) -> Vec<PreProcessedColumnId> {
        self.preprocessed_trace_variant.to_preprocessed_trace().ids()
    }

    fn public_params(&self, _context: &mut Context<Value>) -> HashMap<String, Var> {
        let segment_ranges = &self.public_data.public_memory.segment_ranges;
        let public_params: HashMap<String, Var> = HashMap::from_iter(
            [
                ("output_segment_start", &segment_ranges[0]),
                ("pedersen_builtin_segment_start", &segment_ranges[1]),
                ("range_check_builtin_segment_start", &segment_ranges[2]),
                ("ecdsa_builtin_segment_start", &segment_ranges[3]),
                ("bitwise_builtin_segment_start", &segment_ranges[4]),
                ("ec_op_builtin_segment_start", &segment_ranges[5]),
                ("keccak_builtin_segment_start", &segment_ranges[6]),
                ("poseidon_builtin_segment_start", &segment_ranges[7]),
                ("range_check96_builtin_segment_start", &segment_ranges[8]),
                ("add_mod_builtin_segment_start", &segment_ranges[9]),
                ("mul_mod_builtin_segment_start", &segment_ranges[10]),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.start.value)),
        );
        public_params
    }

    fn verify_claim(
        &self,
        context: &mut Context<Value>,
        component_sizes: &[Var],
        shifted_relation_uses: &HashMap<&'static str, Var>,
    ) {
        let PublicData { initial_state, final_state, public_memory: _ } = &self.public_data;

        self.verify_builtins(context, component_sizes);
        // TODO(ilya): Consider adding sanity checks on the content of the program segment.

        let CasmState { pc: initial_pc, ap: initial_ap, fp: initial_fp } = initial_state;
        let CasmState { pc: final_pc, ap: final_ap, fp: final_fp } = final_state;

        // A vector of values that are going to be range checked to 29 bits.
        let mut range_checks = vec![];

        eq(context, *initial_pc, context.one());
        // Check that initial_pc (== 1) + 2 < initial_ap.
        // i.e. 3 < initial_ap < 2**29 + 4.
        // At this point we actually know that `initial_ap < 2**27` because we enforced it when
        // computing the safe_call logup sum.
        range_checks.push(eval!(context, (*initial_ap) - (context.constant(4.into()))));

        eq(context, *initial_fp, *final_fp);
        eq(context, *initial_fp, *initial_ap);
        let expected_final_pc = context.constant(5.into());
        eq(context, *final_pc, expected_final_pc);
        // Check that the initial_ap <= final_ap.
        // i.e. 0 <= final_ap - initial_ap < 2**29.
        range_checks.push(eval!(context, (*final_ap) - (*initial_ap)));

        let rc_simd = Simd::pack(
            context,
            &range_checks.iter().map(|value| M31Wrapper::new_unsafe(*value)).collect_vec(),
        );
        extract_bits(context, &rc_simd, 29);

        // Sanity check: ensure that the maximum address in the address_to_id component fits within
        // a 29-bit address space (i.e., is less than 2**29).
        // Higher addresses are not supported by components that assume 29-bit addresses.
        // Assumes that there is only one ADDRESS_TO_ID component and it uses Seq.
        const { assert!(MEMORY_ADDRESS_TO_ID_SPLIT * MAX_SEQUENCE_LOG_SIZE < 1 << 29) };

        let shifted_opcode_relation_uses =
            Simd::from_packed(vec![shifted_relation_uses["Opcodes"]], 1);
        extract_bits(
            context,
            &shifted_opcode_relation_uses,
            (29 - RELATION_USES_NUM_ROWS_SHIFT).try_into().unwrap(),
        );
    }

    fn get_preprocessed_root(&self, context: &mut Context<Value>) -> HashValue<Var> {
        HashValue(
            context.constant(self.preprocessed_root.0),
            context.constant(self.preprocessed_root.1),
        )
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

pub fn segment_ranges_logup_sum(
    context: &mut Context<impl IValue>,
    interaction_elements: [Var; 2],
    segment_ranges: &[SegmentRange<Var>; N_SEGMENTS],
    mut argument_address: Var,
    mut return_value_address: Var,
) -> Var {
    let one = context.one();
    let mut sum = context.zero();
    for (i, segment_range) in segment_ranges.iter().enumerate() {
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

pub fn memory_segment_logup_sum(
    context: &mut Context<impl IValue>,
    interaction_elements: [Var; 2],
    start_address: Var,
    ids: &[Var],
    memory_values: &[[M31Wrapper<Var>; MEMORY_VALUES_LIMBS]],
) -> Var {
    let one = context.one();
    let mut sum = context.zero();

    let mut address = start_address;
    for (i, (&id, value_limbs)) in zip_eq(ids, memory_values).enumerate() {
        if i != 0 {
            address = eval!(context, (address) + (one));
        }

        let address_to_id_logup_term =
            address_to_id_logup_term(context, address, id, interaction_elements);
        sum = eval!(context, (sum) + (address_to_id_logup_term));

        let id_to_value_logup_term = id_to_big_logup_term(
            context,
            id,
            value_limbs.iter().map(|limb| *limb.get()),
            interaction_elements,
        );
        sum = eval!(context, (sum) + (id_to_value_logup_term));
    }

    sum
}

pub fn public_logup_sum(
    context: &mut Context<impl IValue>,
    public_data: &PublicData<Var>,
    program: &[[M31Wrapper<Var>; MEMORY_VALUES_LIMBS]],
    outputs: &[[M31Wrapper<Var>; MEMORY_VALUES_LIMBS]],
    interaction_elements: [Var; 2],
) -> Var {
    let PublicData {
        initial_state,
        final_state,
        public_memory: PublicMemory { segment_ranges, safe_call_ids, output_ids, program_ids },
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

    // Enforce correct initialization of the safe call memory section:
    // memory[initial_ap - 2] = (safe_call_id0, initial_ap)
    // memory[initial_ap - 1] = (safe_call_id1, 0).
    let split_initial_ap = split_27bit_to_9bit_limbs(context, initial_ap);
    // The value of memory[initial_ap - 1] is 0, so its 9-bit limbs are all zeros.
    // Passing an empty slice to id_to_big_logup_term is equivalent to passing [0, 0, 0]
    // because trailing zeros don't affect the polynomial combination in combine_term.
    let safe_call_values = [split_initial_ap.as_slice(), &[]];
    for (address, id, value_limbs) in izip!(safe_call_addresses, safe_call_ids, safe_call_values) {
        let logup_term =
            safe_call_id_logup_term(context, interaction_elements, address, *id, value_limbs);
        sum = eval!(context, (sum) + (logup_term));
    }

    let argument_address = initial_ap;
    let return_value_address =
        eval!(context, (final_ap) - (context.constant(QM31::from(N_SEGMENTS as u32))));
    let segment_ranges_logup_sum = segment_ranges_logup_sum(
        context,
        interaction_elements,
        segment_ranges,
        argument_address,
        return_value_address,
    );
    sum = eval!(context, (sum) + (segment_ranges_logup_sum));

    let output_logup_sum = memory_segment_logup_sum(
        context,
        interaction_elements,
        segment_ranges[0].start.value,
        output_ids,
        outputs,
    );
    sum = eval!(context, (sum) + (output_logup_sum));

    let program_logup_sum = memory_segment_logup_sum(
        context,
        interaction_elements,
        initial_state.pc,
        program_ids,
        program,
    );
    sum = eval!(context, (sum) + (program_logup_sum));

    sum
}
