use crate::circuit_prover::preprocessed::generate_preprocessed_trace;
use crate::circuits::context::Context;
use stwo::core::channel::Blake2sM31Channel;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo::core::poly::circle::CanonicCoset;
use stwo::core::vcs::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::prover::CommitmentSchemeProver;
use stwo::prover::backend::Column;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::poly::circle::PolyOps;

#[cfg(test)]
#[path = "prover_test.rs"]
pub mod test;

pub fn prove(context: Context<QM31>) {
    let pcs_config = PcsConfig::default();

    // Generate preprocessed trace.
    let preprocessed_trace = generate_preprocessed_trace::<SimdBackend>(&context.circuit);

    // The trace size is the size of the largest column in the preprocessed trace (since all
    // components have preprocessed columns).
    let trace_log_size =
        preprocessed_trace.iter().map(|col| col.values.len()).max().unwrap().ilog2();
    let composition_polynomial_log_size = trace_log_size + 1;

    // Precompute twiddles.
    // Double the size of the composition polynomial domains because we compute on a half-coset,
    // and account for blowup factor.
    let twiddles = SimdBackend::precompute_twiddles(
        CanonicCoset::new(
            composition_polynomial_log_size + 1 + pcs_config.fri_config.log_blowup_factor,
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
    tree_builder.extend_evals(preprocessed_trace);
    tree_builder.commit(channel);

    // TODO(Gali): Implement.
}
