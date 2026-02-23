use crate::finalize::finalize_context;
use crate::witness::components::qm31_ops;
use crate::witness::preprocessed::PreprocessedCircuit;
use crate::witness::trace::TraceGenerator;
use crate::witness::trace::write_interaction_trace;
use crate::witness::trace::write_trace;
use circuit_air::components::CircuitComponents;
use circuit_air::statement::INTERACTION_POW_BITS;
use circuit_air::{CircuitClaim, CircuitInteractionClaim, CircuitInteractionElements, lookup_sum};
use circuits::context::Context;
use itertools::chain;
use num_traits::Zero;
use stwo::core::air::Component;
use stwo::core::channel::Blake2sM31Channel;
use stwo::core::channel::Channel;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo::core::poly::circle::CanonicCoset;
use stwo::core::proof::ExtendedStarkProof;
use stwo::core::proof_of_work::GrindOps;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;
use stwo::prover::CommitmentSchemeProver;
use stwo::prover::ComponentProver;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::poly::circle::PolyOps;
use stwo::prover::{ProvingError, prove_ex};

const COMPOSITION_POLYNOMIAL_LOG_DEGREE_BOUND: u32 = 1;

pub struct CircuitParams {
    pub trace_log_size: u32,
    pub first_permutation_row: usize,
    pub n_blake_gates: usize,
    pub output_addresses: Vec<usize>,
}

pub struct CircuitProof {
    pub pcs_config: PcsConfig,
    pub preprocessed_circuit: PreprocessedCircuit,
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

pub fn to_component_provers(
    components: &CircuitComponents,
) -> Vec<&dyn ComponentProver<SimdBackend>> {
    chain!([
        &components.eq as &dyn ComponentProver<SimdBackend>,
        &components.qm31_ops as &dyn ComponentProver<SimdBackend>,
        &components.blake_gate as &dyn ComponentProver<SimdBackend>,
        &components.blake_round as &dyn ComponentProver<SimdBackend>,
        &components.blake_round_sigma as &dyn ComponentProver<SimdBackend>,
        &components.blake_g as &dyn ComponentProver<SimdBackend>,
        &components.blake_output as &dyn ComponentProver<SimdBackend>,
        &components.triple_xor_32 as &dyn ComponentProver<SimdBackend>,
        &components.verify_bitwise_xor_8 as &dyn ComponentProver<SimdBackend>,
        &components.verify_bitwise_xor_12 as &dyn ComponentProver<SimdBackend>,
        &components.verify_bitwise_xor_4 as &dyn ComponentProver<SimdBackend>,
        &components.verify_bitwise_xor_7 as &dyn ComponentProver<SimdBackend>,
        &components.verify_bitwise_xor_9 as &dyn ComponentProver<SimdBackend>,
        &components.range_check_15 as &dyn ComponentProver<SimdBackend>,
        &components.range_check_16 as &dyn ComponentProver<SimdBackend>,
    ])
    .collect()
}

pub fn prove_circuit(context: &mut Context<QM31>) -> CircuitProof {
    finalize_context(context);

    let preprocessed_circuit = PreprocessedCircuit::preprocess_circuit(&context.circuit);
    let context_values = context.values();

    prove_circuit_assignment(context_values, preprocessed_circuit)
}

pub fn prove_circuit_assignment(
    values: &[QM31],
    preprocessed_circuit: PreprocessedCircuit,
) -> CircuitProof {
    let preprocessed_trace = preprocessed_circuit.preprocessed_trace.clone();
    let params = &preprocessed_circuit.params;
    let CircuitParams {
        trace_log_size,
        first_permutation_row,
        n_blake_gates,
        output_addresses,
        ..
    } = params;
    let trace_generator = TraceGenerator {
        qm31_ops_trace_generator: qm31_ops::TraceGenerator {
            first_permutation_row: *first_permutation_row,
        },
    };

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
    let (claim, interaction_generator) = write_trace(
        values,
        preprocessed_trace.clone(),
        output_addresses,
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

    // Validate lookup argument.
    assert_eq!(
        lookup_sum(
            &claim,
            &interaction_claim,
            &interaction_elements,
            output_addresses,
            *n_blake_gates
        ),
        QM31::zero()
    );

    interaction_claim.mix_into(channel);
    tree_builder.commit(channel);
    // Component provers.
    let circuit_components = CircuitComponents::new(
        &claim,
        &interaction_elements,
        &interaction_claim,
        &preprocessed_trace.ids(),
    );
    let components = to_component_provers(&circuit_components);

    // Prove stark.
    let proof = prove_ex::<SimdBackend, _>(&components, channel, commitment_scheme, true);
    CircuitProof {
        pcs_config,
        preprocessed_circuit,
        claim,
        interaction_pow_nonce,
        interaction_claim,
        components: circuit_components.components(),
        stark_proof: proof,
        channel_salt,
    }
}
