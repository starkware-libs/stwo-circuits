use crate::circuit_prover::prover::prove;
use crate::circuits::{context::Context, ops::guess};
use crate::eval;
use expect_test::expect;
use num_traits::{One, Zero};
use stwo::core::fields::qm31::QM31;

// TODO(Gali): Change to 1024 after padding components.
const N: usize = 1022;

pub fn build_fibonacci_context() -> Context<QM31> {
    let mut context = Context::<QM31>::default();

    let (mut a, mut b) = (guess(&mut context, QM31::zero()), guess(&mut context, QM31::one()));
    for _ in 2..N {
        (a, b) = (b, eval!(&mut context, (a) + (b)));
    }

    expect![[r#"
        (1397909768 + 0i) + (0 + 0i)u
    "#]]
    .assert_debug_eq(&context.get(b));

    context
}

#[test]
fn test_prove_fibonacci() {
    let mut fibonacci_context = build_fibonacci_context();
    fibonacci_context.finalize_guessed_vars();
    fibonacci_context.validate_circuit();
    prove(fibonacci_context);
}
