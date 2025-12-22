use crate::circuit_prover::prover::{finalize_context, prove_circuit};
use crate::circuits::{context::Context, ops::guess};
use crate::eval;
use expect_test::expect;
use num_traits::{One, Zero};
use stwo::core::fields::qm31::QM31;

const N: usize = 1030;

pub fn build_fibonacci_context() -> Context<QM31> {
    let mut context = Context::<QM31>::default();

    let (mut a, mut b) = (guess(&mut context, QM31::zero()), guess(&mut context, QM31::one()));
    for _ in 2..N {
        (a, b) = (b, eval!(&mut context, (a) + (b)));
    }

    expect![[r#"
        (809871181 + 0i) + (0 + 0i)u
    "#]]
    .assert_debug_eq(&context.get(b));

    context
}

#[test]
fn test_prove_fibonacci() {
    let fibonacci_context = build_fibonacci_context();
    fibonacci_context.validate_circuit();

    let proof = prove_circuit(fibonacci_context);

    assert!(proof.is_ok());
}

#[test]
fn test_finalize_context() {
    let context = build_fibonacci_context();
    let finalized_context = finalize_context(context);

    assert!(finalized_context.circuit.add.len().is_power_of_two());
    finalized_context.validate_circuit();
}
