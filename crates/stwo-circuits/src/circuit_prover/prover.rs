use std::sync::Arc;

use crate::circuit_air::components::CircuitComponents;
use crate::circuit_air::relations::CommonLookupElements;
use crate::circuit_air::statement::INTERACTION_POW_BITS;
use crate::circuit_air::{
    CircuitClaim, CircuitInteractionClaim, CircuitInteractionElements, lookup_sum,
};
use crate::circuit_prover::finalize::finalize_context;
use crate::circuit_prover::witness::components::blake_gate::blake2s_initial_state;
use crate::circuit_prover::witness::preprocessed::PreProcessedTrace;
use crate::circuit_prover::witness::trace::write_interaction_trace;
use crate::circuit_prover::witness::trace::write_trace;
use crate::circuits::context::Context;
use num_traits::Zero;
use stwo::core::air::Component;
use stwo::core::channel::Blake2sM31Channel;
use stwo::core::channel::Channel;
use stwo::core::fields::FieldExpOps;
use stwo::core::fields::m31::M31;
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
use stwo_constraint_framework::Relation;

const COMPOSITION_POLYNOMIAL_LOG_DEGREE_BOUND: u32 = 1;
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

fn blake_iv_public_logup_sum(
    context: &Context<QM31>,
    common_lookup_elements: &CommonLookupElements,
) -> SecureField {
    let state_id = M31::from(1061955672);
    let initial_state = blake2s_initial_state();
    let initial_state_limbs = [
        M31::from(initial_state[0] & 0xffff),
        M31::from((initial_state[0] >> 16) & 0xffff),
        M31::from(initial_state[1] & 0xffff),
        M31::from((initial_state[1] >> 16) & 0xffff),
        M31::from(initial_state[2] & 0xffff),
        M31::from((initial_state[2] >> 16) & 0xffff),
        M31::from(initial_state[3] & 0xffff),
        M31::from((initial_state[3] >> 16) & 0xffff),
        M31::from(initial_state[4] & 0xffff),
        M31::from((initial_state[4] >> 16) & 0xffff),
        M31::from(initial_state[5] & 0xffff),
        M31::from((initial_state[5] >> 16) & 0xffff),
        M31::from(initial_state[6] & 0xffff),
        M31::from((initial_state[6] >> 16) & 0xffff),
        M31::from(initial_state[7] & 0xffff),
        M31::from((initial_state[7] >> 16) & 0xffff),
    ];

    let limbs = [
        state_id,
        M31::from(0u32),
        initial_state_limbs[0],
        initial_state_limbs[1],
        initial_state_limbs[2],
        initial_state_limbs[3],
        initial_state_limbs[4],
        initial_state_limbs[5],
        initial_state_limbs[6],
        initial_state_limbs[7],
        initial_state_limbs[8],
        initial_state_limbs[9],
        initial_state_limbs[10],
        initial_state_limbs[11],
        initial_state_limbs[12],
        initial_state_limbs[13],
        initial_state_limbs[14],
        initial_state_limbs[15],
    ];
    let denom: SecureField = common_lookup_elements.combine(&limbs);
    denom.inverse() * M31::from(context.circuit.blake.len())
}

pub fn prove_circuit(context: &mut Context<QM31>) -> (CircuitProof, Vec<u32>) {
    finalize_context(context);
    // Generate preprocessed trace.
    let (preprocessed_trace, trace_generator) =
        PreProcessedTrace::generate_preprocessed_trace(&context.circuit);

    // The trace size is the size of the largest column in the preprocessed trace (since all
    // components have preprocessed columns).
    let preprocessed_trace_sizes = preprocessed_trace.log_sizes();
    let trace_log_size = preprocessed_trace_sizes.iter().copied().max().unwrap();

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

    commitment_scheme.set_store_polynomials_coefficients();

    // Preprocessed trace.
    let mut tree_builder = commitment_scheme.tree_builder();
    tree_builder.extend_evals(preprocessed_trace.get_trace::<SimdBackend>());
    tree_builder.commit(channel);

    // Base trace.
    let mut tree_builder = commitment_scheme.tree_builder();
    let preprocessed_trace_arc = Arc::new(preprocessed_trace);
    let (claim, interaction_generator) = write_trace(
        context.values(),
        preprocessed_trace_arc.clone(),
        &mut tree_builder,
        &trace_generator,
    );
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
    let public_logup_sum =
        blake_iv_public_logup_sum(context, &interaction_elements.common_lookup_elements);
    assert_eq!(lookup_sum(&interaction_claim) - public_logup_sum, SecureField::zero());

    interaction_claim.mix_into(channel);
    tree_builder.commit(channel);
    // Component provers.
    let component_builder = CircuitComponents::new(
        &claim,
        &interaction_elements,
        &interaction_claim,
        &preprocessed_trace_arc.ids(),
    );
    let components = component_builder.provers();
    // Prove stark.
    let proof = prove_ex::<SimdBackend, _>(&components, channel, commitment_scheme, true);
    (
        CircuitProof {
            pcs_config,
            claim,
            interaction_pow_nonce,
            interaction_claim,
            components: component_builder.components(),
            stark_proof: proof,
            channel_salt,
        },
        preprocessed_trace_sizes,
    )
}
