use blake2::{Blake2s256, Digest};
use circuit_cairo_verifier::privacy::get_pcs_config;
use circuit_common::{
    N_RESERVED,
    finalize::{ComponentSizes, pad_to_targets},
    preprocessed::PreprocessedCircuit,
};
use circuit_verifier::{
    statement::{INTERACTION_POW_BITS, all_circuit_components, circuit_component_log_sizes},
    verify::CircuitConfig,
};
use circuits::{blake::HashValue, context::FinalizedContext, ivalue::NoValue};
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use circuits_stark_verifier::proof::{ProofConfig, empty_proof};
use stwo::core::{fields::qm31::QM31, pcs::PcsConfig};
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::verify::{MultiverifierInput, SharedConfig, build_multiverifier_circuit};
use stwo::core::poly::circle::CanonicCoset;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::prover::CommitmentTreeProver;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::mempool::BaseColumnPool;
use stwo::prover::poly::circle::PolyOps;

// Shared test fixtures: config constants, target padding sizes, and helpers used by the
// multiverifier test modules (`verify_test` and `backward_compatibility_test`).

const PRIVACY_CAIRO_VERIFIER_TRACE_LOG_SIZE: u32 = 21;
pub const LOG_BLOWUP_FACTOR: u32 = 3;
pub const PCS_CONFIG: PcsConfig =
    get_pcs_config(PRIVACY_CAIRO_VERIFIER_TRACE_LOG_SIZE, LOG_BLOWUP_FACTOR);
pub const TARGET_PADDING_SIZES: ComponentSizes = ComponentSizes {
    eq: 1 << 17,
    qm31_ops: 1 << 21,
    m31_to_u32: 1 << 18,
    triple_xor: 1 << 17,
    blake_g_gate: 1 << 20,
};
/// The number of preprocessed columns in a trace of a circuit.
pub const CIRCUIT_N_PREPROCESSED_COLUMNS: usize = 45;
/// The Blake2s digest of the output in the privacy Cairo verifier proof fixture
/// (`test_data/circuit_multiverifier/proof_cairo.bin`).
pub const PRIVACY_CAIRO_VERIFIER_OUTPUT_VALUES: [u32; 8] =
    [3035180123, 3555538090, 587798257, 1881776298, 3385462846, 2102605012, 3369268656, 403460632];

/// The preprocessed root of the privacy Cairo verifier circuit.
pub const PRIVACY_CAIRO_VERIFIER_PREPROCESSED_ROOT: [u32; 8] =
    [695775592, 831947430, 3864682957, 1778749033, 1073148880, 3248306553, 1968525874, 3767461582];

/// The preprocessed root of the multiverifier circuit.
pub const MULTIVERIFIER_PREPROCESSED_ROOT: [u32; 8] =
    [933544165, 1273803072, 152539421, 3171454012, 281718454, 2434086603, 4253372773, 2721160983];
/// A multiverifier proof verifying two identical Cairo verifier proofs.
pub const MULTIVERIFIER_OF_TWO_CAIRO_PROOFS_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../test_data/circuit_multiverifier/proof.bin");

/// Out-of-circuit implementation of [`circuits::blake::blake2s_u32s`].
pub fn blake2s_u32s_host(words: &[u32]) -> [u32; 8] {
    let mut hasher = Blake2s256::new();
    for word in words {
        hasher.update(word.to_le_bytes());
    }
    let hash: [u8; 32] = hasher.finalize().into();
    std::array::from_fn(|i| u32::from_le_bytes(hash[i * 4..i * 4 + 4].try_into().unwrap()))
}

pub fn multiverifier_preprocessed_column_log_sizes() -> OrderedHashMap<PreProcessedColumnId, u32> {
    [
        ("bitwise_xor_4_0", 8),
        ("bitwise_xor_4_1", 8),
        ("bitwise_xor_4_2", 8),
        ("bitwise_xor_7_0", 14),
        ("bitwise_xor_7_1", 14),
        ("bitwise_xor_7_2", 14),
        ("seq_16", 16),
        ("bitwise_xor_8_0", 16),
        ("bitwise_xor_8_1", 16),
        ("bitwise_xor_8_2", 16),
        ("eq_in0_address", 17),
        ("eq_in1_address", 17),
        ("triple_xor_input_addr_0", 17),
        ("triple_xor_input_addr_1", 17),
        ("triple_xor_input_addr_2", 17),
        ("triple_xor_output_addr", 17),
        ("triple_xor_multiplicity", 17),
        ("m31_to_u32_input_addr", 18),
        ("m31_to_u32_output_addr", 18),
        ("m31_to_u32_multiplicity", 18),
        ("bitwise_xor_9_0", 18),
        ("bitwise_xor_9_1", 18),
        ("bitwise_xor_9_2", 18),
        ("blake_g_gate_input_addr_a", 20),
        ("blake_g_gate_input_addr_b", 20),
        ("blake_g_gate_input_addr_c", 20),
        ("blake_g_gate_input_addr_d", 20),
        ("blake_g_gate_input_addr_f0", 20),
        ("blake_g_gate_input_addr_f1", 20),
        ("blake_g_gate_output_addr_a", 20),
        ("blake_g_gate_output_addr_b", 20),
        ("blake_g_gate_output_addr_c", 20),
        ("blake_g_gate_output_addr_d", 20),
        ("blake_g_gate_multiplicity", 20),
        ("bitwise_xor_10_0", 20),
        ("bitwise_xor_10_1", 20),
        ("bitwise_xor_10_2", 20),
        ("qm31_ops_add_flag", 21),
        ("qm31_ops_sub_flag", 21),
        ("qm31_ops_mul_flag", 21),
        ("qm31_ops_pointwise_mul_flag", 21),
        ("qm31_ops_in0_address", 21),
        ("qm31_ops_in1_address", 21),
        ("qm31_ops_out_address", 21),
        ("qm31_ops_mults", 21),
    ]
    .into_iter()
    .map(|(id, log_size)| (PreProcessedColumnId { id: id.to_string() }, log_size))
    .collect()
}

/// Builds a `NoValue` multiverifier and preprocesses it. The multiverifier is built by feeding it
/// two identical proofs of a circuit.
pub fn get_preprocessed_multiverifier_from_circuit(
    preprocessed_leaf_circuit: &PreprocessedCircuit,
    pcs_config: PcsConfig,
    target_padding: Option<ComponentSizes>,
) -> (PreprocessedCircuit, FinalizedContext<NoValue>) {
    assert_eq!(
        pcs_config.min_lifting_log_size,
        preprocessed_leaf_circuit.trace_log_size + pcs_config.fri_config.log_blowup_factor
    );
    let preprocessed_column_log_sizes = preprocessed_leaf_circuit.preprocessed_trace.log_sizes();
    // `ProofConfig` expects the components in ascending log-size order.
    let mut components = all_circuit_components::<NoValue>();
    let log_sizes = circuit_component_log_sizes(&components, &preprocessed_column_log_sizes);
    components.sort_by(|a, _, b, _| log_sizes[*a].cmp(&log_sizes[*b]));
    let proof_config = ProofConfig::new(
        &components,
        preprocessed_leaf_circuit.preprocessed_trace.n_columns(),
        &pcs_config,
        INTERACTION_POW_BITS,
    );
    let subcircuit_config = CircuitConfig {
        config: pcs_config,
        n_outputs: preprocessed_leaf_circuit.n_outputs,
        preprocessed_column_log_sizes,
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
        output_values: [0u32; N_RESERVED],
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
        lifting_log_size,
        &BaseColumnPool::<SimdBackend>::new(),
    );
    preprocessed_tree.commitment.root().into()
}
