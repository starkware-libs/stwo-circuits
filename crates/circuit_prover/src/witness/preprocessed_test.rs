use crate::witness::preprocessed::PreProcessedTrace;
use circuits::circuit::{Add, Circuit, Eq, Mul, PointwiseMul, Sub};
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
    circuit.n_vars = 24;
    // TODO(Gali): Add blake gates

    let preprocessed_trace =
        PreProcessedTrace::generate_preprocessed_trace(&circuit).0.get_trace::<SimdBackend>();

    assert_eq!(preprocessed_trace.len(), 10);
    assert_eq!(preprocessed_trace[0].values.len(), 2);
    assert_eq!(preprocessed_trace[1].values.len(), 2);
    assert_eq!(preprocessed_trace[2].values.len(), 8);
    assert_eq!(preprocessed_trace[3].values.len(), 8);
    assert_eq!(preprocessed_trace[4].values.len(), 8);
    assert_eq!(preprocessed_trace[5].values.len(), 8);
    assert_eq!(preprocessed_trace[6].values.len(), 8);
    assert_eq!(preprocessed_trace[7].values.len(), 8);
    assert_eq!(preprocessed_trace[8].values.len(), 8);
    assert_eq!(preprocessed_trace[9].values.len(), 8);
    expect![[r#"
        [M31(0), M31(0)]
        [M31(1), M31(2)]
        [M31(1), M31(1), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0)]
        [M31(0), M31(0), M31(1), M31(1), M31(0), M31(0), M31(0), M31(0)]
        [M31(0), M31(0), M31(0), M31(0), M31(1), M31(1), M31(0), M31(0)]
        [M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(1), M31(1)]
        [M31(0), M31(3), M31(6), M31(9), M31(12), M31(15), M31(18), M31(21)]
        [M31(1), M31(4), M31(7), M31(10), M31(13), M31(16), M31(19), M31(22)]
        [M31(2), M31(5), M31(8), M31(11), M31(14), M31(17), M31(20), M31(23)]
        [M31(1), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0)]"#]]
    .assert_eq(
        &preprocessed_trace
            .into_iter()
            .map(|eval| format!("{:?}", eval.values.clone().into_cpu_vec()))
            .collect_vec()
            .join("\n"),
    );
}
