use crate::witness::preprocessed::PreprocessedCircuit;
use circuits::circuit::{Add, Blake, Circuit, Eq, Mul, PointwiseMul, Sub, Var};
use expect_test::expect;
use itertools::Itertools;
use stwo::prover::backend::Column;
use stwo::prover::backend::simd::SimdBackend;

const fn var(idx: usize) -> Var {
    Var { idx }
}

#[test]
fn test_preprocess_circuit() {
    let mut circuit = Circuit::default();
    circuit.add.push(Add { in0: var(0), in1: var(1), out: var(2) });
    circuit.add.push(Add { in0: var(3), in1: var(4), out: var(5) });
    circuit.sub.push(Sub { in0: var(6), in1: var(7), out: var(8) });
    circuit.sub.push(Sub { in0: var(9), in1: var(10), out: var(11) });
    circuit.mul.push(Mul { in0: var(12), in1: var(13), out: var(14) });
    circuit.mul.push(Mul { in0: var(15), in1: var(16), out: var(17) });
    circuit.pointwise_mul.push(PointwiseMul { in0: var(18), in1: var(19), out: var(20) });
    circuit.pointwise_mul.push(PointwiseMul { in0: var(21), in1: var(22), out: var(23) });
    circuit.eq.push(Eq { in0: var(0), in1: var(1) });
    circuit.eq.push(Eq { in0: var(0), in1: var(2) });
    for i in 0..16 {
        let in0 = (i * 4) % 24;
        let in1 = (i * 4 + 1) % 24;
        let in2 = (i * 4 + 2) % 24;
        let in3 = (i * 4 + 3) % 24;
        circuit.blake.push(Blake {
            input: vec![[var(in0), var(in1), var(in2), var(in3)]],
            n_bytes: 64,
            out0: var(24 + 2 * i),
            out1: var(24 + 2 * i + 1),
        });
    }
    circuit.n_vars = 56;

    let preprocessed_trace = PreprocessedCircuit::from_finalized_circuit(&circuit)
        .preprocessed_trace
        .get_trace::<SimdBackend>();

    assert_eq!(preprocessed_trace.len(), 73);
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
            32,
            64,
            128,
            256,
            256,
            256,
            256,
            512,
            1024,
            2048,
            4096,
            8192,
            16384,
            16384,
            16384,
            16384,
            32768,
            65536,
            65536,
            65536,
            65536,
            131072,
            262144,
            262144,
            262144,
            262144,
            524288,
            1048576,
            1048576,
            1048576,
            1048576,
        ]
    "#]]
    .assert_debug_eq(&lengths);
}
