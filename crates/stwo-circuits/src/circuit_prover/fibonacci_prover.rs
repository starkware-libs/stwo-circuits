use crate::circuits::context::Context;
use stwo::core::fields::qm31::QM31;

#[cfg(test)]
#[path = "fibonacci_prover_test.rs"]
pub mod test;

/// Proves a Fibonacci circuit that computes the N-th Fibonacci number.
/// a_0 = 0, a_1 = 1, a_n = a_{n-1} + a_{n-2}.
pub fn prove_fibonacci(mut _context: Context<QM31>) {
    // TODO(Gali): Implement.
}
