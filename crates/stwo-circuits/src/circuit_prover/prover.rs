use std::sync::Arc;

use crate::circuit_air::components::CircuitComponents;
use crate::circuit_air::{
    CircuitClaim, CircuitInteractionClaim, CircuitInteractionElements, lookup_sum,
};
use crate::circuit_prover::finalize::finalize_context;
use crate::circuit_prover::witness::preprocessed::PreProcessedTrace;
use crate::circuit_prover::witness::trace::write_interaction_trace;
use crate::circuit_prover::witness::trace::write_trace;
use crate::circuits::context::Context;
use num_traits::Zero;
use stwo::core::air::Component;
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
use stwo::prover::poly::circle::PolyOps;
use stwo::prover::{ProvingError, prove_ex};

const COMPOSITION_POLYNOMIAL_LOG_DEGREE_BOUND: u32 = 1;
pub struct CircuitProof {
    pub claim: CircuitClaim,
    pub interaction_claim: CircuitInteractionClaim,
    pub components: Vec<Box<dyn Component>>,
    pub pcs_config: PcsConfig,
    pub stark_proof: Result<ExtendedStarkProof<Blake2sM31MerkleHasher>, ProvingError>,
}

#[cfg(test)]
#[path = "prover_test.rs"]
pub mod test;

pub fn prove_circuit(context: &mut Context<QM31>) -> CircuitProof {
    finalize_context(context);
    let pcs_config = PcsConfig::default();
    // Blake gate preprocessed trace.
    // After creating this preprocessed trace and the next one, we need to sort the columns by size
    // and then commit to them. Generate preprocessed trace.
    let (preprocessed_trace, trace_generator) =
        PreProcessedTrace::generate_preprocessed_trace(&context.circuit);

    for (i, col) in preprocessed_trace.columns.iter().enumerate() {

        eprintln!("Preprocessed col index: {}, log size {}, is power of two: {}", i, col.len(), col.len().is_power_of_two());

    }
    
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
    let mut commitment_scheme =
        CommitmentSchemeProver::<SimdBackend, Blake2sM31MerkleChannel>::new(pcs_config, &twiddles);

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
    // debug_assert_eq!(lookup_sum(&interaction_claim), SecureField::zero());

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
    let proof = prove_ex::<SimdBackend, _>(&components, channel, commitment_scheme);

    CircuitProof {
        components: component_builder.components(),
        claim,
        interaction_claim,
        pcs_config,
        stark_proof: proof,
    }
}
