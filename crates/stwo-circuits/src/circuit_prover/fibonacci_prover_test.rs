use crate::{circuit_prover::fibonacci_prover::prove_fibonacci, circuits::context::Context};

#[test]
fn test_prove_fibonacci() {
    let fibonacci_circuit = Context::default();
    prove_fibonacci(fibonacci_circuit);
}
