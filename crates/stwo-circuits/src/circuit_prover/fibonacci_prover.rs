use crate::circuit_prover::preprocessed::PreProcessedTrace;
use crate::circuits::{context::Context, ops::guess};
use crate::eval;
use num_traits::{One, Zero};
use stwo::core::channel::Blake2sM31Channel;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo::core::poly::circle::CanonicCoset;
use stwo::core::vcs::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::prover::CommitmentSchemeProver;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::poly::circle::PolyOps;

const N: usize = 1024;
const LOG_N: u32 = (N as u32).ilog2();
// TODO(Gali): Calculate the correct domain size.
const TRACE_LOG_SIZE: u32 = LOG_N + 1;
const COMPOSITION_POLYNOMIAL_LOG_SIZE: u32 = TRACE_LOG_SIZE + 1;

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
pub fn prove_fibonacci(context: Context<QM31>) {
    let pcs_config = PcsConfig::default();

    // Precompute twiddles.
    // Double the size of the composition polynomial domains because we compute on a half-coset,
    // and account for blowup factor.
    let twiddles = SimdBackend::precompute_twiddles(
        CanonicCoset::new(
            COMPOSITION_POLYNOMIAL_LOG_SIZE + 1 + pcs_config.fri_config.log_blowup_factor,
        )
        .circle_domain()
        .half_coset,
    );

    // Setup protocol.
    let channel = &mut Blake2sM31Channel::default();
    pcs_config.mix_into(channel);
    let mut commitment_scheme =
        CommitmentSchemeProver::<SimdBackend, Blake2sM31MerkleChannel>::new(pcs_config, &twiddles);

    // Preprocessed trace.
    let preprocessed_trace =
        PreProcessedTrace::generate_preprocessed_trace(&context.circuit).gen_trace();
    let mut tree_builder = commitment_scheme.tree_builder();
    tree_builder.extend_evals(preprocessed_trace);
    tree_builder.commit(channel);

    // TODO(Gali): Implement.
}
