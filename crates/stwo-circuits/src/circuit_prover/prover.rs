use crate::circuit_prover::witness::preprocessed::PreProcessedTrace;
use crate::circuit_prover::witness::trace::write_trace;
use crate::circuits::context::Context;
use stwo::core::channel::Blake2sM31Channel;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo::core::poly::circle::CanonicCoset;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::prover::CommitmentSchemeProver;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::poly::circle::PolyOps;

const COMPOSITION_POLYNOMIAL_LOG_DEGREE_BOUND: u32 = 1;

#[cfg(test)]
#[path = "prover_test.rs"]
pub mod test;

pub fn prove(context: Context<QM31>) {
    let pcs_config = PcsConfig::default();

    // Generate preprocessed trace.
    let preprocessed_trace = PreProcessedTrace::generate_preprocessed_trace(&context.circuit);

    // The trace size is the size of the largest column in the preprocessed trace (since all
    // components have preprocessed columns).
    let trace_log_size = preprocessed_trace.log_sizes().into_iter().max().unwrap();

    // Precompute twiddles.
    // Account for blowup factor and for composition polynomial calculation (taking the max since
    // the composition polynomial is split prior to LDE).
    let twiddles = SimdBackend::precompute_twiddles(
        CanonicCoset::new(
            trace_log_size
                + std::cmp::max(
                    pcs_config.fri_config.log_blowup_factor,
                    COMPOSITION_POLYNOMIAL_LOG_DEGREE_BOUND,
                ),
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
    let mut tree_builder = commitment_scheme.tree_builder();
    tree_builder.extend_evals(preprocessed_trace.get_trace::<SimdBackend>());
    tree_builder.commit(channel);

    // Base trace.
    let mut tree_builder = commitment_scheme.tree_builder();
    let claim = write_trace(context.values(), &preprocessed_trace, &mut tree_builder);
    claim.mix_into(channel);
    tree_builder.commit(channel);

    // TODO(Gali): Implement.
}
