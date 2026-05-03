use crate::circuit_air::circuit_components::CircuitComponents;
use crate::witness::trace::TraceGenerator;
use crate::witness::trace::write_interaction_trace;
use crate::witness::trace::write_trace;
use circuit_common::CircuitParams;
use circuit_common::Qm31OpsTraceGenerator;
use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_verifier::circuit_claim::{
    CircuitClaim, CircuitInteractionClaim, CircuitInteractionElements, lookup_sum,
};
use circuit_verifier::statement::{INTERACTION_POW_BITS, component_log_sizes};
use circuit_verifier::verify::CircuitPublicData;
use circuits_stark_verifier::proof::Proof;
use circuits_stark_verifier::proof::ProofConfig;
use circuits_stark_verifier::proof_from_stark_proof::proof_from_stark_proof;
use itertools::chain;
use num_traits::Zero;
use stwo::core::air::Component;
use stwo::core::channel::{Channel, MerkleChannel};
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo::core::poly::circle::CanonicCoset;
use stwo::core::proof::ExtendedStarkProof;
use stwo::core::proof_of_work::GrindOps;
use stwo::core::utils::MaybeOwned;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;
use stwo::core::vcs_lifted::merkle_hasher::MerkleHasherLifted;
use stwo::prover::CommitmentSchemeProver;
use stwo::prover::CommitmentTreeProver;
use stwo::prover::ComponentProver;
pub use stwo::prover::backend::simd::SimdBackend;
pub use stwo::prover::mempool::BaseColumnPool;
use stwo::prover::poly::circle::PolyOps;
use stwo::prover::poly::twiddles::TwiddleTree;
use stwo::prover::{ProvingError, prove_ex};

const COMPOSITION_POLYNOMIAL_LOG_DEGREE_BOUND: u32 = 1;

pub struct CircuitProof<H: MerkleHasherLifted> {
    pub pcs_config: PcsConfig,
    pub claim: CircuitClaim,
    pub interaction_pow_nonce: u64,
    pub interaction_claim: CircuitInteractionClaim,
    pub components: Vec<Box<dyn Component>>,
    pub stark_proof: ExtendedStarkProof<H>,
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
        &components.m_31_to_u_32 as &dyn ComponentProver<SimdBackend>,
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

pub fn prove_circuit_assignment(
    values: &[QM31],
    preprocessed_circuit: &PreprocessedCircuit,
    base_column_pool: &BaseColumnPool<SimdBackend>,
    pcs_config: PcsConfig,
) -> Result<CircuitProof<Blake2sM31MerkleHasher>, ProvingError> {
    prove_circuit_assignment_with_channel::<Blake2sM31MerkleChannel>(
        values,
        preprocessed_circuit,
        base_column_pool,
        pcs_config,
    )
}

pub fn prove_circuit_assignment_with_channel<MC>(
    values: &[QM31],
    preprocessed_circuit: &PreprocessedCircuit,
    base_column_pool: &BaseColumnPool<SimdBackend>,
    pcs_config: PcsConfig,
) -> Result<CircuitProof<MC::H>, ProvingError>
where
    MC: MerkleChannel,
    SimdBackend: stwo::prover::backend::BackendForChannel<MC>,
{
    let trace_log_size = preprocessed_circuit.params.trace_log_size;
    let lifting_log_size = trace_log_size + pcs_config.fri_config.log_blowup_factor;
    let pcs_config = PcsConfig { lifting_log_size: Some(lifting_log_size), ..pcs_config };

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

    let preprocessed_trace = preprocessed_circuit.preprocessed_trace.get_trace::<SimdBackend>();
    let preprocessed_trace_polys = SimdBackend::interpolate_columns(preprocessed_trace, &twiddles);

    let store_polynomials_coefficients = true;
    let preprocessed_tree = CommitmentTreeProver::<SimdBackend, MC>::new(
        preprocessed_trace_polys,
        pcs_config.fri_config.log_blowup_factor,
        &twiddles,
        store_polynomials_coefficients,
        pcs_config.lifting_log_size,
        base_column_pool,
    );

    prove_circuit_with_precompute::<MC>(
        base_column_pool,
        &twiddles,
        preprocessed_circuit,
        MaybeOwned::Owned(preprocessed_tree),
        values,
        pcs_config,
    )
}

pub fn prove_circuit_with_precompute<'a, MC>(
    base_column_pool: &BaseColumnPool<SimdBackend>,
    twiddles: &TwiddleTree<SimdBackend>,
    preprocessed_circuit: &PreprocessedCircuit,
    preprocessed_tree: MaybeOwned<'a, CommitmentTreeProver<SimdBackend, MC>>,
    values: &[QM31],
    pcs_config: PcsConfig,
) -> Result<CircuitProof<MC::H>, ProvingError>
where
    MC: MerkleChannel,
    SimdBackend: stwo::prover::backend::BackendForChannel<MC>,
{
    let PreprocessedCircuit { preprocessed_trace, params } = preprocessed_circuit;
    let CircuitParams {
        first_permutation_row,
        n_blake_gates,
        n_blake_compress,
        output_addresses,
        ..
    } = params;
    let trace_generator = TraceGenerator {
        qm31_ops_trace_generator: Qm31OpsTraceGenerator {
            first_permutation_row: *first_permutation_row,
        },
    };

    // Setup protocol.
    let channel = &mut MC::C::default();

    // Mix channel salt. Note that we first reduce it modulo `M31::P`, then cast it as QM31.
    let channel_salt = 0_u32;
    channel.mix_felts(&[channel_salt.into()]);
    pcs_config.mix_into(channel);
    let mut commitment_scheme = CommitmentSchemeProver::<SimdBackend, MC>::with_memory_pool(
        pcs_config,
        twiddles,
        base_column_pool,
    );

    commitment_scheme.set_store_polynomials_coefficients();

    // Preprocessed trace.
    commitment_scheme.commit_tree(preprocessed_tree, channel);

    // Base trace.
    let mut tree_builder = commitment_scheme.tree_builder();
    let (claim, interaction_generator) = write_trace(
        values,
        preprocessed_trace.clone(),
        output_addresses,
        &mut tree_builder,
        &trace_generator,
        twiddles,
    );

    let expected_log_sizes = component_log_sizes(
        *n_blake_compress,
        &preprocessed_trace.ids(),
        &preprocessed_trace.log_sizes(),
    );
    assert_eq!(claim.log_sizes, expected_log_sizes);
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
        twiddles,
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
    let stark_proof = prove_ex::<SimdBackend, _>(&components, channel, commitment_scheme, true)?;
    Ok(CircuitProof {
        pcs_config,
        claim,
        interaction_pow_nonce,
        interaction_claim,
        components: circuit_components.components(),
        stark_proof,
        channel_salt,
    })
}

pub fn prepare_circuit_proof_for_circuit_verifier(
    circuit_proof: CircuitProof<Blake2sM31MerkleHasher>,
    proof_config: &ProofConfig,
) -> (Proof<QM31>, CircuitPublicData) {
    let CircuitProof {
        pcs_config: _,
        claim,
        interaction_pow_nonce,
        interaction_claim,
        components: _,
        stark_proof,
        channel_salt,
    } = circuit_proof;

    let public_data = CircuitPublicData { output_values: claim.output_values.clone() };

    let claimed_sums = interaction_claim.claimed_sums.to_vec();

    let proof = proof_from_stark_proof(
        &stark_proof,
        proof_config,
        claimed_sums,
        interaction_pow_nonce,
        channel_salt,
    );
    (proof, public_data)
}
