use crate::circuit_prover::preprocessed::generate_preprocessed_trace;
use crate::circuits::circuit::{Add, Circuit, Eq, Mul, PointwiseMul, Sub, Var};
use expect_test::expect;
use itertools::Itertools;
use stwo::prover::backend::simd::m31::N_LANES;

#[test]
fn test_generate_empty_preprocessed_trace() {
    let circuit = Circuit::default();
    let preprocessed_trace = generate_preprocessed_trace(&circuit);
    assert!(preprocessed_trace.is_empty());
}

#[test]
fn test_generate_preprocessed_trace() {
    let mut circuit = Circuit::default();
    circuit.add.push(Add { in0: Var { idx: 0 }, in1: Var { idx: 1 }, out: Var { idx: 2 } });
    circuit.add.push(Add { in0: Var { idx: 3 }, in1: Var { idx: 4 }, out: Var { idx: 5 } });
    circuit.sub.push(Sub { in0: Var { idx: 6 }, in1: Var { idx: 7 }, out: Var { idx: 8 } });
    circuit.mul.push(Mul { in0: Var { idx: 9 }, in1: Var { idx: 10 }, out: Var { idx: 11 } });
    circuit.pointwise_mul.push(PointwiseMul {
        in0: Var { idx: 12 },
        in1: Var { idx: 13 },
        out: Var { idx: 14 },
    });
    circuit.eq.push(Eq { in0: Var { idx: 15 }, in1: Var { idx: 16 } });
    // TODO(Gali): Add blake gates

    let preprocessed_trace = generate_preprocessed_trace(&circuit);

    assert_eq!(preprocessed_trace.len(), 14);
    assert_eq!(preprocessed_trace[0].length, N_LANES);
    assert_eq!(preprocessed_trace[3].length, N_LANES);
    assert_eq!(preprocessed_trace[6].length, N_LANES);
    assert_eq!(preprocessed_trace[9].length, N_LANES);
    assert_eq!(preprocessed_trace[12].length, N_LANES);
    expect![[r#"
        [M31(0), M31(3), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0)]
        [M31(1), M31(4), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0)]
        [M31(2), M31(5), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0)]
        [M31(6), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0)]
        [M31(7), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0)]
        [M31(8), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0)]
        [M31(9), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0)]
        [M31(10), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0)]
        [M31(11), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0)]
        [M31(12), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0)]
        [M31(13), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0)]
        [M31(14), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0)]
        [M31(15), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0)]
        [M31(16), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0)]"#]]
    .assert_eq(
        &preprocessed_trace
            .iter()
            .map(|eval| format!("{:?}", eval.values.clone().into_cpu_vec()))
            .collect_vec()
            .join("\n"),
    );
}
