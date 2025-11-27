use crate::circuit_prover::claim_generator::CircuitClaimGenerator;
use crate::circuit_prover::preprocessed::generate_preprocessed_trace;
use crate::circuits::context::Context;
use stwo::core::channel::Blake2sM31Channel;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo::core::poly::circle::CanonicCoset;
use stwo::core::vcs::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::prover::CommitmentSchemeProver;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::poly::circle::PolyOps;

// TODO(Gali): Change to 1024 after padding components.
const N: usize = 1022;
const LOG_N: u32 = (N as u32).ilog2();
// TODO(Gali): Calculate the correct domain size according to the circuit.
const TRACE_LOG_SIZE: u32 = LOG_N + 1;
const COMPOSITION_POLYNOMIAL_LOG_SIZE: u32 = TRACE_LOG_SIZE + 1;

#[cfg(test)]
#[path = "prover_test.rs"]
pub mod test;

pub fn prove(context: Context<QM31>) {
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
    let preprocessed_trace = generate_preprocessed_trace(&context.circuit);
    let mut tree_builder = commitment_scheme.tree_builder();
    tree_builder.extend_evals(preprocessed_trace);
    tree_builder.commit(channel);

    // Run circuit.
    let claim_generator = CircuitClaimGenerator::new(context);
    // Base trace.
    let mut tree_builder = commitment_scheme.tree_builder();
    let (claim, _interaction_generator) = claim_generator.write_trace(&mut tree_builder);
    claim.mix_into(channel);
    tree_builder.commit(channel);

    // TODO(Gali): Implement.
}
