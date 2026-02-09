use crate::circuit_prover::witness::components::qm31_ops;
use crate::circuit_prover::witness::trace::TraceGenerator;
use crate::circuits::circuit::{Blake, Circuit, Permutation};
use crate::circuits::circuit::{Eq, Gate};
use itertools::{Itertools, chain, zip_eq};
use std::collections::HashMap;
use stwo::core::fields::m31::BaseField;
use stwo::core::poly::circle::CanonicCoset;
use stwo::prover::backend::Backend;
use stwo::prover::backend::Col;
use stwo::prover::backend::Column;
use stwo::prover::backend::simd::m31::{N_LANES, PackedM31};
use stwo::prover::poly::BitReversedOrder;
use stwo::prover::poly::circle::CircleEvaluation;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

#[cfg(test)]
#[path = "preprocessed_test.rs"]
pub mod test;

const N_QM31_OPS_PP_COLUMNS: usize = 8;
const N_EQ_PP_COLUMNS: usize = 2;
const N_OP_CODES: usize = 4;

#[derive(Copy, Clone)]
enum OpCode {
    Add,
    Sub,
    Mul,
    PointwiseMul,
}

fn vec_to_evaluation<B: Backend>(
    vec: Vec<usize>,
) -> CircleEvaluation<B, BaseField, BitReversedOrder> {
    let col = Col::<B, BaseField>::from_iter(vec.into_iter().map(BaseField::from));
    CircleEvaluation::new(CanonicCoset::new(col.len().ilog2()).circle_domain(), col)
}

/// Adds the binary operation gates to the qm31 ops preprocessed trace.
fn fill_binary_op_columns<G: Gate>(
    gates: &[G],
    op_code: OpCode,
    multiplicities: &[usize],
    columns: &mut [Vec<usize>; N_QM31_OPS_PP_COLUMNS],
) {
    let op_code_idx = op_code as usize;
    for gate in gates.iter() {
        let [in0, in1] = gate.uses()[..] else { panic!("Expected 2 uses for gate") };
        let [out] = gate.yields()[..] else { panic!("Expected 1 yield for gate") };
        (0..N_OP_CODES).for_each(|i| {
            columns[i].push(if i == op_code_idx { 1 } else { 0 });
        });
        columns[4].push(in0);
        columns[5].push(in1);
        columns[6].push(out);
        // TODO(Gali): Consider negating the multiplicities.
        columns[7].push(multiplicities[out]);
    }
}

/// Implements a permutation gate with n inputs and n outputs using 2n Add gates.
///
/// Process:
/// 1. First n gates: Write inputs to permutation wire
///    - `permutation_wire = Add(0, input_i)` for each input i
/// 2. Next n gates: Read outputs from permutation wire
///    - `output_i = Add(0, permutation_wire)` for each output i
///
/// Using the same wire address for all the inputs with multiplicity 1 ensures that the outputs
/// are a permutation of the inputs.
fn fill_permutation_columns(
    gates: &[Permutation],
    multiplicities: &[usize],
    columns: &mut [Vec<usize>; N_QM31_OPS_PP_COLUMNS],
    first_unused_address: usize,
) {
    let add_op_code_idx = OpCode::Add as usize;
    let mut permutation_address = first_unused_address;
    for gate in gates.iter() {
        let inputs = gate.uses();
        let outputs = gate.yields();

        // Set flag to Add opcode.
        (0..N_OP_CODES).for_each(|i| {
            columns[i].extend(std::iter::repeat_n(
                (i == add_op_code_idx) as usize,
                inputs.len() + outputs.len(),
            ));
        });

        // TODO(alonf): Parallelize, and insert the above loop inside.
        for (input, output) in zip_eq(inputs, outputs) {
            // Input row.
            columns[4].push(0);
            columns[5].push(input);
            columns[6].push(permutation_address);
            columns[7].push(1);

            // Output row.
            columns[4].push(0);
            columns[5].push(permutation_address);
            columns[6].push(output);
            columns[7].push(multiplicities[output]);
        }

        permutation_address += 1;
    }
}

/// Adds the eq gates to the eq preprocessed trace.
fn fill_eq_columns(eq_gates: &[Eq], columns: &mut [Vec<usize>; N_EQ_PP_COLUMNS]) {
    for gate in eq_gates.iter() {
        let [in0, in1] = gate.uses()[..] else { panic!("Expected 2 uses for gate") };
        assert!(gate.yields().is_empty(), "Expected no yields for Eq gate");
        columns[0].push(in0);
        columns[1].push(in1);
    }
}

/// Adds the preprocessed columns of qm31_ops component to the preprocessed trace. If the component
/// is empty, no columns are added. Preprocessed columns are in the following format:
/// | add_flag | sub_flag | mul_flag | pointwise_mul_flag | in0_address | in1_address | out_address | mults |
fn add_qm31_ops_to_preprocessed_trace(
    circuit: &Circuit,
    multiplicities: &[usize],
    pp_trace: &mut PreProcessedTrace,
) -> qm31_ops::TraceGenerator {
    let Circuit { n_vars, add, sub, mul, pointwise_mul, eq: _, blake: _, permutation } = circuit;
    let mut qm31_ops_columns: [_; N_QM31_OPS_PP_COLUMNS] = std::array::from_fn(|_| vec![]);
    fill_binary_op_columns(add, OpCode::Add, &multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(sub, OpCode::Sub, &multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(mul, OpCode::Mul, &multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(
        pointwise_mul,
        OpCode::PointwiseMul,
        &multiplicities,
        &mut qm31_ops_columns,
    );
    let qm31_ops_trace_generator =
        qm31_ops::TraceGenerator { first_permutation_row: qm31_ops_columns[0].len() };

    fill_permutation_columns(permutation, &multiplicities, &mut qm31_ops_columns, *n_vars);

    let n_columns = pp_trace.columns.len();
    pp_trace.column_indices.extend([
        (PreProcessedColumnId { id: "qm31_ops_add_flag".to_owned() }, n_columns),
        (PreProcessedColumnId { id: "qm31_ops_sub_flag".to_owned() }, n_columns + 1),
        (PreProcessedColumnId { id: "qm31_ops_mul_flag".to_owned() }, n_columns + 2),
        (PreProcessedColumnId { id: "qm31_ops_pointwise_mul_flag".to_owned() }, n_columns + 3),
        (PreProcessedColumnId { id: "qm31_ops_in0_address".to_owned() }, n_columns + 4),
        (PreProcessedColumnId { id: "qm31_ops_in1_address".to_owned() }, n_columns + 5),
        (PreProcessedColumnId { id: "qm31_ops_out_address".to_owned() }, n_columns + 6),
        (PreProcessedColumnId { id: "qm31_ops_mults".to_owned() }, n_columns + 7),
    ]);
    pp_trace.columns.extend(qm31_ops_columns);
    qm31_ops_trace_generator
}

/// Adds the preprocessed columns of eq component to the preprocessed trace. If the component
/// is empty, no columns are added. Preprocessed columns are in the following format:
/// | in0_address | in1_address |
fn add_eq_to_preprocessed_trace(circuit: &Circuit, pp_trace: &mut PreProcessedTrace) {
    let Circuit {
        n_vars: _,
        add: _,
        sub: _,
        mul: _,
        pointwise_mul: _,
        eq,
        blake: _,
        permutation: _,
    } = circuit;
    let mut eq_columns: [_; N_EQ_PP_COLUMNS] = std::array::from_fn(|_| vec![]);
    fill_eq_columns(eq, &mut eq_columns);

    let n_columns = pp_trace.columns.len();
    pp_trace.column_indices.extend([
        (PreProcessedColumnId { id: "eq_in0_address".to_owned() }, n_columns),
        (PreProcessedColumnId { id: "eq_in1_address".to_owned() }, n_columns + 1),
    ]);
    pp_trace.columns.extend(eq_columns);
}

/// Currently fills 9 columns.
fn fill_blake_columns(
    blake: &[Blake],
    multiplicities: &[usize],
    columns: &mut [Vec<usize>; N_BLAKE_PP_COLUMNS],
) {
    // IV should somehow be in state_address 0.
    let mut state_address = 0;
    let mut message_length = 0;
    for gate in blake.iter() {
        for [in0, in1, in2, in3] in gate.input.iter() {
            // The current message length split to 2 u16.
            message_length = gate.n_bytes.min(message_length + 16 * 4);
            columns[0].push(message_length & 0xffff);
            columns[1].push((message_length >> 16) & 0xffff);
            // Finalize flag.
            columns[2].push(0);
            // State before address.
            columns[3].push(state_address);
            eprintln!("{state_address}");
            // State after address.
            state_address += 1;
            columns[4].push(state_address);
            eprintln!("{state_address}");
            // Message addresses.
            columns[5].push(*in0);
            columns[6].push(*in1);
            columns[7].push(*in2);
            columns[8].push(*in3);

            // Enable
            columns[9].push(1);
        }
        // Set the finalize flag to 1 for the last compression of the gate.
        *columns[2].last_mut().unwrap() = 1;
        // Fill the preprocessed column needed by the blake_output component.
        // Set final state address.
        columns[10].push(state_address);
        eprintln!("{state_address}");

        let [out0, out1] = gate.yields()[..] else { panic!("Expected 2 yields for gate") };
        columns[11].push(out0);
        columns[12].push(out1);
        columns[13].push(multiplicities[out0]);
        columns[14].push(multiplicities[out1]);
    }
    // Pad the preprocessed columns used in blake compress.
    let n_blake_compress = columns[0].len();
    let blake_compress_padding = std::cmp::max(n_blake_compress.next_power_of_two(), N_LANES);
    // Pad with the first element.
    (0..9).for_each(|i| columns[i].resize(blake_compress_padding, *columns[i].first().unwrap()));
    columns[9].resize(blake_compress_padding, 0); // Enabler columns.

    // Pad the preprocessed columns used in blake output
    let n_blake_output = columns[10].len();
    let blake_output_padding = std::cmp::max(n_blake_output.next_power_of_two(), N_LANES);
    (10..13).for_each(|i| columns[i].resize(blake_output_padding, *columns[i].first().unwrap()));
    (13..15).for_each(|i| columns[i].resize(blake_output_padding, 0)); // Multiplicity columns.
}

const BLAKE2S_SIGMA: [[usize; 16]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
];

/// Generates 16 columns of size 16 (LOG_SIZE=4) for the blake round sigma permutation table.
/// Column i contains BLAKE2S_SIGMA[round][i] for rounds 0..9, padded with round 0 values for
/// rows 10..15.
fn gen_blake_sigma_columns() -> [Vec<usize>; 16] {
    std::array::from_fn(|i| {
        let mut col = Vec::with_capacity(16);
        for round in 0..10 {
            col.push(BLAKE2S_SIGMA[round][i]);
        }
        // Pad rows 10..15 with round 0 values.
        for _ in 10..16 {
            col.push(BLAKE2S_SIGMA[0][i]);
        }
        col
    })
}

const N_BLAKE_PP_COLUMNS: usize = 9 + 1 + 5;

fn add_blake_to_preprocessed_trace(
    circuit: &Circuit,
    multiplicities: &[usize],
    pp_trace: &mut PreProcessedTrace,
) {
    let Circuit {
        n_vars: _,
        add: _,
        sub: _,
        mul: _,
        pointwise_mul: _,
        eq: _,
        blake,
        permutation: _,
    } = circuit;
    let mut blake_columns: [_; N_BLAKE_PP_COLUMNS] = std::array::from_fn(|_| vec![]);
    fill_blake_columns(blake, multiplicities, &mut blake_columns);

    let n_columns = pp_trace.columns.len();
    pp_trace.column_indices.extend([
        (PreProcessedColumnId { id: "t0".to_owned() }, n_columns),
        (PreProcessedColumnId { id: "t1".to_owned() }, n_columns + 1),
        (PreProcessedColumnId { id: "finalize_flag".to_owned() }, n_columns + 2),
        (PreProcessedColumnId { id: "state_before_addr".to_owned() }, n_columns + 3),
        (PreProcessedColumnId { id: "state_after_addr".to_owned() }, n_columns + 4),
        (PreProcessedColumnId { id: "message0_addr".to_owned() }, n_columns + 5),
        (PreProcessedColumnId { id: "message1_addr".to_owned() }, n_columns + 6),
        (PreProcessedColumnId { id: "message2_addr".to_owned() }, n_columns + 7),
        (PreProcessedColumnId { id: "message3_addr".to_owned() }, n_columns + 8),
        (PreProcessedColumnId { id: "compress_enabler".to_owned() }, n_columns + 9),
        (PreProcessedColumnId { id: "final_state_addr".to_owned() }, n_columns + 10),
        (PreProcessedColumnId { id: "blake_output0_addr".to_owned() }, n_columns + 11),
        (PreProcessedColumnId { id: "blake_output1_addr".to_owned() }, n_columns + 12),
        (PreProcessedColumnId { id: "blake_output0_mults".to_owned() }, n_columns + 13),
        (PreProcessedColumnId { id: "blake_output1_mults".to_owned() }, n_columns + 14),
    ]);
    pp_trace.columns.extend(blake_columns);

    // Add blake sigma columns (16 columns of 16 rows each).
    let blake_sigma = gen_blake_sigma_columns();
    let n_columns = pp_trace.columns.len();
    pp_trace.column_indices.extend([
        (PreProcessedColumnId { id: "blake_sigma_0".to_owned() }, n_columns),
        (PreProcessedColumnId { id: "blake_sigma_1".to_owned() }, n_columns + 1),
        (PreProcessedColumnId { id: "blake_sigma_2".to_owned() }, n_columns + 2),
        (PreProcessedColumnId { id: "blake_sigma_3".to_owned() }, n_columns + 3),
        (PreProcessedColumnId { id: "blake_sigma_4".to_owned() }, n_columns + 4),
        (PreProcessedColumnId { id: "blake_sigma_5".to_owned() }, n_columns + 5),
        (PreProcessedColumnId { id: "blake_sigma_6".to_owned() }, n_columns + 6),
        (PreProcessedColumnId { id: "blake_sigma_7".to_owned() }, n_columns + 7),
        (PreProcessedColumnId { id: "blake_sigma_8".to_owned() }, n_columns + 8),
        (PreProcessedColumnId { id: "blake_sigma_9".to_owned() }, n_columns + 9),
        (PreProcessedColumnId { id: "blake_sigma_10".to_owned() }, n_columns + 10),
        (PreProcessedColumnId { id: "blake_sigma_11".to_owned() }, n_columns + 11),
        (PreProcessedColumnId { id: "blake_sigma_12".to_owned() }, n_columns + 12),
        (PreProcessedColumnId { id: "blake_sigma_13".to_owned() }, n_columns + 13),
        (PreProcessedColumnId { id: "blake_sigma_14".to_owned() }, n_columns + 14),
        (PreProcessedColumnId { id: "blake_sigma_15".to_owned() }, n_columns + 15),
    ]);
    pp_trace.columns.extend(blake_sigma);
}

/// A collection of preprocessed columns, whose values are publicly acknowledged, and independent of
/// the proof.
pub struct PreProcessedTrace {
    pub columns: Vec<Vec<usize>>,
    column_indices: HashMap<PreProcessedColumnId, usize>,
}

impl PreProcessedTrace {
    /// Generates the preprocessed trace for the circuit, assuming it is already finalized.
    pub fn generate_preprocessed_trace(circuit: &Circuit) -> (Self, TraceGenerator) {
        let mut pp_trace = Self { columns: vec![], column_indices: HashMap::new() };

        // Adjust multiplicities to account for the use of the constant 0 in the permutation gate
        // implementation. See `fill_permutation_columns` for details.
        let mut multiplicities = circuit.compute_multiplicities().0;
        let additional_zero_multiplicity: usize =
            circuit.permutation.iter().map(|gate| gate.inputs.len() + gate.outputs.len()).sum();
        multiplicities[0] += additional_zero_multiplicity;

        // Add Eq columns.
        add_eq_to_preprocessed_trace(circuit, &mut pp_trace);
        // Add QM31 operations columns.
        let qm31_ops_trace_generator =
            add_qm31_ops_to_preprocessed_trace(circuit, &multiplicities, &mut pp_trace);

        // TODO(Gali): Add Blake columns.
        add_blake_to_preprocessed_trace(circuit, &multiplicities, &mut pp_trace);

        Self::add_non_circuit_preprocessed_columns(&mut pp_trace);

        (pp_trace, TraceGenerator { qm31_ops_trace_generator })
    }

    fn add_non_circuit_preprocessed_columns(pp_trace: &mut PreProcessedTrace) {
        let n_columns = pp_trace.columns.len();
        let seq: [Vec<usize>; 17] = std::array::from_fn(|i| (0..1_usize << (i + 4)).collect());
        let bitwise_xor: Vec<Vec<usize>> = [4, 7, 8, 9, 10]
            .into_iter()
            .flat_map(|n_bits| gen_xor_columns(n_bits).into_iter())
            .collect();
        let blake_sigma = gen_blake_sigma_columns();

        pp_trace.columns.extend(chain!(seq, bitwise_xor, blake_sigma));
        pp_trace.column_indices.extend([
            (PreProcessedColumnId { id: "seq_4".to_owned() }, n_columns),
            (PreProcessedColumnId { id: "seq_5".to_owned() }, n_columns + 1),
            (PreProcessedColumnId { id: "seq_6".to_owned() }, n_columns + 2),
            (PreProcessedColumnId { id: "seq_7".to_owned() }, n_columns + 3),
            (PreProcessedColumnId { id: "seq_8".to_owned() }, n_columns + 4),
            (PreProcessedColumnId { id: "seq_9".to_owned() }, n_columns + 5),
            (PreProcessedColumnId { id: "seq_10".to_owned() }, n_columns + 6),
            (PreProcessedColumnId { id: "seq_11".to_owned() }, n_columns + 7),
            (PreProcessedColumnId { id: "seq_12".to_owned() }, n_columns + 8),
            (PreProcessedColumnId { id: "seq_13".to_owned() }, n_columns + 9),
            (PreProcessedColumnId { id: "seq_14".to_owned() }, n_columns + 10),
            (PreProcessedColumnId { id: "seq_15".to_owned() }, n_columns + 11),
            (PreProcessedColumnId { id: "seq_16".to_owned() }, n_columns + 12),
            (PreProcessedColumnId { id: "seq_17".to_owned() }, n_columns + 13),
            (PreProcessedColumnId { id: "seq_18".to_owned() }, n_columns + 14),
            (PreProcessedColumnId { id: "seq_19".to_owned() }, n_columns + 15),
            (PreProcessedColumnId { id: "seq_20".to_owned() }, n_columns + 16),
            // bitwise_xor columns start after 17 seq columns
            (PreProcessedColumnId { id: "bitwise_xor_4_0".to_owned() }, n_columns + 17),
            (PreProcessedColumnId { id: "bitwise_xor_4_1".to_owned() }, n_columns + 18),
            (PreProcessedColumnId { id: "bitwise_xor_4_2".to_owned() }, n_columns + 19),
            (PreProcessedColumnId { id: "bitwise_xor_7_0".to_owned() }, n_columns + 20),
            (PreProcessedColumnId { id: "bitwise_xor_7_1".to_owned() }, n_columns + 21),
            (PreProcessedColumnId { id: "bitwise_xor_7_2".to_owned() }, n_columns + 22),
            (PreProcessedColumnId { id: "bitwise_xor_8_0".to_owned() }, n_columns + 23),
            (PreProcessedColumnId { id: "bitwise_xor_8_1".to_owned() }, n_columns + 24),
            (PreProcessedColumnId { id: "bitwise_xor_8_2".to_owned() }, n_columns + 25),
            (PreProcessedColumnId { id: "bitwise_xor_9_0".to_owned() }, n_columns + 26),
            (PreProcessedColumnId { id: "bitwise_xor_9_1".to_owned() }, n_columns + 27),
            (PreProcessedColumnId { id: "bitwise_xor_9_2".to_owned() }, n_columns + 28),
            (PreProcessedColumnId { id: "bitwise_xor_10_0".to_owned() }, n_columns + 29),
            (PreProcessedColumnId { id: "bitwise_xor_10_1".to_owned() }, n_columns + 30),
            (PreProcessedColumnId { id: "bitwise_xor_10_2".to_owned() }, n_columns + 31),
        ]);
    }

    pub fn log_sizes(&self) -> Vec<u32> {
        self.columns.iter().map(|c| c.len().ilog2()).collect()
    }

    pub fn ids(&self) -> Vec<PreProcessedColumnId> {
        self.column_indices.keys().cloned().sorted_by_key(|k| self.column_indices[k]).collect()
    }

    pub fn get_trace<B: Backend>(&self) -> Vec<CircleEvaluation<B, BaseField, BitReversedOrder>> {
        self.columns.iter().map(|c| vec_to_evaluation::<B>(c.clone())).collect()
    }

    pub fn get_column(&self, id: &PreProcessedColumnId) -> &Vec<usize> {
        &self.columns[*self
            .column_indices
            .get(id)
            .unwrap_or_else(|| panic!("Missing preprocessed column {id:?}"))]
    }

    pub fn get_packed_column(&self, id: &PreProcessedColumnId) -> Vec<PackedM31> {
        let column = self.get_column(id);
        column
            .chunks_exact(N_LANES)
            .map(|c| PackedM31::from_array(std::array::from_fn(|i| BaseField::from(c[i]))))
            .collect::<Vec<_>>()
    }
}

// Generates 3 columns of size 2^(2*n_bits) where the third column is the XOR of the first two.
fn gen_xor_columns(n_bits: usize) -> [Vec<usize>; 3] {
    let size = 1_usize << (2 * n_bits);
    let mask = (1_usize << n_bits) - 1;
    let mut columns: [Vec<usize>; 3] = std::array::from_fn(|_| vec![0; size]);
    for i in 0..size {
        let lhs = i & mask;
        let rhs = i >> n_bits;
        columns[0][i] = lhs;
        columns[1][i] = rhs;
        columns[2][i] = lhs ^ rhs;
    }
    columns
}

// /// A table of a,b,c, where a,b,c are integers and a ^ b = c.
// ///
// /// # Attributes
// ///
// /// - `n_bits`: The number of bits in each integer.
// /// - `col_index`: The column index in the preprocessed table.
// #[derive(Debug)]
// pub struct BitwiseXor {
//     n_bits: u32,
//     col_index: usize,
// }
// impl BitwiseXor {
//     pub const fn new(n_bits: u32, col_index: usize) -> Self {
//         assert!(col_index < 3, "col_index must be in range 0..=2");
//         Self { n_bits, col_index }
//     }
// }

// impl PreProcessedColumn for BitwiseXor {
//     fn log_size(&self) -> u32 {
//         2 * self.n_bits
//     }

//     fn packed_at(&self, vec_row: usize) -> PackedM31 {
//         let lhs = || -> u32x16 {
//             (SIMD_ENUMERATION_0 + Simd::splat((vec_row * N_LANES) as u32)) >> self.n_bits
//         };
//         let rhs = || -> u32x16 {
//             (SIMD_ENUMERATION_0 + Simd::splat((vec_row * N_LANES) as u32))
//                 & Simd::splat((1 << self.n_bits) - 1)
//         };
//         let simd = match self.col_index {
//             0 => lhs(),
//             1 => rhs(),
//             2 => lhs() ^ rhs(),
//             _ => unreachable!(),
//         };
//         unsafe { PackedM31::from_simd_unchecked(simd) }
//     }

//     fn gen_column_simd(&self) -> CircleEvaluation<SimdBackend, BaseField, BitReversedOrder> {
//         CircleEvaluation::new(
//             CanonicCoset::new(self.log_size()).circle_domain(),
//             BaseColumn::from_simd(
//                 (0..(1 << (self.log_size() - LOG_N_LANES)))
//                     .map(|i| self.packed_at(i))
//                     .collect(),
//             ),
//         )
//     }

//     fn id(&self) -> PreProcessedColumnId {
//         PreProcessedColumnId {
//             id: format!("bitwise_xor_{}_{}", self.n_bits, self.col_index),
//         }
//     }
// }

// use std::simd::Simd;

// use stwo::core::fields::m31::M31;

// const N_LANES: usize = 4;

// // Pads all rows below <padding_offset> with the first row. Uses the <get_m31> function to get
// the // value in a given row and column.
// pub fn pad<F>(get_m31: F, padding_offset: usize, col: usize) -> Vec<M31>
// where
//     F: Fn(usize, usize) -> M31,
// {
//     let n = padding_offset.next_power_of_two();
//     (0..n)
//         .map(|i| if i < padding_offset { i } else { 0 })
//         .map(|i| get_m31(i, col))
//         .collect()
// }

// pub const SIMD_ENUMERATION_0: Simd<u32, N_LANES> =
//     Simd::from_array([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
