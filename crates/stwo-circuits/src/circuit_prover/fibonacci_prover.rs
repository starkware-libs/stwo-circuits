use crate::circuits::{context::Context, ops::guess};
use crate::eval;
use num_traits::{One, Zero};
use stwo::core::fields::qm31::QM31;

const N: usize = 1024;

#[cfg(test)]
#[path = "fibonacci_prover_test.rs"]
pub mod test;

pub fn build_fibonacci_circuit() -> Context<QM31> {
    let mut context = Context::<QM31>::default();

    let (mut a, mut b) = (guess(&mut context, QM31::zero()), guess(&mut context, QM31::one()));
    for _ in 2..N {
        (a, b) = (b, eval!(&mut context, (a) + (b)));
    }

    context
}

/// Proves a Fibonacci circuit that computes the N-th Fibonacci number.
/// a_0 = 0, a_1 = 1, a_n = a_{n-1} + a_{n-2}.
pub fn prove_fibonacci(mut _context: Context<QM31>) {
    // TODO(Gali): Implement.
}
