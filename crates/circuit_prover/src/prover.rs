use crate::circuit_air::circuit_components::CircuitComponents;
use crate::witness::trace::TraceGenerator;
use crate::witness::trace::write_interaction_trace;
use crate::witness::trace::write_trace;
use circuit_common::Qm31OpsTraceGenerator;
use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_verifier::circuit_claim::{CircuitInteractionElements, lookup_sum};
pub use circuit_verifier::circuit_proof::CircuitProof;
use circuit_verifier::statement::INTERACTION_POW_BITS;
use circuit_verifier::statement::all_circuit_components;
use circuit_verifier::verify::CircuitPublicData;
use circuits_stark_verifier::proof::Proof;
use circuits_stark_verifier::proof::ProofConfig;
use circuits_stark_verifier::proof_from_stark_proof::proof_from_stark_proof;
use num_traits::Zero;
use stwo::core::channel::{Channel, MerkleChannel};
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo::core::poly::circle::CanonicCoset;
use stwo::core::proof_of_work::GrindOps;
use stwo::core::utils::MaybeOwned;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;
use stwo::prover::CommitmentSchemeProver;
use stwo::prover::CommitmentTreeProver;
pub use stwo::prover::backend::simd::SimdBackend;
pub use stwo::prover::mempool::BaseColumnPool;
use stwo::prover::poly::circle::PolyOps;
use stwo::prover::poly::twiddles::TwiddleTree;
use stwo::prover::{ProvingError, prove_ex};
use stwo_constraint_framework::PREPROCESSED_TRACE_IDX;

const COMPOSITION_POLYNOMIAL_LOG_DEGREE_BOUND: u32 = 1;

#[cfg(test)]
#[path = "prover_test.rs"]
pub mod test;

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
    let trace_log_size = preprocessed_circuit.trace_log_size;
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
    let PreprocessedCircuit {
        preprocessed_trace,
        first_permutation_row,
        n_outputs,
        trace_log_size: _,
    } = preprocessed_circuit;
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
    let (claim, component_log_sizes, interaction_generator) = write_trace(
        values,
        preprocessed_trace.clone(),
        *n_outputs,
        &mut tree_builder,
        &trace_generator,
        twiddles,
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
        &component_log_sizes,
        interaction_generator,
        &mut tree_builder,
        &interaction_elements,
        twiddles,
    );

    // Validate lookup argument.
    assert_eq!(lookup_sum(&claim, &interaction_claim, &interaction_elements), QM31::zero());

    interaction_claim.mix_into(channel);
    tree_builder.commit(channel);
    // Component provers.
    let circuit_components = CircuitComponents::new(
        &interaction_elements,
        &interaction_claim,
        &component_log_sizes,
        &preprocessed_trace.ids(),
    );
    let components = circuit_components.component_provers();

    // Prove stark.
    let stark_proof = prove_ex::<SimdBackend, _>(&components, channel, commitment_scheme, true)?;
    Ok(CircuitProof {
        pcs_config,
        claim,
        interaction_pow_nonce,
        interaction_claim,
        stark_proof,
        channel_salt,
    })
}

pub fn prepare_circuit_proof_for_circuit_verifier(
    circuit_proof: CircuitProof<Blake2sM31MerkleHasher>,
) -> (Proof<QM31>, CircuitPublicData) {
    let CircuitProof {
        pcs_config,
        claim,
        interaction_pow_nonce,
        interaction_claim,
        stark_proof,
        channel_salt,
    } = circuit_proof;

    let public_data = CircuitPublicData { output_values: claim.output_values.clone() };

    let proof_config = ProofConfig::new(
        &all_circuit_components::<QM31>(),
        stark_proof.proof.sampled_values[PREPROCESSED_TRACE_IDX].len(),
        &pcs_config,
        INTERACTION_POW_BITS,
    );

    // The claimed sums are already stored in size-sorted component order (see
    // `write_interaction_trace`), matching the order in which the verifier consumes them and the
    // order in which they are mixed into the channel.
    let claimed_sums = interaction_claim.claimed_sums.to_vec();

    let proof = proof_from_stark_proof(
        &stark_proof,
        &proof_config,
        claimed_sums,
        interaction_pow_nonce,
        channel_salt,
    );
    (proof, public_data)
}
