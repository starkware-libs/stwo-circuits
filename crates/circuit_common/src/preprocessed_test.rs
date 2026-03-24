use crate::preprocessed::PreprocessedCircuit;
use circuits::circuit::{
    Add, Blake, BlakeGGate, Circuit, Eq, M31ToU32Gate, Mul, PointwiseMul, Sub, TripleXorGate,
};
use expect_test::expect;
use itertools::Itertools;
use stwo::prover::backend::Column;
use stwo::prover::backend::simd::SimdBackend;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

#[test]
fn test_preprocess_circuit() {
    let mut circuit = Circuit::default();
    circuit.add.push(Add { in0: 0, in1: 1, out: 2 });
    circuit.add.push(Add { in0: 3, in1: 4, out: 5 });
    circuit.sub.push(Sub { in0: 6, in1: 7, out: 8 });
    circuit.sub.push(Sub { in0: 9, in1: 10, out: 11 });
    circuit.mul.push(Mul { in0: 12, in1: 13, out: 14 });
    circuit.mul.push(Mul { in0: 15, in1: 16, out: 17 });
    circuit.pointwise_mul.push(PointwiseMul { in0: 18, in1: 19, out: 20 });
    circuit.pointwise_mul.push(PointwiseMul { in0: 21, in1: 22, out: 23 });
    circuit.eq.push(Eq { in0: 0, in1: 1 });
    circuit.eq.push(Eq { in0: 0, in1: 2 });
    for i in 0..16 {
        let in0 = (i * 4) % 24;
        let in1 = (i * 4 + 1) % 24;
        let in2 = (i * 4 + 2) % 24;
        let in3 = (i * 4 + 3) % 24;
        circuit.blake.push(Blake {
            input: vec![[in0, in1, in2, in3]],
            n_bytes: 64,
            out0: 24 + 2 * i,
            out1: 24 + 2 * i + 1,
        });
    }
    circuit.n_vars = 56;

    let preprocessed_trace = PreprocessedCircuit::from_finalized_circuit(&circuit)
        .preprocessed_trace
        .get_trace::<SimdBackend>();

    assert_eq!(preprocessed_trace.len(), 59);
    let lengths = preprocessed_trace.iter().map(|column| column.values.len()).collect_vec();
    expect![[r#"
        [
            2,
            2,
            8,
            8,
            8,
            8,
            8,
            8,
            8,
            8,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            16,
            256,
            256,
            256,
            16384,
            16384,
            16384,
            32768,
            65536,
            65536,
            65536,
            65536,
            262144,
            262144,
            262144,
            1048576,
            1048576,
            1048576,
        ]
    "#]]
    .assert_debug_eq(&lengths);
}

#[test]
fn test_preprocess_decomposed_gates() {
    let mut circuit = Circuit::default();

    // Wire layout:
    //   0..15:   m31_to_u32 inputs  (yielded by Add gates)
    //   16..31:  m31_to_u32 outputs (used by blake_g as inputs)
    //   32..95:  blake_g outputs    (each used exactly once by Eq)
    //   96..143: triple_xor inputs  (yielded by Add gates)
    //   144..159: triple_xor outputs (each used exactly once by Eq)
    //   160..191: blake monolithic outputs

    // M31ToU32 gates: 16 gates.
    for i in 0..16 {
        circuit.m31_to_u32.push(M31ToU32Gate { input: i, out: 16 + i });
    }

    // BlakeG gates: 16 gates, 6 inputs from wires 16..31 (wrapping), 4 outputs to wires 32..95.
    for i in 0..16 {
        let inp = |j: usize| 16 + (i * 6 + j) % 16;
        let base_out = 32 + i * 4;
        circuit.blake_g.push(BlakeGGate {
            a: inp(0),
            b: inp(1),
            c: inp(2),
            d: inp(3),
            m0: inp(4),
            m1: inp(5),
            out_a: base_out,
            out_b: base_out + 1,
            out_c: base_out + 2,
            out_d: base_out + 3,
        });
    }

    // TripleXor gates: 16 gates, inputs from dedicated wires 96..143, outputs to 144..159.
    for i in 0..16 {
        circuit.triple_xor.push(TripleXorGate {
            a: 96 + i * 3,
            b: 96 + i * 3 + 1,
            c: 96 + i * 3 + 2,
            out: 144 + i,
        });
    }

    // Monolithic blake gates (required for "finalize_flag" column).
    for i in 0..16 {
        circuit.blake.push(Blake {
            input: vec![[0, 0, 0, 0]],
            n_bytes: 64,
            out0: 160 + 2 * i,
            out1: 160 + 2 * i + 1,
        });
    }

    // Yield wires 0..15 (m31_to_u32 inputs) and 96..143 (triple_xor inputs).
    for i in 0..16 {
        circuit.add.push(Add { in0: 0, in1: 0, out: i });
    }
    for i in 96..144 {
        circuit.add.push(Add { in0: 0, in1: 0, out: i });
    }
    // Use blake_g outputs (32..95) and triple_xor outputs (144..159) via Eq.
    for i in 32..96 {
        circuit.eq.push(Eq { in0: i, in1: 0 });
    }
    for i in 144..160 {
        circuit.eq.push(Eq { in0: i, in1: 0 });
    }

    circuit.n_vars = 192;

    let preprocessed = PreprocessedCircuit::from_finalized_circuit(&circuit);
    let pp_trace = &preprocessed.preprocessed_trace;

    // Verify M31ToU32 columns exist and have correct values.
    let m31_input_addr =
        pp_trace.get_column(&PreProcessedColumnId { id: "m31_to_u32_input_addr".to_owned() });
    let m31_output_addr =
        pp_trace.get_column(&PreProcessedColumnId { id: "m31_to_u32_output_addr".to_owned() });
    let m31_mult =
        pp_trace.get_column(&PreProcessedColumnId { id: "m31_to_u32_multiplicity".to_owned() });
    assert_eq!(m31_input_addr.len(), 16);
    assert_eq!(m31_input_addr[0], 0);
    assert_eq!(m31_output_addr[0], 16);
    // All mults should be > 0 (each output is used by blake_g gates).
    assert!(m31_mult.iter().all(|&m| m > 0));

    // Verify BlakeG columns exist and have correct addresses.
    let bg_a =
        pp_trace.get_column(&PreProcessedColumnId { id: "blake_g_gate_input_addr_a".to_owned() });
    let bg_out_a =
        pp_trace.get_column(&PreProcessedColumnId { id: "blake_g_gate_output_addr_a".to_owned() });
    let bg_mult =
        pp_trace.get_column(&PreProcessedColumnId { id: "blake_g_gate_multiplicity".to_owned() });
    assert_eq!(bg_a.len(), 16);
    assert_eq!(bg_a[0], 16); // First gate's a = wire 16.
    assert_eq!(bg_out_a[0], 32); // First gate's out_a = wire 32.
    assert!(bg_mult.iter().all(|&m| m <= 1));

    // Verify TripleXor columns exist and have correct addresses.
    let tx_a =
        pp_trace.get_column(&PreProcessedColumnId { id: "triple_xor_input_addr_0".to_owned() });
    let tx_out =
        pp_trace.get_column(&PreProcessedColumnId { id: "triple_xor_output_addr".to_owned() });
    let tx_mult =
        pp_trace.get_column(&PreProcessedColumnId { id: "triple_xor_multiplicity".to_owned() });
    assert_eq!(tx_a.len(), 16);
    assert_eq!(tx_a[0], 96); // First triple_xor gate's a = wire 96.
    assert_eq!(tx_out[0], 144); // First triple_xor gate's out = wire 144.
    assert!(tx_mult.iter().all(|&m| m == 1));
}
