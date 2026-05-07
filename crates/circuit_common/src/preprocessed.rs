use crate::CircuitParams;
use crate::N_LANES;
use crate::Qm31OpsTraceGenerator;
use crate::finalize::finalize_context;
use crate::order_hash_map::OrderedHashMap;
use circuits::circuit::Blake;
use circuits::circuit::Gate;
use circuits::circuit::M31ToU32;
use circuits::circuit::{Circuit, Permutation};
use circuits::context::Context;
use circuits::ivalue::IValue;
use itertools::zip_eq;
use std::sync::Arc;
#[cfg(feature = "prover")]
use stwo::core::fields::m31::BaseField;
#[cfg(feature = "prover")]
use stwo::core::poly::circle::CanonicCoset;
#[cfg(feature = "prover")]
use stwo::prover::backend::simd::m31::PackedM31;
#[cfg(feature = "prover")]
use stwo::prover::backend::{Backend, Col, Column};
#[cfg(feature = "prover")]
use stwo::prover::poly::BitReversedOrder;
#[cfg(feature = "prover")]
use stwo::prover::poly::circle::CircleEvaluation;
pub use stwo_cairo_common::preprocessed_columns::blake::BLAKE_SIGMA;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

#[cfg(feature = "prover")]
#[cfg(test)]
#[path = "preprocessed_test.rs"]
pub mod test;

const N_QM31_OPS_PP_COLUMNS: usize = 8;
const N_OP_CODES: usize = 4;

#[derive(Copy, Clone)]
enum OpCode {
    Add,
    Sub,
    Mul,
    PointwiseMul,
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

/// Adds the preprocessed columns of qm31_ops component to the preprocessed trace. If the component
/// is empty, no columns are added. Preprocessed columns are in the following format:
/// | add_flag | sub_flag | mul_flag | pointwise_mul_flag | in0_address | in1_address | out_address | mults |
fn add_qm31_ops_to_preprocessed_trace(
    circuit: &Circuit,
    multiplicities: &[usize],
    pp_trace: &mut PreProcessedTrace,
) -> Qm31OpsTraceGenerator {
    let Circuit { n_vars, add, sub, mul, pointwise_mul, permutation, .. } = circuit;
    let mut qm31_ops_columns: [_; N_QM31_OPS_PP_COLUMNS] = std::array::from_fn(|_| vec![]);
    fill_binary_op_columns(add, OpCode::Add, multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(sub, OpCode::Sub, multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(mul, OpCode::Mul, multiplicities, &mut qm31_ops_columns);
    fill_binary_op_columns(
        pointwise_mul,
        OpCode::PointwiseMul,
        multiplicities,
        &mut qm31_ops_columns,
    );
    let qm31_ops_trace_generator =
        Qm31OpsTraceGenerator { first_permutation_row: qm31_ops_columns[0].len() };

    fill_permutation_columns(permutation, multiplicities, &mut qm31_ops_columns, *n_vars);

    let ids = [
        "qm31_ops_add_flag",
        "qm31_ops_sub_flag",
        "qm31_ops_mul_flag",
        "qm31_ops_pointwise_mul_flag",
        "qm31_ops_in0_address",
        "qm31_ops_in1_address",
        "qm31_ops_out_address",
        "qm31_ops_mults",
    ];
    for (id, column) in zip_eq(ids, qm31_ops_columns) {
        pp_trace.push_column(PreProcessedColumnId { id: id.to_owned() }, column);
    }
    qm31_ops_trace_generator
}

/// Adds the preprocessed columns of eq component to the preprocessed trace. If the component
/// is empty, no columns are added. Preprocessed columns are in the following format:
/// | in0_address | in1_address |
fn add_eq_to_preprocessed_trace(circuit: &Circuit, pp_trace: &mut PreProcessedTrace) {
    let Circuit { eq, .. } = circuit;
    let mut eq_in0_address = vec![];
    let mut eq_in1_address = vec![];
    for gate in eq.iter() {
        let [in0, in1] = gate.uses()[..] else { panic!("Expected 2 uses for gate") };
        assert!(gate.yields().is_empty(), "Expected no yields for Eq gate");
        eq_in0_address.push(in0);
        eq_in1_address.push(in1);
    }

    pp_trace.push_column(PreProcessedColumnId { id: "eq_in0_address".to_owned() }, eq_in0_address);
    pp_trace.push_column(PreProcessedColumnId { id: "eq_in1_address".to_owned() }, eq_in1_address);
}

// TODO(alonf): Parallelize.
fn add_blake_columns(blake: &[Blake], multiplicities: &[usize], pp_trace: &mut PreProcessedTrace) {
    let mut t0 = vec![];
    let mut t1 = vec![];
    let mut finalize_flag = vec![];
    let mut state_before_addr = vec![];
    let mut state_after_addr = vec![];
    let mut message0_addr = vec![];
    let mut message1_addr = vec![];
    let mut message2_addr = vec![];
    let mut message3_addr = vec![];
    let mut compress_enabler = vec![];
    let mut final_state_addr = vec![];
    let mut blake_output0_addr = vec![];
    let mut blake_output1_addr = vec![];
    let mut blake_output0_mults = vec![];
    let mut blake_output1_mults = vec![];

    // IV should be in state_address 0.
    let mut state_address = 1;
    for gate in blake.iter() {
        let mut message_length = 0;
        for (i, [in0, in1, in2, in3]) in gate.input.iter().enumerate() {
            // The current message length split to 2 u16.
            message_length = gate.n_bytes.min(message_length + 16 * 4);
            t0.push(message_length & 0xffff);
            t1.push((message_length >> 16) & 0xffff);

            // Finalize flag.
            finalize_flag.push(0);

            // State before and after addresses.
            let is_first_compression = i == 0;
            let state_address_before = if is_first_compression {
                // First compression starts from IV at address 0.
                0
            } else {
                state_address
            };
            state_before_addr.push(state_address_before);

            if !is_first_compression {
                state_address += 1;
            }
            state_after_addr.push(state_address);

            // Message addresses.
            message0_addr.push(*in0);
            message1_addr.push(*in1);
            message2_addr.push(*in2);
            message3_addr.push(*in3);

            // Enable
            compress_enabler.push(1);
        }

        // Set the finalize flag to 1 for the last compression of the gate.
        *finalize_flag.last_mut().unwrap() = 1;

        // Fill the preprocessed column needed by the blake_output component.
        // Set final state address.
        final_state_addr.push(state_address);

        let [out0, out1] = gate.yields()[..] else { panic!("Expected 2 yields for gate") };
        blake_output0_addr.push(out0);
        blake_output1_addr.push(out1);
        blake_output0_mults.push(multiplicities[out0]);
        blake_output1_mults.push(multiplicities[out1]);

        // Start a new blake chain.
        state_address += 1;
    }

    // Pad the preprocessed columns used in blake compress.
    let n_blake_compress = t0.len();
    let blake_compress_padding = std::cmp::max(n_blake_compress.next_power_of_two(), N_LANES);

    // TODO(Leo): remove after we remove the circuit gates padding.
    assert_eq!(
        n_blake_compress % N_LANES,
        0,
        "Only padding to multiple of N_LANES through circuit gates for now."
    );

    // Pad with the first element.
    for col in [
        &mut t0,
        &mut t1,
        &mut finalize_flag,
        &mut state_before_addr,
        &mut state_after_addr,
        &mut message0_addr,
        &mut message1_addr,
        &mut message2_addr,
        &mut message3_addr,
    ] {
        col.resize(blake_compress_padding, *col.first().unwrap());
    }
    compress_enabler.resize(blake_compress_padding, 0); // Enabler columns.

    // Pad the preprocessed columns used in blake output
    let n_blake_output = final_state_addr.len();
    let blake_output_padding = std::cmp::max(n_blake_output.next_power_of_two(), N_LANES);

    // Pad final_state_addr with zeros, so padding rows read the Blake initial state as the final
    // state
    final_state_addr.resize(blake_output_padding, 0);
    blake_output0_addr.resize(blake_output_padding, *blake_output0_addr.first().unwrap());
    blake_output1_addr.resize(blake_output_padding, *blake_output1_addr.first().unwrap());

    // Multiplicity columns.
    blake_output0_mults.resize(blake_output_padding, 0);
    blake_output1_mults.resize(blake_output_padding, 0);

    pp_trace.push_column(PreProcessedColumnId { id: "t0".to_owned() }, t0);
    pp_trace.push_column(PreProcessedColumnId { id: "t1".to_owned() }, t1);
    pp_trace.push_column(PreProcessedColumnId { id: "finalize_flag".to_owned() }, finalize_flag);
    pp_trace.push_column(
        PreProcessedColumnId { id: "state_before_addr".to_owned() },
        state_before_addr,
    );
    pp_trace
        .push_column(PreProcessedColumnId { id: "state_after_addr".to_owned() }, state_after_addr);
    pp_trace.push_column(PreProcessedColumnId { id: "message0_addr".to_owned() }, message0_addr);
    pp_trace.push_column(PreProcessedColumnId { id: "message1_addr".to_owned() }, message1_addr);
    pp_trace.push_column(PreProcessedColumnId { id: "message2_addr".to_owned() }, message2_addr);
    pp_trace.push_column(PreProcessedColumnId { id: "message3_addr".to_owned() }, message3_addr);
    pp_trace
        .push_column(PreProcessedColumnId { id: "compress_enabler".to_owned() }, compress_enabler);
    pp_trace
        .push_column(PreProcessedColumnId { id: "final_state_addr".to_owned() }, final_state_addr);
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_output0_addr".to_owned() },
        blake_output0_addr,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_output1_addr".to_owned() },
        blake_output1_addr,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_output0_mults".to_owned() },
        blake_output0_mults,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_output1_mults".to_owned() },
        blake_output1_mults,
    );
}

/// Builds the fixed preprocessed table for BLAKE sigma lookups.
///
/// Returns 16 columns (`blake_sigma_0..blake_sigma_15`), each with 16 rows (`LOG_SIZE = 4`).
/// For each column `i`:
/// - rows `0..10` hold `BLAKE_SIGMA[round][i]` for rounds `0..9`.
/// - rows `10..16` are padded with `BLAKE_SIGMA[0][i]`.
///
/// The padding keeps the table at a power-of-two height while preserving a valid sigma row.
fn gen_blake_sigma_columns() -> [Vec<usize>; 16] {
    std::array::from_fn(|i| {
        let mut col = Vec::with_capacity(16);
        for sigma_row in BLAKE_SIGMA.iter().take(10) {
            col.push(sigma_row[i] as usize);
        }
        // Pad rows 10..15 with round 0 values.
        for _ in 10..16 {
            col.push(BLAKE_SIGMA[0][i] as usize);
        }
        col
    })
}

fn add_blake_to_preprocessed_trace(
    circuit: &Circuit,
    multiplicities: &[usize],
    pp_trace: &mut PreProcessedTrace,
) {
    let Circuit { blake, .. } = circuit;
    add_blake_columns(blake, multiplicities, pp_trace);

    // Add blake sigma columns (16 columns of 16 rows each).
    let blake_sigma = gen_blake_sigma_columns();
    for (i, column) in blake_sigma.into_iter().enumerate() {
        pp_trace.push_column(PreProcessedColumnId { id: format!("blake_sigma_{i}") }, column);
    }
}

/// Adds TripleXor gates to the preprocessed trace. Preprocessed columns are in the format:
/// | input_addr_0 | input_addr_1 | input_addr_2 | output_addr | multiplicity |
fn add_triple_xor_to_preprocessed_trace(
    circuit: &Circuit,
    multiplicities: &[usize],
    pp_trace: &mut PreProcessedTrace,
) {
    let Circuit { triple_xor, .. } = circuit;
    let mut triple_xor_input_addr_0 = vec![];
    let mut triple_xor_input_addr_1 = vec![];
    let mut triple_xor_input_addr_2 = vec![];
    let mut triple_xor_output_addr = vec![];
    let mut triple_xor_multiplicity = vec![];
    for gate in triple_xor.iter() {
        let [input_a, input_b, input_c] = gate.uses()[..] else {
            panic!("Expected 3 uses for TripleXor")
        };
        let [out] = gate.yields()[..] else { panic!("Expected 1 yield for TripleXor") };
        triple_xor_input_addr_0.push(input_a);
        triple_xor_input_addr_1.push(input_b);
        triple_xor_input_addr_2.push(input_c);
        triple_xor_output_addr.push(out);
        triple_xor_multiplicity.push(multiplicities[out]);
    }

    pp_trace.push_column(
        PreProcessedColumnId { id: "triple_xor_input_addr_0".to_owned() },
        triple_xor_input_addr_0,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "triple_xor_input_addr_1".to_owned() },
        triple_xor_input_addr_1,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "triple_xor_input_addr_2".to_owned() },
        triple_xor_input_addr_2,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "triple_xor_output_addr".to_owned() },
        triple_xor_output_addr,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "triple_xor_multiplicity".to_owned() },
        triple_xor_multiplicity,
    );
}

const N_M31_TO_U32_PP_COLUMNS: usize = 3;

/// Adds M31ToU32 gates to preprocessed trace columns.
/// | input_address | output_address | multiplicity |
fn fill_m31_to_u32_columns(
    gates: &[M31ToU32],
    multiplicities: &[usize],
    columns: &mut [Vec<usize>; N_M31_TO_U32_PP_COLUMNS],
) {
    for gate in gates.iter() {
        let [input] = gate.uses()[..] else { panic!("Expected 1 use for M31ToU32") };
        let [out] = gate.yields()[..] else { panic!("Expected 1 yield for M31ToU32") };
        columns[0].push(input);
        columns[1].push(out);
        columns[2].push(multiplicities[out]);
    }
}

fn add_m31_to_u32_to_preprocessed_trace(
    circuit: &Circuit,
    multiplicities: &[usize],
    pp_trace: &mut PreProcessedTrace,
) {
    let mut columns: [_; N_M31_TO_U32_PP_COLUMNS] = std::array::from_fn(|_| vec![]);
    fill_m31_to_u32_columns(&circuit.m31_to_u32, multiplicities, &mut columns);

    let ids = ["m31_to_u32_input_addr", "m31_to_u32_output_addr", "m31_to_u32_multiplicity"];
    for (id, column) in zip_eq(ids, columns) {
        pp_trace.push_column(PreProcessedColumnId { id: id.to_owned() }, column);
    }
}

/// Adds BlakeGGate gates to the preprocessed trace. Preprocessed columns are in the format:
/// | input_addr_a | input_addr_b | input_addr_c | input_addr_d | input_addr_f0 | input_addr_f1 |
/// | output_addr_a | output_addr_b | output_addr_c | output_addr_d | multiplicity |
fn add_blake_g_gate_to_preprocessed_trace(
    circuit: &Circuit,
    multiplicities: &[usize],
    pp_trace: &mut PreProcessedTrace,
) {
    let Circuit { blake_g_gate, .. } = circuit;
    let mut blake_g_gate_input_addr_a = vec![];
    let mut blake_g_gate_input_addr_b = vec![];
    let mut blake_g_gate_input_addr_c = vec![];
    let mut blake_g_gate_input_addr_d = vec![];
    let mut blake_g_gate_input_addr_f0 = vec![];
    let mut blake_g_gate_input_addr_f1 = vec![];
    let mut blake_g_gate_output_addr_a = vec![];
    let mut blake_g_gate_output_addr_b = vec![];
    let mut blake_g_gate_output_addr_c = vec![];
    let mut blake_g_gate_output_addr_d = vec![];
    let mut blake_g_gate_multiplicity = vec![];
    for gate in blake_g_gate.iter() {
        let [input_a, input_b, input_c, input_d, input_f0, input_f1] = gate.uses()[..] else {
            panic!("Expected 6 uses for BlakeGGate")
        };
        let [out_a, out_b, out_c, out_d] = gate.yields()[..] else {
            panic!("Expected 4 yields for BlakeGGate")
        };
        blake_g_gate_input_addr_a.push(input_a);
        blake_g_gate_input_addr_b.push(input_b);
        blake_g_gate_input_addr_c.push(input_c);
        blake_g_gate_input_addr_d.push(input_d);
        blake_g_gate_input_addr_f0.push(input_f0);
        blake_g_gate_input_addr_f1.push(input_f1);
        blake_g_gate_output_addr_a.push(out_a);
        blake_g_gate_output_addr_b.push(out_b);
        blake_g_gate_output_addr_c.push(out_c);
        blake_g_gate_output_addr_d.push(out_d);

        // All four outputs of a Blake G gate share one multiplicity column. In the Blake
        // construction, each G output is consumed exactly once (by another G step or by the
        // triple-XOR).
        let mult = multiplicities[out_a];
        for y in [out_b, out_c, out_d] {
            assert_eq!(
                multiplicities[y], mult,
                "BlakeGGate output multiplicities must be identical"
            );
        }
        blake_g_gate_multiplicity.push(mult);
    }

    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_input_addr_a".to_owned() },
        blake_g_gate_input_addr_a,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_input_addr_b".to_owned() },
        blake_g_gate_input_addr_b,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_input_addr_c".to_owned() },
        blake_g_gate_input_addr_c,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_input_addr_d".to_owned() },
        blake_g_gate_input_addr_d,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_input_addr_f0".to_owned() },
        blake_g_gate_input_addr_f0,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_input_addr_f1".to_owned() },
        blake_g_gate_input_addr_f1,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_output_addr_a".to_owned() },
        blake_g_gate_output_addr_a,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_output_addr_b".to_owned() },
        blake_g_gate_output_addr_b,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_output_addr_c".to_owned() },
        blake_g_gate_output_addr_c,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_output_addr_d".to_owned() },
        blake_g_gate_output_addr_d,
    );
    pp_trace.push_column(
        PreProcessedColumnId { id: "blake_g_gate_multiplicity".to_owned() },
        blake_g_gate_multiplicity,
    );
}

/// A collection of preprocessed columns, whose values are publicly acknowledged, and independent of
/// the proof.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PreProcessedTrace {
    columns: OrderedHashMap<PreProcessedColumnId, Vec<usize>>,
}

impl PreProcessedTrace {
    fn push_column(&mut self, id: PreProcessedColumnId, column: Vec<usize>) {
        assert!(
            self.columns.insert(id.clone(), column).is_none(),
            "Duplicate preprocessed column id: {id:?}"
        );
    }

    /// Sorts preprocessed columns by size (ascending), preserving original order for ties.
    fn sort_by_size(&mut self) {
        // `IndexMap::sort_by` is a stable sort, so ties keep insertion order.
        self.columns.sort_by(|_, c1, _, c2| c1.len().cmp(&c2.len()));
    }
    fn add_non_circuit_preprocessed_columns(
        pp_trace: &mut PreProcessedTrace,
        log_seq_sizes: &[u32],
    ) {
        for &log_size in log_seq_sizes {
            let seq_column: Vec<usize> = (0..1_usize << log_size).collect();
            pp_trace
                .push_column(PreProcessedColumnId { id: format!("seq_{log_size}") }, seq_column);
        }
        let bitwise_xor: Vec<Vec<usize>> = [4, 7, 8, 9, 10]
            .into_iter()
            .flat_map(|n_bits| gen_xor_columns(n_bits).into_iter())
            .collect();
        let xor_col_ids = [
            "bitwise_xor_4_0",
            "bitwise_xor_4_1",
            "bitwise_xor_4_2",
            "bitwise_xor_7_0",
            "bitwise_xor_7_1",
            "bitwise_xor_7_2",
            "bitwise_xor_8_0",
            "bitwise_xor_8_1",
            "bitwise_xor_8_2",
            "bitwise_xor_9_0",
            "bitwise_xor_9_1",
            "bitwise_xor_9_2",
            "bitwise_xor_10_0",
            "bitwise_xor_10_1",
            "bitwise_xor_10_2",
        ];
        for (id, column) in zip_eq(xor_col_ids, bitwise_xor) {
            pp_trace.push_column(PreProcessedColumnId { id: id.to_owned() }, column);
        }
    }

    pub fn log_sizes(&self) -> Vec<u32> {
        self.columns.values().map(|c| c.len().ilog2()).collect()
    }

    pub fn ids(&self) -> Vec<PreProcessedColumnId> {
        self.columns.keys().cloned().collect()
    }

    pub fn n_columns(&self) -> usize {
        self.columns.len()
    }

    #[cfg(feature = "prover")]
    pub fn get_trace<B: Backend>(&self) -> Vec<CircleEvaluation<B, BaseField, BitReversedOrder>> {
        let to_evaluation = |vec: &[usize]| {
            let col = Col::<B, BaseField>::from_iter(vec.iter().cloned().map(BaseField::from));
            CircleEvaluation::new(CanonicCoset::new(col.len().ilog2()).circle_domain(), col)
        };

        self.columns.values().map(|c| to_evaluation(c)).collect()
    }

    pub fn get_column(&self, id: &PreProcessedColumnId) -> &Vec<usize> {
        self.columns.get(id).unwrap_or_else(|| panic!("Missing preprocessed column {id:?}"))
    }

    #[cfg(feature = "prover")]
    pub fn get_packed_column(&self, id: &PreProcessedColumnId) -> Vec<PackedM31> {
        let column = self.get_column(id);
        column
            .chunks_exact(N_LANES)
            .map(|c| PackedM31::from_array(std::array::from_fn(|i| BaseField::from(c[i]))))
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, PartialEq)]
pub struct PreprocessedCircuit {
    pub preprocessed_trace: Arc<PreProcessedTrace>,
    pub params: CircuitParams,
}

impl PreprocessedCircuit {
    /// Finalizes the context, then builds the preprocessed circuit.
    pub fn preprocess_circuit(context: &mut Context<impl IValue>) -> Self {
        finalize_context(context);
        Self::from_finalized_circuit(&context.circuit)
    }

    /// Builds the preprocessed circuit data (trace + params) from a finalized circuit.
    pub fn from_finalized_circuit(circuit: &Circuit) -> Self {
        let mut pp_trace = PreProcessedTrace::default();

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
        // Add Blake columns.
        add_blake_to_preprocessed_trace(circuit, &multiplicities, &mut pp_trace);
        let log_n_blake_updates = pp_trace
            .get_column(&PreProcessedColumnId { id: "finalize_flag".to_owned() })
            .len()
            .ilog2();
        // Add TripleXor columns.
        add_triple_xor_to_preprocessed_trace(circuit, &multiplicities, &mut pp_trace);
        // Add M31ToU32 columns.
        add_m31_to_u32_to_preprocessed_trace(circuit, &multiplicities, &mut pp_trace);
        // Add BlakeGGate columns.
        add_blake_g_gate_to_preprocessed_trace(circuit, &multiplicities, &mut pp_trace);

        // Generate seq columns for sizes needed by circuit components:
        // - 15, 16: needed by range_check_15 and range_check_16.
        // - 4: needed by blake_sigma.
        // - log_n_blake_updates: needed by blake_gate component which uses seq_of_component_size.
        let mut log_seq_sizes = vec![log_n_blake_updates, 4, 15, 16];
        log_seq_sizes.sort();
        log_seq_sizes.dedup();

        PreProcessedTrace::add_non_circuit_preprocessed_columns(&mut pp_trace, &log_seq_sizes);
        pp_trace.sort_by_size();

        // The trace size is the max between:
        // 1. The largest preprocessed column size.
        // 2. BlakeG trace size (= number of blake updates * 2^7).
        let max_pp_trace_log_size = pp_trace.log_sizes().into_iter().max().unwrap();
        let blake_g_log_size = log_n_blake_updates + 7;
        let trace_log_size = std::cmp::max(max_pp_trace_log_size, blake_g_log_size);

        let params = CircuitParams {
            trace_log_size,
            first_permutation_row: qm31_ops_trace_generator.first_permutation_row,
            n_blake_gates: circuit.blake.len(),
            output_addresses: circuit.output.iter().map(|out| out.in0).collect(),
        };

        Self { preprocessed_trace: Arc::new(pp_trace), params }
    }
}

/// Generates three columns of size (2^n_bits)^2. The first two columns are all ordered pairs of
/// n-bit values, and the third column contains the bitwise XOR of each pair.
fn gen_xor_columns(n_bits: usize) -> [Vec<usize>; 3] {
    let size = 1_usize << (2 * n_bits);
    let mask = (1_usize << n_bits) - 1;
    let mut columns: [Vec<usize>; 3] = std::array::from_fn(|_| vec![0; size]);
    for i in 0..size {
        let lhs = i & mask;
        let rhs = i >> n_bits;
        columns[0][i] = rhs;
        columns[1][i] = lhs;
        columns[2][i] = lhs ^ rhs;
    }
    columns
}
