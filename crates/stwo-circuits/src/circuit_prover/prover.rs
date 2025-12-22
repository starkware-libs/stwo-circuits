use crate::circuit_air::components::CircuitComponents;
use crate::circuit_air::components::CircuitInteractionElements;
use crate::circuit_air::components::lookup_sum;
use crate::circuit_prover::witness::preprocessed::PreProcessedTrace;
use crate::circuit_prover::witness::trace::write_interaction_trace;
use crate::circuit_prover::witness::trace::write_trace;
use crate::circuits::context::Context;
use crate::eval;
use num_traits::Zero;
use stwo::core::channel::Blake2sM31Channel;
use stwo::core::fields::qm31::QM31;
use stwo::core::fields::qm31::SecureField;
use stwo::core::pcs::PcsConfig;
use stwo::core::poly::circle::CanonicCoset;
use stwo::core::proof::ExtendedStarkProof;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;
use stwo::prover::CommitmentSchemeProver;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::backend::simd::m31::N_LANES;
use stwo::prover::poly::circle::PolyOps;
use stwo::prover::{ProvingError, prove_ex};

const COMPOSITION_POLYNOMIAL_LOG_DEGREE_BOUND: u32 = 1;

fn pad_qm31_ops(context: &mut Context<QM31>) {
    let qm31_ops_n_rows = context.circuit.add.len()
        + context.circuit.sub.len()
        + context.circuit.mul.len()
        + context.circuit.pointwise_mul.len();

    let qm31_padding =
        std::cmp::max(qm31_ops_n_rows.next_power_of_two(), N_LANES) - qm31_ops_n_rows;
    let zero = context.zero();
    for _ in 0..qm31_padding {
        eval!(context, (zero) + (zero));
    }
}

/// Finalizes the context by appending gates to the context for:
/// - Hashing the constants.
/// - Hashing the outputs.
/// - Padding the components to a power of two.
fn finalize_context(context: Context<QM31>) -> Context<QM31> {
    let mut context = context;

    // TODO(Gali): Hash the constants.

    // TODO(Gali): Hash the outputs (all variables that have no uses).

    // Padding the components to a power of two.
    pad_qm31_ops(&mut context);
    // TODO(Gali): Pad blake gates.

    context
}

#[cfg(test)]
#[path = "prover_test.rs"]
pub mod test;

pub fn prove_circuit(
    context: Context<QM31>,
) -> Result<ExtendedStarkProof<Blake2sM31MerkleHasher>, ProvingError> {
    let context = finalize_context(context);
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
    let (claim, interaction_generator) =
        write_trace(context.values(), &preprocessed_trace, &mut tree_builder);
    claim.mix_into(channel);
    tree_builder.commit(channel);

    // Draw interaction elements.
    // TODO(Gali): Add proof of work.
    let interaction_elements = CircuitInteractionElements::draw(channel);

    // Interaction trace.
    let mut tree_builder = commitment_scheme.tree_builder();
    let interaction_claim = write_interaction_trace(
        &claim,
        interaction_generator,
        &mut tree_builder,
        &interaction_elements,
    );

    // Validate lookup argument.
    debug_assert_eq!(lookup_sum(&interaction_claim), SecureField::zero());

    interaction_claim.mix_into(channel);
    tree_builder.commit(channel);

    // Component provers.
    let component_builder = CircuitComponents::<SimdBackend>::new(
        &claim,
        &interaction_elements,
        &interaction_claim,
        &preprocessed_trace.ids(),
    );

    let components = component_builder.provers();

    // Prove stark.
    prove_ex::<SimdBackend, _>(&components, channel, commitment_scheme)
    // TODO(Gali): Convert to circuit proof.
}
