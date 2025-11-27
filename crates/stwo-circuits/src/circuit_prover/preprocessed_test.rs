use crate::circuit_prover::preprocessed::generate_preprocessed_trace;
use crate::circuits::circuit::{Add, Circuit, Eq, Mul, PointwiseMul, Sub};
use expect_test::expect;
use itertools::Itertools;

#[test]
fn test_generate_empty_preprocessed_trace() {
    let circuit = Circuit::default();
    let preprocessed_trace = generate_preprocessed_trace(&circuit);
    assert!(preprocessed_trace.is_empty());
}

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
    circuit.eq.push(Eq { in0: 24, in1: 25 });
    circuit.eq.push(Eq { in0: 26, in1: 27 });
    // TODO(Gali): Add blake gates

    let preprocessed_trace = generate_preprocessed_trace(&circuit);

    assert_eq!(preprocessed_trace.len(), 14);
    preprocessed_trace.iter().for_each(|eval| assert_eq!(eval.length, 2));
    expect![[r#"
        [M31(0), M31(3)]
        [M31(1), M31(4)]
        [M31(2), M31(5)]
        [M31(6), M31(9)]
        [M31(7), M31(10)]
        [M31(8), M31(11)]
        [M31(12), M31(15)]
        [M31(13), M31(16)]
        [M31(14), M31(17)]
        [M31(18), M31(21)]
        [M31(19), M31(22)]
        [M31(20), M31(23)]
        [M31(24), M31(26)]
        [M31(25), M31(27)]"#]]
    .assert_eq(
        &preprocessed_trace
            .into_iter()
            .map(|eval| format!("{:?}", eval.values.clone().into_cpu_vec()))
            .collect_vec()
            .join("\n"),
    );
}
