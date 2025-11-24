use stwo::core::fields::qm31::QM31;

use crate::circuits::context::Context;
use crate::circuits::context::Var;
use crate::circuits::ops::Guess;
use crate::stark_verifier::fri_proof::FriConfig;
use crate::stark_verifier::proof::Proof;
use crate::stark_verifier::proof::ProofConfig;
use crate::stark_verifier::proof::dummy_qm31_proof;

const N: usize = 1024;

#[cfg(test)]
#[path = "fibonacci_prover_test.rs"]
pub mod test;

/// Proves a fibonacci circuit that computes the 1024th Fibonacci number.
/// a_0 = 0, a_1 = 1, a_n = a_{n-1} + a_{n-2}.
pub fn prove_fibonacci(mut context: Context<QM31>) -> Proof<Var> {
    //TODO(Gali): Implement.
    let config = ProofConfig {
        n_proof_of_work_bits: 0,
        n_preprocessed_columns: 0,
        n_trace_columns: 0,
        n_interaction_columns: 0,
        fri: FriConfig {
            log_trace_size: N.ilog2().try_into().unwrap(),
            log_blowup_factor: 1,
            n_queries: 3,
            log_n_last_layer_coefs: 0,
        },
    };
    let dummy_proof = dummy_qm31_proof(&config);

    dummy_proof.guess(&mut context)
}
