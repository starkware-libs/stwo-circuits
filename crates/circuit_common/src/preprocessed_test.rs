use crate::preprocessed::PreprocessedCircuit;
use circuits::circuit::{Add, Blake, Circuit, Eq, M31ToU32, Mul, PointwiseMul, Sub, TripleXor};
use expect_test::expect;
use itertools::Itertools;
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
    for i in 0..16 {
        circuit.triple_xor.push(TripleXor { input_a: 0, input_b: 1, input_c: 2, out: 56 + i });
    }
    for i in 0..16 {
        circuit.m31_to_u32.push(M31ToU32 { input: 0, out: 72 + i });
    }
    circuit.n_vars = 88;

    let preprocessed = PreprocessedCircuit::from_finalized_circuit(&circuit).preprocessed_trace;

    assert_eq!(preprocessed.columns.len(), 78);
    let lengths = preprocessed.columns.iter().map(|column| column.len()).collect_vec();
    expect![[r#"
        [
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
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
