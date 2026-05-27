use std::fs::File;
use std::io::Write;

use cairo_air::utils::binary_deserialize_from_file;
use circuit_cairo_verifier::privacy::{
    get_pcs_config, privacy_cairo_verifier_config, privacy_components,
};
use circuit_cairo_verifier::utils::get_proof_file_path;
use circuit_cairo_verifier::verify::build_cairo_verifier_circuit;
use circuit_cairo_verifier::verify::verify_cairo_with_component_set;
use circuit_common::finalize::{ComponentSizes, compute_padded_sizes, pad_to_targets};
use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_prover::prover::{
    prepare_circuit_proof_for_circuit_verifier, prove_circuit_assignment,
};
use circuit_serialize::deserialize::deserialize_proof_with_config;
use circuit_serialize::serialize::CircuitSerialize;
use circuit_verifier::statement::{INTERACTION_POW_BITS, all_circuit_components};
use circuit_verifier::verify::CircuitPublicData;
use circuits::blake::HashValue;
use circuits::context::FinalizedContext;
use circuits::ivalue::NoValue;
use circuits_stark_verifier::order_hash_map::OrderedHashMap;
use circuits_stark_verifier::proof::{Proof, ProofConfig};
use itertools::Itertools;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::mempool::BaseColumnPool;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::test_utils::{get_preprocessed_multiverifier_from_circuit, get_preprocessed_root};
use crate::verify::{MultiverifierInput, SharedConfig, build_multiverifier_circuit};

const PRIVACY_CAIRO_VERIFIER_TRACE_LOG_SIZE: u32 = 21;
const LOG_BLOWUP_FACTOR: u32 = 3;
const PCS_CONFIG: PcsConfig =
    get_pcs_config(PRIVACY_CAIRO_VERIFIER_TRACE_LOG_SIZE, LOG_BLOWUP_FACTOR);
const TARGET_PADDING_SIZES: ComponentSizes = ComponentSizes {
    eq: 1 << 17,
    qm31_ops: 1 << 21,
    m31_to_u32: 1 << 18,
    triple_xor: 1 << 17,
    blake_g_gate: 1 << 20,
};
/// The number of preprocessed columns in a trace of a circuit.
const CIRCUIT_N_PREPROCESSED_COLUMNS: usize = 45;

/// Constants related to the cairo verifier circuit.
const PRIVACY_CAIRO_VERIFIER_PREPROCESSED_ROOT: [u32; 8] =
    [1564451235, 1866679958, 2011431219, 402982173, 1661380608, 1553398776, 620364350, 714877734];
const PRIVACY_CAIRO_VERIFIER_PROOF_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../test_data/circuit_multiverifier/proof_cairo.bin");
const PRIVACY_CAIRO_VERIFIER_OUTPUT_VALUES: [QM31; 2] = [
    QM31::from_m31_array([M31(151966945), M31(1514947052), M31(87572453), M31(633358207)]),
    QM31::from_m31_array([M31(462231094), M31(464091325), M31(2016711704), M31(1173534648)]),
];

/// Constants related to the multiverifier circuit.
const MULTIVERIFIER_PREPROCESSED_ROOT: [u32; 8] =
    [1207218485, 45060776, 317382138, 1169749503, 506165738, 1544606560, 1742997373, 1081501915];
const MULTIVERIFIER_OF_TWO_CAIRO_PROOFS_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../test_data/circuit_multiverifier/proof.bin");

fn multiverifier_preprocessed_column_log_sizes() -> OrderedHashMap<PreProcessedColumnId, u32> {
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

/// Builds a `NoValue` Cairo verifier circuit (with configs of privacy) and preprocesses it.
///
/// If `target_padding` is `None`, we don't pad the circuits (note that even in this case, the
/// default padding to the next power of 2 happens in [`PreprocessedCircuit::preprocess_circuit`]).
fn get_preprocessed_cairo_verifier(
    pcs_config: PcsConfig,
    target_padding: Option<ComponentSizes>,
) -> (PreprocessedCircuit, FinalizedContext<NoValue>) {
    let const_config = privacy_cairo_verifier_config(pcs_config.fri_config.log_blowup_factor);
    let mut novalue_context = build_cairo_verifier_circuit(&const_config);
    if let Some(target_padding) = target_padding {
        pad_to_targets(&mut novalue_context, target_padding);
    }
    let preprocessed_cairo_verifier = PreprocessedCircuit::preprocess_circuit(&mut novalue_context);
    (preprocessed_cairo_verifier, novalue_context)
}

/// Regression test on the padded sizes of the Cairo verifier and the multiverifier circuits.
///
/// Builds both circuits (unpadded) for `PCS_CONFIG` and prints their `compute_padded_sizes`
/// breakdowns side by side. For the multiverifier to be able to verify executions of the cairo
/// verifier and itself without changing, the sizes of the AIR components for both circuits must be
/// the same. This is achieved by padding each component to the next power of two of the max between
/// the component's size in the two AIRs (note that by doing this we may end up padding both
/// circuits). This test is needed to deduce the target padding whenever the `PCS_CONFIG` changes.
#[test]
fn test_regression_cairo_and_multiverifier_component_log_sizes() {
    let pcs_config = PCS_CONFIG;
    let (preprocessed_cairo_verifier, novalue_cairo_context) =
        get_preprocessed_cairo_verifier(pcs_config, None);

    let cairo_component_sizes = compute_padded_sizes(&novalue_cairo_context);
    let expected_sizes = ComponentSizes {
        eq: 1 << 17,
        qm31_ops: 1 << 21,
        m31_to_u32: 1 << 18,
        triple_xor: 1 << 17,
        blake_g_gate: 1 << 20,
    };
    assert_eq!(cairo_component_sizes, expected_sizes);

    let (_, novalue_multiverifier_context) =
        get_preprocessed_multiverifier_from_circuit(&preprocessed_cairo_verifier, pcs_config, None);
    let multiverifier_component_sizes = compute_padded_sizes(&novalue_multiverifier_context);
    let expected_sizes = ComponentSizes {
        eq: 1 << 15,
        qm31_ops: 1 << 21,
        m31_to_u32: 1 << 18,
        triple_xor: 1 << 17,
        blake_g_gate: 1 << 20,
    };
    assert_eq!(multiverifier_component_sizes, expected_sizes);
}

#[test]
fn test_padding_is_correct() {
    let pcs_config = PCS_CONFIG;
    let target_padding = TARGET_PADDING_SIZES;
    let (pp_cairo_circuit, _context) =
        get_preprocessed_cairo_verifier(pcs_config, Some(target_padding.clone()));
    let (pp_multiverifier_circuit, _context) = get_preprocessed_multiverifier_from_circuit(
        &pp_cairo_circuit,
        pcs_config,
        Some(target_padding),
    );
    assert_eq!(
        pp_multiverifier_circuit.preprocessed_trace.ids(),
        pp_cairo_circuit.preprocessed_trace.ids()
    );
    assert_eq!(pp_multiverifier_circuit.trace_log_size, pp_cairo_circuit.trace_log_size);
}

#[test]
fn test_regression_constants() {
    let pcs_config = PCS_CONFIG;
    let target_padding = TARGET_PADDING_SIZES;

    let (pp_cairo_circuit, _context) =
        get_preprocessed_cairo_verifier(pcs_config, Some(target_padding.clone()));
    let (pp_multiverifier, _context) = get_preprocessed_multiverifier_from_circuit(
        &pp_cairo_circuit,
        pcs_config,
        Some(target_padding),
    );
    let cairo_verifier_root =
        get_preprocessed_root(&pp_cairo_circuit, pcs_config.fri_config.log_blowup_factor);
    let multiverifier_root =
        get_preprocessed_root(&pp_multiverifier, pcs_config.fri_config.log_blowup_factor);
    let to_u32_array = |qm31: QM31| [qm31.0.0.0, qm31.0.1.0, qm31.1.0.0, qm31.1.1.0];

    assert_eq!(
        &PRIVACY_CAIRO_VERIFIER_PREPROCESSED_ROOT,
        [cairo_verifier_root.0, cairo_verifier_root.1]
            .into_iter()
            .flat_map(to_u32_array)
            .collect_vec()
            .as_slice()
    );
    assert_eq!(
        &MULTIVERIFIER_PREPROCESSED_ROOT,
        [multiverifier_root.0, multiverifier_root.1]
            .into_iter()
            .flat_map(to_u32_array)
            .collect_vec()
            .as_slice()
    );
    assert_eq!(CIRCUIT_N_PREPROCESSED_COLUMNS, pp_multiverifier.preprocessed_trace.ids().len());
    assert_eq!(
        multiverifier_preprocessed_column_log_sizes(),
        pp_multiverifier.preprocessed_trace.log_sizes()
    );
}

/// Builds a `MultiverifierInput` that wraps a proof of the privacy cairo verifier (i.e. the circuit
/// with preprocessed root = `PRIVACY_CAIRO_VERIFIER_PREPROCESSED_ROOT`).
fn build_cairo_input(proof: &Proof<QM31>) -> MultiverifierInput<QM31> {
    MultiverifierInput {
        proof: proof.clone(),
        preprocessed_root: HashValue::<QM31>::from(PRIVACY_CAIRO_VERIFIER_PREPROCESSED_ROOT),
        output_values: PRIVACY_CAIRO_VERIFIER_OUTPUT_VALUES,
    }
}

/// Builds and proves the Cairo verifier circuit on a privacy proof.
fn prove_privacy_with_recursion_and_prepare() -> (Proof<QM31>, CircuitPublicData) {
    let proof_path = get_proof_file_path("privacy");
    let proof_file = File::open(proof_path).expect("test_data/privacy/proof.bin must exist");
    let cairo_proof = binary_deserialize_from_file(&proof_file).expect("read cairo proof");

    let const_config = privacy_cairo_verifier_config(LOG_BLOWUP_FACTOR);
    let mut novalue_context = build_cairo_verifier_circuit(&const_config);
    pad_to_targets(&mut novalue_context, TARGET_PADDING_SIZES.clone());
    let preprocessed = PreprocessedCircuit::preprocess_circuit(&mut novalue_context);

    // QM31 context with values from the proof.
    let mut context = verify_cairo_with_component_set(&cairo_proof, privacy_components()).unwrap();
    pad_to_targets(&mut context, TARGET_PADDING_SIZES.clone());

    let circuit_proof = prove_circuit_assignment(
        context.values(),
        &preprocessed,
        &BaseColumnPool::<SimdBackend>::new(),
        PCS_CONFIG,
    )
    .unwrap();

    prepare_circuit_proof_for_circuit_verifier(circuit_proof)
}

/// Regression test on the Cairo verifier proof.
///
/// Builds and proves the Cairo verifier circuit on the privacy proof (via
/// [`prove_privacy_with_recursion_and_prepare`]) and asserts its public outputs match
/// [`PRIVACY_CAIRO_VERIFIER_OUTPUT_VALUES`]. If `FIX_PROOF` env var is not set it deserializes the
/// proof stored at [`PRIVACY_CAIRO_VERIFIER_PROOF_PATH`] and asserts equality with the freshly
/// produced proof. When run with the `FIX_PROOF` env var set, it regenerates and overwrites the
/// proof.
#[test]
fn test_cairo_proof_regression() {
    let (proof, public_data) = prove_privacy_with_recursion_and_prepare();
    assert_eq!(public_data.output_values, PRIVACY_CAIRO_VERIFIER_OUTPUT_VALUES);
    if std::env::var("FIX_PROOF").is_err() {
        let proof_config = ProofConfig::new(
            &all_circuit_components::<QM31>(),
            CIRCUIT_N_PREPROCESSED_COLUMNS,
            &PCS_CONFIG,
            INTERACTION_POW_BITS,
        );
        let bytes = std::fs::read(PRIVACY_CAIRO_VERIFIER_PROOF_PATH).unwrap();
        let stored_proof =
            deserialize_proof_with_config(&mut bytes.as_slice(), &proof_config).unwrap();
        // Don't assert_eq to avoid printing large structs to stdout.
        if proof != stored_proof {
            panic!("The cairo verifier proof changed. Run again with `FIX_PROOF=1`.");
        };
    } else {
        let mut serialized = vec![];
        proof.serialize(&mut serialized);
        let mut file = File::create(PRIVACY_CAIRO_VERIFIER_PROOF_PATH).unwrap();
        file.write_all(&serialized).unwrap();
    }
}

/// Builds the multiverifier over two Cairo verifier proofs.
///
/// Reads the stored Cairo verifier proof fixture (produced by [`test_cairo_proof_regression`]) and
/// builds a multiverifier circuit that verifies two copies of it.
///
/// When run with the `FIX_PROOF` env var set, it additionally preprocesses and proves the
/// multiverifier circuit, then serializes the resulting proof to
/// [`MULTIVERIFIER_OF_TWO_CAIRO_PROOFS_PATH`] as a fixture for downstream tests.
#[test]
fn test_prove_multiverifier_of_two_cairo_subcircuits() {
    let shared_config = SharedConfig {
        pcs_config: PCS_CONFIG,
        proof_config: ProofConfig::new(
            &all_circuit_components::<QM31>(),
            CIRCUIT_N_PREPROCESSED_COLUMNS,
            &PCS_CONFIG,
            INTERACTION_POW_BITS,
        ),
        preprocessed_column_log_sizes: multiverifier_preprocessed_column_log_sizes(),
    };
    let bytes = std::fs::read(PRIVACY_CAIRO_VERIFIER_PROOF_PATH).unwrap();
    let proof =
        deserialize_proof_with_config(&mut bytes.as_slice(), &shared_config.proof_config).unwrap();
    let mut multiverifier_context = build_multiverifier_circuit::<QM31>(
        build_cairo_input(&proof),
        build_cairo_input(&proof),
        &shared_config,
    );
    pad_to_targets(&mut multiverifier_context, TARGET_PADDING_SIZES.clone());
    multiverifier_context.validate_circuit();

    if std::env::var("FIX_PROOF").is_ok() {
        // Prove the multiverifier.
        let preprocessed_multiverifier =
            PreprocessedCircuit::preprocess_circuit(&mut multiverifier_context);
        let multi_circuit_proof = prove_circuit_assignment(
            multiverifier_context.values(),
            &preprocessed_multiverifier,
            &BaseColumnPool::<SimdBackend>::new(),
            PCS_CONFIG,
        )
        .unwrap();
        let (multi_proof, _multi_public_data) =
            prepare_circuit_proof_for_circuit_verifier(multi_circuit_proof);
        let mut serialized = vec![];
        multi_proof.serialize(&mut serialized);
        let mut file = File::create(MULTIVERIFIER_OF_TWO_CAIRO_PROOFS_PATH).unwrap();
        file.write_all(&serialized).unwrap();
    }
}
