use std::fs::File;

use cairo_air::utils::binary_deserialize_from_file;
use circuit_cairo_verifier::privacy::{privacy_cairo_verifier_config, privacy_components};
use circuit_cairo_verifier::utils::get_proof_file_path;
use circuit_common::finalize::finalize_context as pad_components_to_powers_of_two;
use circuit_common::order_hash_map::OrderedHashMap;
use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_prover::prover::{
    BaseColumnPool, SimdBackend, prepare_circuit_proof_for_circuit_verifier,
    prove_circuit_assignment,
};
use circuit_serialize::serialize::CircuitSerialize;
use circuit_verifier::statement::{INTERACTION_POW_BITS, all_circuit_components};
use circuit_verifier::verify::{CircuitConfig, CircuitPublicData};
use circuits::blake::{HashValue, blake_qm31};
use circuits::context::Context;
use circuits::ivalue::{NoValue, qm31_from_u32s};
use circuits::wrappers::M31Wrapper;
use circuits_stark_verifier::proof::ProofConfig;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::fri::FriConfig;
use stwo::core::pcs::PcsConfig;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use super::{
    Metadata, MetadataTree, SubCircuitConfig, SubCircuitInput, build_multiverifier_circuit,
};
use crate::cairo_subcircuit::{build_cairo_verifier_subcircuit, verify_cairo_with_component_set};
use crate::padding::pad_components_to_target_counts;
use crate::test_utils::{
    ComponentSizes, compute_component_sizes, pp_multiverifier_circuit_from_subcircuit,
};

const N_OUTPUTS: usize = 5;

mod shared_targets {
    pub(super) const EQ: usize = 1 << 17; // 131_072
    pub(super) const QM31_OPS: usize = 1 << 21; // 2_097_152
    pub(super) const N_BLAKE_GATES: usize = 1 << 14; // 16_384
    pub(super) const N_BLAKE_COMPRESS_ROWS: usize = 1 << 14; // 32_768
}

const FRI_CONFIG: FriConfig =
    FriConfig { log_blowup_factor: 3, log_last_layer_degree_bound: 0, n_queries: 27, fold_step: 4 };
const PCS_CONFIG: PcsConfig =
    PcsConfig { pow_bits: 10, fri_config: FRI_CONFIG, lifting_log_size: None };

fn pp_cairo_circuit(
    pcs_config: &mut PcsConfig,
    target_padding: Option<ComponentSizes>,
) -> (PreprocessedCircuit, Context<NoValue>) {
    let const_config = privacy_cairo_verifier_config(pcs_config.fri_config.log_blowup_factor);
    let mut novalue_context = build_cairo_verifier_subcircuit(&const_config);
    if let Some(ComponentSizes { eq, qm31_ops, n_blake_gates, n_blake_updates }) = target_padding {
        pad_components_to_target_counts(
            &mut novalue_context,
            eq,
            qm31_ops,
            n_blake_gates,
            n_blake_updates,
        );
    }
    let pp = PreprocessedCircuit::preprocess_circuit(&mut novalue_context);
    // Modify the pcs config to match the pcs config which will be used to **prove** pp.
    pcs_config.lifting_log_size =
        Some(pp.params.trace_log_size + pcs_config.fri_config.log_blowup_factor);
    (pp, novalue_context)
}

/// Diagnostic: print cairo's and multi's natural component counts under the
/// current `PCS_CONFIG` so we can pick the right bracket targets.
#[test]
fn test_compare_cairo_and_multiverifier_stats() {
    let mut pcs_config = PCS_CONFIG;
    let target_padding = ComponentSizes {
        eq: 1 << 17,
        qm31_ops: 1 << 21,
        n_blake_gates: 1 << 14,
        n_blake_updates: 1 << 14,
    };

    let (pp_cairo_circuit, novalue_cairo_context) =
        pp_cairo_circuit(&mut pcs_config, Some(target_padding.clone()));
    // pp_cairo_circuit(&mut pcs_config, None);
    let cairo_component_sizes = compute_component_sizes(&pp_cairo_circuit, &novalue_cairo_context);
    println!("cairo: {}", cairo_component_sizes);

    let (pp_multiverifier_circuit, novalue_multiverifier_context) =
        pp_multiverifier_circuit_from_subcircuit(
            &pp_cairo_circuit,
            pcs_config,
            Some(target_padding),
        );
    let multiverifier_component_sizes =
        compute_component_sizes(&pp_multiverifier_circuit, &novalue_multiverifier_context);
    println!("multiverifier: {}", multiverifier_component_sizes);

    // Compute the max between the two vectors of sizes.
    let shared_max_component_sizes = ComponentSizes {
        eq: cairo_component_sizes.eq.max(multiverifier_component_sizes.eq),
        qm31_ops: cairo_component_sizes.qm31_ops.max(multiverifier_component_sizes.qm31_ops),
        n_blake_gates: cairo_component_sizes
            .n_blake_gates
            .max(multiverifier_component_sizes.n_blake_gates),
        n_blake_updates: cairo_component_sizes
            .n_blake_updates
            .max(multiverifier_component_sizes.n_blake_updates),
    };
    assert_eq!(
        pp_multiverifier_circuit.preprocessed_trace.ids(),
        pp_cairo_circuit.preprocessed_trace.ids()
    );
    println!("max: {}", shared_max_component_sizes)
}

// /// After padding multi up into Cairo's brackets, both circuits should produce
// /// identical `preprocessed_column_ids` (in the same order) when preprocessed.
// /// // TODO: use pp cairo circuit and pp_multiverifier
// #[test]
// fn test_preprocessed_column_ids_are_equal() {
//     let cairo_config = cairo_circuit_config(PCS_CONFIG);
//     let cairo_ids = &cairo_config.preprocessed_column_ids;

//     let multi_meta = extract_padded_multi_metadata(&cairo_config);
//     let multi_ids = &multi_meta.preprocessed_column_ids;
//     assert_eq!(cairo_ids, multi_ids);
// }

const MULTI_OF_CAIRO_PROOF_PATH: &str = "/tmp/circuit_multiverifier_test_multi_of_cairo_proof.bin";

/// Bundles the per-proof outputs of `prove_cairo_and_prepare` so callers can
/// build a `SubCircuitInput` *and* read `config` / `public_data` separately
/// (e.g. to derive `pcs_config`, output addresses, or payload values).
struct CairoBundle {
    proof: circuits_stark_verifier::proof::Proof<QM31>,
    public_data: CircuitPublicData,
    config: CircuitConfig,
}

impl CairoBundle {
    fn into_subcircuit_input(self) -> SubCircuitInput<QM31> {
        SubCircuitInput {
            proof: self.proof,
            metadata: Metadata::from_config(&self.config),
            unconstrained_outputs: [
                self.public_data.output_values[0],
                self.public_data.output_values[1],
            ],
            is_multiverifier: false,
        }
    }
}

/// Builds & proves the Cairo verifier circuit (post-processed to expose 5
/// outputs in the multiverifier's convention) on the privacy proof fixture.
fn prove_cairo_and_prepare() -> CairoBundle {
    // Load the cached Cairo proof.
    let proof_path = get_proof_file_path("privacy");
    let proof_file = File::open(proof_path).expect("test_data/privacy/proof.bin must exist");
    let cairo_proof = binary_deserialize_from_file(&proof_file).expect("read cairo proof");

    let cairo_proof_log_blowup_factor = 3;
    let const_config = privacy_cairo_verifier_config(cairo_proof_log_blowup_factor);

    // NoValue topology — uses `_with_prepended_outputs` so the circuit has 5
    // outputs in the multiverifier's `[0, 0, hash, hash, u]` layout, then
    // padded into the shared brackets so its preprocessed columns agree with
    // the multiverifier's. The QM31 path below applies the same padding so
    // both topologies match (and `commitments[0]` matches the offline
    // preprocessed_root).
    let mut novalue_context = build_cairo_verifier_subcircuit(&const_config);
    pad_components_to_target_counts(
        &mut novalue_context,
        shared_targets::EQ,
        shared_targets::QM31_OPS,
        shared_targets::N_BLAKE_GATES,
        shared_targets::N_BLAKE_COMPRESS_ROWS,
    );
    let preprocessed = PreprocessedCircuit::preprocess_circuit(&mut novalue_context);

    // QM31 context with values from the proof.
    let mut context = verify_cairo_with_component_set(&cairo_proof, privacy_components()).unwrap();
    pad_components_to_target_counts(
        &mut context,
        shared_targets::EQ,
        shared_targets::QM31_OPS,
        shared_targets::N_BLAKE_GATES,
        shared_targets::N_BLAKE_COMPRESS_ROWS,
    );
    pad_components_to_powers_of_two(&mut context);

    let circuit_proof = prove_circuit_assignment(
        context.values(),
        &preprocessed,
        &BaseColumnPool::<SimdBackend>::new(),
        PCS_CONFIG,
    ).unwrap();

    let preprocessed_root: HashValue<QM31> =
        circuit_proof.stark_proof.proof.commitments[0].into();

    let preprocessed_column_log_sizes = preprocessed.preprocessed_trace.log_sizes();
    let proof_config = ProofConfig::new(
        &all_circuit_components::<QM31>(),
        vec![true; all_circuit_components::<QM31>().len()],
        preprocessed_column_log_sizes.len(),
        &circuit_proof.pcs_config,
        INTERACTION_POW_BITS,
    );

    let config = CircuitConfig {
        config: circuit_proof.pcs_config,
        output_addresses: preprocessed.params.output_addresses.clone(),
        n_blake_gates: preprocessed.params.n_blake_gates,
        preprocessed_column_log_sizes,
        preprocessed_root,
    };

    let (proof, public_data) =
        prepare_circuit_proof_for_circuit_verifier(circuit_proof, &proof_config);

    CairoBundle { proof, public_data, config }
}

/// Structural metadata of the *padded* multiverifier circuit (the one whose
/// `preprocessed_column_ids` match the Cairo verifier's). Same shape Tests A
/// and B both target — the recursion fixed point is at the same trace size as
/// Cairo (`trace_log_size = 21`).
#[derive(Debug)]
struct MultiverifierBlob {
    preprocessed_root: HashValue<QM31>,
    output_addresses: Vec<usize>,
    n_blake_gates: usize,
    preprocessed_column_ids:
        Vec<stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId>,
}

fn build_multiverifier_blob(
    pp: &PreprocessedCircuit,
    forward_log_blowup_factor: u32,
) -> MultiverifierBlob {
    MultiverifierBlob {
        preprocessed_root: get_preprocessed_root(pp, forward_log_blowup_factor),
        output_addresses: pp.params.output_addresses.clone(),
        n_blake_gates: pp.params.n_blake_gates,
        preprocessed_column_ids: pp.preprocessed_trace.ids(),
    }
}

/// Builds a `Metadata<QM31>` for the multiverifier circuit. (Used as the
/// leaf-1 entry of the metadata Merkle tree.)
fn build_multiverifier_metadata(blob: MultiverifierBlob) -> Metadata<QM31> {
    Metadata {
        n_blake_gates_pow_two: M31Wrapper::new_unsafe(QM31::from(
            blob.n_blake_gates.next_power_of_two(),
        )),
        output_addresses: blob
            .output_addresses
            .iter()
            .map(|x| M31Wrapper::new_unsafe(QM31::from(*x)))
            .collect(),
        preprocessed_root: blob.preprocessed_root,
    }
}

fn get_preprocessed_root(pp: &PreprocessedCircuit, log_blowup_factor: u32) -> HashValue<QM31> {
    use stwo::core::poly::circle::CanonicCoset;
    use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
    use stwo::prover::CommitmentTreeProver;
    use stwo::prover::poly::circle::PolyOps;

    assert!(log_blowup_factor > 0);
    let lifting_log_size = pp.params.trace_log_size + log_blowup_factor;
    // Compute the multi's preprocessed_root via the commitment tree.
    let twiddles = SimdBackend::precompute_twiddles(
        CanonicCoset::new(lifting_log_size).circle_domain().half_coset,
    );
    let preprocessed_trace = pp.preprocessed_trace.get_trace::<SimdBackend>();
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

#[test]
fn generate_multiverifier_metadata_constant() {
    let mut pcs_config = PCS_CONFIG;
    let target_padding = ComponentSizes {
        eq: 1 << 17,
        qm31_ops: 1 << 21,
        n_blake_gates: 1 << 14,
        n_blake_updates: 1 << 14,
    };

    let (pp_cairo_circuit, _) = pp_cairo_circuit(&mut pcs_config, Some(target_padding.clone()));
    let (pp_multiverifier, _) = pp_multiverifier_circuit_from_subcircuit(
        &pp_cairo_circuit,
        pcs_config,
        Some(target_padding),
    );
    let blob = build_multiverifier_blob(&pp_multiverifier, pcs_config.fri_config.log_blowup_factor);
    println!("preprocessed_column_ids: {:?}", blob.preprocessed_column_ids);
    println!("n_blake_gates: {}", blob.n_blake_gates);
    println!("{:?}", build_multiverifier_metadata(blob));
}
// ---------- end of discovery part

const MULTIVERIFIER_N_BLAKE_GATES: usize = 10946;
const MULTIVERIFIER_METADATA_N_BLAKE_GATES_POW_TWO: u32 = 16384;
const MULTIVERIFIER_METADATA_OUTPUT_ADDRESSES: [u32; N_OUTPUTS] = [777062, 777063, 3, 4, 2];
const MULTIVERIFIER_METADATA_PREPROCESSED_ROOT: [u32; 8] = [
    2147172461, 427259034, 1773458221, 293019583, 239753370, 1538327556, 747464207, 1848932673,
];
const MULTIVERIFIER_N_PREPROCESSED_COLUMNS: usize = 79;
fn multiverifier_preprocessed_column_log_sizes() -> OrderedHashMap<PreProcessedColumnId, u32> {
    [
        ("blake_sigma_0", 4),
        ("blake_sigma_1", 4),
        ("blake_sigma_2", 4),
        ("blake_sigma_3", 4),
        ("blake_sigma_4", 4),
        ("blake_sigma_5", 4),
        ("blake_sigma_6", 4),
        ("blake_sigma_7", 4),
        ("blake_sigma_8", 4),
        ("blake_sigma_9", 4),
        ("blake_sigma_10", 4),
        ("blake_sigma_11", 4),
        ("blake_sigma_12", 4),
        ("blake_sigma_13", 4),
        ("blake_sigma_14", 4),
        ("blake_sigma_15", 4),
        ("triple_xor_input_addr_0", 4),
        ("triple_xor_input_addr_1", 4),
        ("triple_xor_input_addr_2", 4),
        ("triple_xor_output_addr", 4),
        ("triple_xor_multiplicity", 4),
        ("m31_to_u32_input_addr", 4),
        ("m31_to_u32_output_addr", 4),
        ("m31_to_u32_multiplicity", 4),
        ("blake_g_gate_input_addr_a", 4),
        ("blake_g_gate_input_addr_b", 4),
        ("blake_g_gate_input_addr_c", 4),
        ("blake_g_gate_input_addr_d", 4),
        ("blake_g_gate_input_addr_f0", 4),
        ("blake_g_gate_input_addr_f1", 4),
        ("blake_g_gate_output_addr_a", 4),
        ("blake_g_gate_output_addr_b", 4),
        ("blake_g_gate_output_addr_c", 4),
        ("blake_g_gate_output_addr_d", 4),
        ("blake_g_gate_multiplicity", 4),
        ("seq_4", 4),
        ("bitwise_xor_4_0", 8),
        ("bitwise_xor_4_1", 8),
        ("bitwise_xor_4_2", 8),
        ("t0", 14),
        ("t1", 14),
        ("finalize_flag", 14),
        ("state_before_addr", 14),
        ("state_after_addr", 14),
        ("message0_addr", 14),
        ("message1_addr", 14),
        ("message2_addr", 14),
        ("message3_addr", 14),
        ("compress_enabler", 14),
        ("final_state_addr", 14),
        ("blake_output0_addr", 14),
        ("blake_output1_addr", 14),
        ("blake_output0_mults", 14),
        ("blake_output1_mults", 14),
        ("seq_14", 14),
        ("bitwise_xor_7_0", 14),
        ("bitwise_xor_7_1", 14),
        ("bitwise_xor_7_2", 14),
        ("seq_15", 15),
        ("seq_16", 16),
        ("bitwise_xor_8_0", 16),
        ("bitwise_xor_8_1", 16),
        ("bitwise_xor_8_2", 16),
        ("eq_in0_address", 17),
        ("eq_in1_address", 17),
        ("bitwise_xor_9_0", 18),
        ("bitwise_xor_9_1", 18),
        ("bitwise_xor_9_2", 18),
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

fn make_struct() -> Metadata<QM31> {
    Metadata {
        n_blake_gates_pow_two: M31Wrapper::from(M31::from(
            MULTIVERIFIER_METADATA_N_BLAKE_GATES_POW_TWO,
        )),
        output_addresses: MULTIVERIFIER_METADATA_OUTPUT_ADDRESSES
            .iter()
            .map(|x| M31Wrapper::from(M31::from(*x)))
            .collect(),
        preprocessed_root: HashValue::from(MULTIVERIFIER_METADATA_PREPROCESSED_ROOT),
    }
}

/// Test A (Cairo): build & prove a *padded* multiverifier verifying two Cairo
/// proofs, with the correct recursion-tree `metadata_root H = hash_node(h_cairo, h_multi)`,
/// and write the resulting `Proof<QM31>` bytes to disk for Test B to consume.
#[test]
fn test_prove_multiverifier_of_two_cairo_subcircuits() {
    let bundle = prove_cairo_and_prepare();
    assert_eq!(bundle.config.output_addresses.len(), N_OUTPUTS);
    let subcircuit_pcs_config = bundle.config.config;

    // TODO: these metadata configs could be consts.
    let metadata_cairo = Metadata::<QM31>::from_config(&bundle.config);
    let multiverifier_metadata = make_struct();
    let metadata_tree = MetadataTree::<QM31>::commit(metadata_cairo, multiverifier_metadata);

    // TODO: should be const.
    let subcircuit_config = SubCircuitConfig {
        pcs_config: subcircuit_pcs_config,
        n_outputs: bundle.config.output_addresses.len(),
        preprocessed_column_ids: bundle
            .config
            .preprocessed_column_log_sizes
            .keys()
            .cloned()
            .collect(),
    };
    let mut multiverifier_context = build_multiverifier_circuit::<QM31>(
        bundle.into_subcircuit_input(),
        // TODO: change to bundle.clone().into_subcircuit_input() when Clone becomes available.
        prove_cairo_and_prepare().into_subcircuit_input(),
        subcircuit_config,
        metadata_tree,
    );
    // Apply the same Cairo-target padding the metadata extraction did.
    pad_components_to_target_counts(
        &mut multiverifier_context,
        shared_targets::EQ,
        shared_targets::QM31_OPS,
        shared_targets::N_BLAKE_GATES,
        shared_targets::N_BLAKE_COMPRESS_ROWS,
    );
    multiverifier_context.validate_circuit();

    // 6. Prove the multiverifier.
    let pp_multi = PreprocessedCircuit::preprocess_circuit(&mut multiverifier_context);
    let multi_circuit_proof = prove_circuit_assignment(
        multiverifier_context.values(),
        &pp_multi,
        &BaseColumnPool::<SimdBackend>::new(),
        PCS_CONFIG,
    ).unwrap();
    let multi_proof_config = ProofConfig::new(
        &all_circuit_components::<QM31>(),
        vec![true; all_circuit_components::<QM31>().len()],
        MULTIVERIFIER_N_PREPROCESSED_COLUMNS,
        &subcircuit_pcs_config,
        INTERACTION_POW_BITS,
    );
    let (multi_proof, _multi_public_data) =
        prepare_circuit_proof_for_circuit_verifier(multi_circuit_proof, &multi_proof_config);
    let mut serialized = vec![];
    multi_proof.serialize(&mut serialized);
    std::fs::write(MULTI_OF_CAIRO_PROOF_PATH, &serialized).expect("write multi-of-cairo proof");
}

/// Test B (Cairo): load Test A's multi proof, prove a fresh Cairo proof, build
/// a *second-level* padded multi that verifies `[multi_proof, cairo_proof]`,
/// and value-validate it. The second-level multi is the same circuit as the
/// first-level one (same padding, same inner pcs_config) — recursion fixed
/// point at `trace_log_size = 21`.
#[test]
fn test_verify_multiverifier_proof_and_cairo_proof() {
    use circuit_serialize::deserialize::deserialize_proof_with_config;

    // 1. Fresh Cairo proof (also gives us the cairo_config for everything else).
    let bundle = prove_cairo_and_prepare();
    let subcircuit_pcs_config = bundle.config.config;

    // TODO: these metadata configs could be consts.
    let metadata_cairo = Metadata::<QM31>::from_config(&bundle.config);
    let multiverifier_metadata = make_struct();
    let metadata_tree = MetadataTree::<QM31>::commit(metadata_cairo, multiverifier_metadata);
    let metadata_root = metadata_tree.root;

    // 3. Reconstruct the multi's expected `CircuitPublicData`. Multi's outputs are `[H_lo, H_hi,
    //    hash_of_payloads_lo, hash_of_payloads_hi, u]`, where `hash_of_payloads =
    //    blake([cairo_payload, cairo_payload], 64)` over the two identical Cairo payload pairs
    //    (Test A's two Cairo proofs are deterministic and identical, so so are this Test's).
    let cairo_payload_lo = bundle.public_data.output_values[0];
    let cairo_payload_hi = bundle.public_data.output_values[1];
    let hash_of_payloads =
        blake_qm31(&[cairo_payload_lo, cairo_payload_hi, cairo_payload_lo, cairo_payload_hi], 64);
    let u_value = qm31_from_u32s(0, 0, 1, 0);
    let multi_public_data = CircuitPublicData {
        output_values: vec![
            hash_of_payloads.0,
            hash_of_payloads.1,
            metadata_root.0,
            metadata_root.1,
            u_value,
        ],
    };

    // 4. Multi's `CircuitConfig` (same shape as Test A produced).
    let multi_config = CircuitConfig {
        config: subcircuit_pcs_config,
        output_addresses: MULTIVERIFIER_METADATA_OUTPUT_ADDRESSES
            .iter()
            .map(|&x| x as usize)
            .collect(),
        n_blake_gates: MULTIVERIFIER_N_BLAKE_GATES,
        preprocessed_column_log_sizes: multiverifier_preprocessed_column_log_sizes(),
        preprocessed_root: HashValue::from(MULTIVERIFIER_METADATA_PREPROCESSED_ROOT),
    };

    // 5. Multi's `ProofConfig` for deserialization.
    let multi_proof_config = ProofConfig::new(
        &all_circuit_components::<QM31>(),
        vec![true; all_circuit_components::<QM31>().len()],
        MULTIVERIFIER_N_PREPROCESSED_COLUMNS,
        &subcircuit_pcs_config,
        INTERACTION_POW_BITS,
    );

    // 6. Load Test A's saved proof.
    let bytes = std::fs::read(MULTI_OF_CAIRO_PROOF_PATH)
        .expect("Test A (cairo) must run first to write the multi proof to disk");
    let mut slice = bytes.as_slice();
    let multi_proof = deserialize_proof_with_config(&mut slice, &multi_proof_config)
        .expect("deserialize multi-of-cairo proof");

    // Build inputs for multiverifier.
    let multi_input = SubCircuitInput {
        proof: multi_proof,
        metadata: Metadata::from_config(&multi_config),
        unconstrained_outputs: [
            multi_public_data.output_values[0],
            multi_public_data.output_values[1],
        ],
        is_multiverifier: true,
    };
    let cairo_input = bundle.into_subcircuit_input();
    let subcircuit_config = SubCircuitConfig {
        pcs_config: subcircuit_pcs_config,
        n_outputs: N_OUTPUTS,
        preprocessed_column_ids: multiverifier_preprocessed_column_log_sizes()
            .keys()
            .cloned()
            .collect(),
    };
    let mut context = build_multiverifier_circuit::<QM31>(
        multi_input,
        cairo_input,
        subcircuit_config,
        metadata_tree,
    );

    pad_components_to_target_counts(
        &mut context,
        shared_targets::EQ,
        shared_targets::QM31_OPS,
        shared_targets::N_BLAKE_GATES,
        shared_targets::N_BLAKE_COMPRESS_ROWS,
    );
    context.circuit.check_yields();
    context.validate_circuit();

    // Check that the circuit hasn't changed.
    let pp = PreprocessedCircuit::preprocess_circuit(&mut context);
    let pp_root = get_preprocessed_root(&pp, subcircuit_pcs_config.fri_config.log_blowup_factor);
    assert_eq!(pp_root, HashValue::from(MULTIVERIFIER_METADATA_PREPROCESSED_ROOT));
}
