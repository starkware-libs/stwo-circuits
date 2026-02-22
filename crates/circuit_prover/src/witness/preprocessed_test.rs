use crate::witness::preprocessed::PreProcessedTrace;
use circuits::circuit::{Add, Blake, Circuit, Eq, Mul, PointwiseMul, Sub};
use expect_test::expect;
use itertools::Itertools;
use stwo::prover::backend::Column;
use stwo::prover::backend::simd::SimdBackend;

#[test]
fn test_generate_preprocessed_trace() {
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

    let preprocessed_trace =
        PreProcessedTrace::generate_preprocessed_trace(&circuit).0.get_trace::<SimdBackend>();

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
