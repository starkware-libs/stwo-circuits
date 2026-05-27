use circuit_common::{
    finalize::{ComponentSizes, pad_to_targets},
    preprocessed::PreprocessedCircuit,
};
use circuit_verifier::{
    statement::{INTERACTION_POW_BITS, all_circuit_components},
    verify::CircuitConfig,
};
use circuits::{blake::HashValue, context::Context, ivalue::NoValue};
use circuits_stark_verifier::proof::{ProofConfig, empty_proof};
use stwo::core::{fields::qm31::QM31, fri::FriConfig, pcs::PcsConfig};

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
) -> (PreprocessedCircuit, Context<NoValue>) {
    let all_circuit_components = &all_circuit_components::<NoValue>();
    let proof_config = ProofConfig::new(
        all_circuit_components,
        vec![true; all_circuit_components.len()],
        preprocessed_leaf_circuit.preprocessed_trace.n_columns(),
        &pcs_config,
        INTERACTION_POW_BITS,
    );
    let subcircuit_config = CircuitConfig {
        config: pcs_config,
        n_outputs: preprocessed_leaf_circuit.params.n_outputs,
        preprocessed_column_log_sizes: preprocessed_leaf_circuit.preprocessed_trace.log_sizes(),
        preprocessed_root: HashValue(QM31::from(0), QM31::from(0)),
    };
    let empty_input = || MultiverifierInput {
        proof: empty_proof(&proof_config),
        preprocessed_root: subcircuit_config.preprocessed_root,
        output_values: [QM31::from(0); 2],
    };
    let shared_config = SharedConfig {
        pcs_config,
        proof_config: proof_config.clone(),
        preprocessed_column_log_sizes: subcircuit_config.preprocessed_column_log_sizes.clone(),
        n_outputs: subcircuit_config.n_outputs,
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
#[expect(dead_code)]
pub fn get_preprocessed_root(
    preprocessed_circuit: &PreprocessedCircuit,
    log_blowup_factor: u32,
) -> HashValue<QM31> {
    let lifting_log_size = preprocessed_circuit.params.trace_log_size + log_blowup_factor;
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

/// Given a trace log size and a log blowup factor, computes the pcs config used by the privacy flow
/// (see [`circuit_cairo_verifier::privacy::privacy_cairo_verifier_config`]).
pub const fn get_pcs_config(trace_log_size: u32, log_blowup_factor: u32) -> PcsConfig {
    let (pow_bits, n_queries) = match log_blowup_factor {
        1 => (26, 70),
        2 => (26, 35),
        3 => (27, 23),
        _ => panic!("Unsupported log blowup factor."),
    };
    let fri_config =
        FriConfig { log_blowup_factor, log_last_layer_degree_bound: 0, n_queries, fold_step: 4 };
    PcsConfig { pow_bits, fri_config, lifting_log_size: Some(trace_log_size + log_blowup_factor) }
}
