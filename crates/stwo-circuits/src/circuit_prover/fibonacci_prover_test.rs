use expect_test::expect;

use crate::circuit_prover::fibonacci_prover::{N, build_fibonacci_circuit, prove_fibonacci};

#[test]
fn test_prove_fibonacci() {
    let fibonacci_circuit = build_fibonacci_circuit();

    assert_eq!(fibonacci_circuit.values().len(), N + 2);
    expect![[r#"
        (980146853 + 0i) + (0 + 0i)u
    "#]]
    .assert_debug_eq(&fibonacci_circuit.values()[N + 1]);

    prove_fibonacci(fibonacci_circuit);
}
