use crate::circuit_air::components::CircuitComponents;
use crate::circuit_air::statement::INTERACTION_POW_BITS;
use crate::circuit_air::{
    CircuitClaim, CircuitInteractionClaim, CircuitInteractionElements, lookup_sum,
};
use crate::circuit_prover::finalize::finalize_context;
use crate::circuit_prover::witness::components::qm31_ops;
use crate::circuit_prover::witness::preprocessed::PreProcessedTrace;
use crate::circuit_prover::witness::trace::TraceGenerator;
use crate::circuit_prover::witness::trace::write_interaction_trace;
use crate::circuit_prover::witness::trace::write_trace;
use crate::circuits::context::Context;
use num_traits::Zero;
use stwo::core::air::Component;
use stwo::core::channel::Blake2sM31Channel;
use stwo::core::channel::Channel;
use stwo::core::fields::qm31::QM31;
use stwo::core::fields::qm31::SecureField;
use stwo::core::pcs::PcsConfig;
use stwo::core::poly::circle::CanonicCoset;
use stwo::core::proof::ExtendedStarkProof;
use stwo::core::proof_of_work::GrindOps;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;
use stwo::prover::CommitmentSchemeProver;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::poly::circle::PolyOps;
use stwo::prover::{ProvingError, prove_ex};

const COMPOSITION_POLYNOMIAL_LOG_DEGREE_BOUND: u32 = 1;

pub struct CircuitParams {
    pub trace_log_size: u32,
    pub first_permutation_row: usize,
}

pub struct CircuitProof {
    pub pcs_config: PcsConfig,
    pub claim: CircuitClaim,
    pub interaction_pow_nonce: u64,
    pub interaction_claim: CircuitInteractionClaim,
    pub components: Vec<Box<dyn Component>>,
    pub stark_proof: Result<ExtendedStarkProof<Blake2sM31MerkleHasher>, ProvingError>,
    pub channel_salt: u32,
}

#[cfg(test)]
#[path = "prover_test.rs"]
pub mod test;

pub fn prove_circuit(context: &mut Context<QM31>) -> CircuitProof {
    finalize_context(context);

    let (preprocessed_trace, params) =
        PreProcessedTrace::generate_preprocessed_trace(&context.circuit);

    prove_circuit_assignment(context.values(), preprocessed_trace, &params)
}

pub fn prove_circuit_assignment(
    values: &[QM31],
    preprocessed_trace: PreProcessedTrace,
    params: &CircuitParams,
) -> CircuitProof {
    let trace_generator = TraceGenerator {
        qm31_ops_trace_generator: qm31_ops::TraceGenerator {
            first_permutation_row: params.first_permutation_row,
        },
    };
    let trace_log_size = params.trace_log_size;

    let mut pcs_config = PcsConfig::default();
    let lifting_log_size = trace_log_size + pcs_config.fri_config.log_blowup_factor;

    pcs_config.lifting_log_size = Some(lifting_log_size);

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

    // Mix channel salt. Note that we first reduce it modulo `M31::P`, then cast it as QM31.
    let channel_salt = 0_u32;
    channel.mix_felts(&[channel_salt.into()]);
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
        write_trace(values, &preprocessed_trace, &mut tree_builder, &trace_generator);
    claim.mix_into(channel);
    tree_builder.commit(channel);

    // Draw interaction elements.
    let interaction_pow_nonce = SimdBackend::grind(channel, INTERACTION_POW_BITS);
    channel.mix_u64(interaction_pow_nonce);
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
    let component_builder = CircuitComponents::new(
        &claim,
        &interaction_elements,
        &interaction_claim,
        &preprocessed_trace.ids(),
    );

    let components = component_builder.provers();

    // Prove stark.
    let proof = prove_ex::<SimdBackend, _>(&components, channel, commitment_scheme, true);
    CircuitProof {
        pcs_config,
        claim,
        interaction_pow_nonce,
        interaction_claim,
        components: component_builder.components(),
        stark_proof: proof,
        channel_salt,
    }
}
