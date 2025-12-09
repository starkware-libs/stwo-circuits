use crate::circuit_air::components::CircuitComponents;
use crate::circuit_air::components::CircuitInteractionElements;
use crate::circuit_air::components::lookup_sum;
use crate::circuit_prover::witness::preprocessed::PreProcessedTrace;
use crate::circuit_prover::witness::trace::write_trace;
use crate::circuits::context::Context;
use num_traits::Zero;
use stwo::core::channel::Blake2sM31Channel;
use stwo::core::fields::qm31::QM31;
use stwo::core::fields::qm31::SecureField;
use stwo::core::pcs::PcsConfig;
use stwo::core::poly::circle::CanonicCoset;
use stwo::core::vcs::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::prover::CommitmentSchemeProver;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::poly::circle::PolyOps;

#[cfg(test)]
#[path = "prover_test.rs"]
pub mod test;

pub fn prove(context: Context<QM31>) {
    let pcs_config = PcsConfig::default();

    // Generate preprocessed trace.
    let preprocessed_trace =
        PreProcessedTrace::<SimdBackend>::generate_preprocessed_trace(&context.circuit);

    // The trace size is the size of the largest column in the preprocessed trace (since all
    // components have preprocessed columns).
    let trace_log_size = preprocessed_trace.log_sizes().into_iter().max().unwrap();
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
    tree_builder.extend_evals(preprocessed_trace.get_trace());
    tree_builder.commit(channel);

    // Base trace.
    let mut tree_builder = commitment_scheme.tree_builder();
    let (claim, interaction_generator) =
        write_trace(&context.circuit, context.values(), &preprocessed_trace, &mut tree_builder);
    claim.mix_into(channel);
    tree_builder.commit(channel);

    // Draw interaction elements.
    // TODO(Gali): Add proof of work.
    let interaction_elements = CircuitInteractionElements::draw(channel);

    // Interaction trace.
    let mut tree_builder = commitment_scheme.tree_builder();
    let interaction_claim =
        interaction_generator.write_interaction_trace(&mut tree_builder, &interaction_elements);

    // Validate lookup argument.
    debug_assert_eq!(
        lookup_sum(&claim, &interaction_elements, &interaction_claim),
        SecureField::zero()
    );

    interaction_claim.mix_into(channel);
    tree_builder.commit(channel);

    // Component provers.
    let component_builder = CircuitComponents::new(
        &claim,
        &interaction_elements,
        &interaction_claim,
        &preprocessed_trace.ids(),
    );

    let _components = component_builder.provers();

    // TODO(Gali): Implement.
}
