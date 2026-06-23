use circuit_common::{
    finalize::{ComponentSizes, pad_to_targets},
    preprocessed::PreprocessedCircuit,
};
use circuit_verifier::{
    statement::{INTERACTION_POW_BITS, all_circuit_components},
    verify::CircuitConfig,
};
use circuits::{blake::HashValue, context::FinalizedContext, ivalue::NoValue};
use circuits_stark_verifier::proof::{ProofConfig, empty_proof};
use stwo::core::{fields::qm31::QM31, pcs::PcsConfig};

use crate::verify::{MultiverifierInput, SharedConfig, build_multiverifier_circuit};
use stwo::core::poly::circle::CanonicCoset;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::prover::CommitmentTreeProver;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::mempool::BaseColumnPool;
use stwo::prover::poly::circle::PolyOps;

/// Builds a `NoValue` multiverifier and preprocesses it. The multiverifier is built by feeding it
/// two identical proofs of a circuit.
pub fn get_preprocessed_multiverifier_from_circuit(
    preprocessed_leaf_circuit: &PreprocessedCircuit,
    pcs_config: PcsConfig,
    target_padding: Option<ComponentSizes>,
) -> (PreprocessedCircuit, FinalizedContext<NoValue>) {
    assert_eq!(
        pcs_config.lifting_log_size.unwrap(),
        preprocessed_leaf_circuit.trace_log_size + pcs_config.fri_config.log_blowup_factor
    );
    let all_circuit_components = &all_circuit_components::<NoValue>();
    let proof_config = ProofConfig::new(
        all_circuit_components,
        preprocessed_leaf_circuit.preprocessed_trace.n_columns(),
        &pcs_config,
        INTERACTION_POW_BITS,
    );
    let subcircuit_config = CircuitConfig {
        config: pcs_config,
        n_outputs: preprocessed_leaf_circuit.n_outputs,
        preprocessed_column_log_sizes: preprocessed_leaf_circuit.preprocessed_trace.log_sizes(),
        preprocessed_root: HashValue::from([0u32; 8]),
    };
    let shared_config = SharedConfig {
        pcs_config,
        proof_config: proof_config.clone(),
        preprocessed_column_log_sizes: subcircuit_config.preprocessed_column_log_sizes.clone(),
    };
    let empty_input = || MultiverifierInput {
        proof: empty_proof(&proof_config),
        preprocessed_root: subcircuit_config.preprocessed_root.clone(),
        output_values: [QM31::from(0); 2],
    };
    let mut multiverifier_context =
        build_multiverifier_circuit::<NoValue>(empty_input(), empty_input(), &shared_config);
    if let Some(target_padding) = target_padding {
        pad_to_targets(&mut multiverifier_context, target_padding);
    }
    let preprocessed_multiverifier_circuit =
        PreprocessedCircuit::preprocess_circuit(&mut multiverifier_context);
    (preprocessed_multiverifier_circuit, multiverifier_context)
}

/// Computes the Merkle root of a circuit's preprocessed trace.
pub fn get_preprocessed_root(
    preprocessed_circuit: &PreprocessedCircuit,
    log_blowup_factor: u32,
) -> HashValue<QM31> {
    let lifting_log_size = preprocessed_circuit.trace_log_size + log_blowup_factor;
    let twiddles = SimdBackend::precompute_twiddles(
        CanonicCoset::new(lifting_log_size).circle_domain().half_coset,
    );
    let preprocessed_trace = preprocessed_circuit.preprocessed_trace.get_trace::<SimdBackend>();
    let preprocessed_trace_polys = SimdBackend::interpolate_columns(preprocessed_trace, &twiddles);
    let preprocessed_tree = CommitmentTreeProver::<SimdBackend, Blake2sM31MerkleChannel>::new(
        preprocessed_trace_polys,
        log_blowup_factor,
        &twiddles,
        true,
        Some(lifting_log_size),
        &BaseColumnPool::<SimdBackend>::new(),
    );
    preprocessed_tree.commitment.root().into()
}
