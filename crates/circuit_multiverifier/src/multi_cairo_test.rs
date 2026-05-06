//! Compares the multiverifier's `preprocessed_column_ids` against the Cairo
//! verifier circuit's, after padding the multiverifier up into Cairo's column
//! brackets via [`pad_components_to_target_counts`]. The goal is to confirm
//! that — once padded — both circuits sort to the *same* column order, which
//! is the precondition for a single multiverifier circuit body to verify both
//! Cairo-verifier proofs and multiverifier proofs.

use std::array;
use std::collections::HashSet;
use std::fs::File;

use cairo_air::CairoProof;
use cairo_air::flat_claims::FlatClaim;
use cairo_air::utils::binary_deserialize_from_file;
use cairo_air::verifier::INTERACTION_POW_BITS as CAIRO_INTERACTION_POW_BITS;
use circuit_cairo_verifier::all_components::all_components as cairo_all_components;
use circuit_cairo_verifier::privacy::{privacy_cairo_verifier_config, privacy_components};
use circuit_cairo_verifier::statement::MEMORY_VALUES_LIMBS;
use circuit_cairo_verifier::utils::get_proof_file_path;
use circuit_cairo_verifier::verify::{
    CairoVerifierConfig, build_cairo_verifier_subcircuit, prepare_cairo_proof_for_circuit_verifier,
    verify_fixed_cairo_subcircuit,
};
use circuit_common::finalize::finalize_context as pad_components_to_powers_of_two;
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
use circuits::ivalue::{IValue, NoValue, qm31_from_u32s};
use circuits::wrappers::M31Wrapper;
use circuits_stark_verifier::constraint_eval::CircuitEval;
use circuits_stark_verifier::proof::{ProofConfig, empty_proof};
use itertools::{Itertools, zip_eq};
use num_traits::Zero;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::fri::FriConfig;
use stwo::core::pcs::PcsConfig;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use super::{
    Metadata, MetadataTree, SubCircuitConfig, SubCircuitInput, build_multiverifier_circuit,
};
use crate::padding::{pad_components_to_target_counts, qm31_ops_n_rows};
use crate::verify::empty_metadata;

const N_OUTPUTS: usize = 5;

/// Shared component-size brackets that *both* the Cairo verifier and the
/// multiverifier are padded into, so they end up with identical
/// `preprocessed_column_ids` after `sort_by_size`. Empirically chosen to be
/// large enough to contain the natural counts of both circuits under the
/// current `PCS_CONFIG` (see `explore_cairo_and_multi_natural_counts`):
///
/// | component       | cairo natural | multi natural | bracket |
/// |-----------------|---------------|---------------|---------|
/// | eq              | 117_390       |  51_795       | 2^17    |
/// | qm31_ops        | 1_403_338     | 702_760       | 2^21    |
/// | n_blake_gates   |   4_318       |  10_937       | 2^14    |
/// | blake_compress  |  11_805       |  14_165       | 2^15    |
///
/// `blake_compress` must be `>= n_blake_gates + max(cairo, multi)
/// (compress_natural - gates_natural)` so the padding helper can reach both
/// targets with at most 4-chunk dummy blakes.
///
/// Keeping `blake_compress = 2^15` (rather than 2^16) keeps the multi's
/// `trace_log_size` at `blake_g_log_size = log_n_blake_updates + 7 = 22`
/// (vs 23 if we used `2^16`) — that's the difference between an 8GB and a
/// 32GB peak memory footprint at proving time with `log_blowup_factor = 3`.
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

/// Builds a NoValue context of the multiverifier when it is fed two proofs (with the same parameters) of the subcircuit.
fn pp_multiverifier_circuit_from_subcircuit(
    pp_subcircuit: &PreprocessedCircuit,
    pcs_config: PcsConfig,
    target_padding: Option<ComponentSizes>,
) -> (PreprocessedCircuit, Context<NoValue>) {
    let proof_config = ProofConfig::from_components(
        &all_circuit_components::<NoValue>(),
        vec![true; all_circuit_components::<NoValue>().len()],
        pp_subcircuit.preprocessed_trace.ids().len(),
        &pcs_config,
        INTERACTION_POW_BITS,
    );
    let subcircuit_config = CircuitConfig {
        config: pcs_config,
        output_addresses: pp_subcircuit.params.output_addresses.clone(),
        n_blake_gates: pp_subcircuit.params.n_blake_gates.clone(),
        preprocessed_column_ids: pp_subcircuit.preprocessed_trace.ids().clone(),
        preprocessed_root: HashValue(QM31::from(0), QM31::from(0)),
    };
    // Use a closure to bypass lack of Clone
    let make_input = || SubCircuitInput {
        proof: empty_proof(&proof_config),
        circuit_public_data: CircuitPublicData { output_values: vec![QM31::zero(); N_OUTPUTS] },
        config: subcircuit_config.clone(),
        is_multiverifier: false,
    };
    let subcircuit_config = SubCircuitConfig {
        pcs_config: subcircuit_config.config,
        n_outputs: N_OUTPUTS,
        preprocessed_column_ids: subcircuit_config.preprocessed_column_ids.clone(),
    };
    let empty_metadata = empty_metadata(N_OUTPUTS);
    let metadata_tree = MetadataTree::<NoValue>::commit(empty_metadata.clone(), empty_metadata);
    let mut multiverifier_context = build_multiverifier_circuit::<NoValue>(
        make_input(),
        make_input(),
        subcircuit_config,
        metadata_tree,
    );
    if let Some(ComponentSizes { eq, qm31_ops, n_blake_gates, n_blake_updates }) = target_padding {
        pad_components_to_target_counts(
            &mut multiverifier_context,
            eq,
            qm31_ops,
            n_blake_gates,
            n_blake_updates,
        );
    }
    let pp = PreprocessedCircuit::preprocess_circuit(&mut multiverifier_context);
    println!(
        "Ingested two proofs of trace_log_size = {} and fri_config = {:?} --> Multiverifier has trace_log_size = {}",
        pp_subcircuit.params.trace_log_size, pcs_config.fri_config, pp.params.trace_log_size
    );
    (pp, multiverifier_context)
}

fn compute_component_sizes(
    pp_circuit: &PreprocessedCircuit,
    context: &Context<impl IValue>,
) -> ComponentSizes {
    ComponentSizes {
        eq: context.stats.equals,
        qm31_ops: qm31_ops_n_rows(&context.circuit),
        n_blake_gates: pp_circuit.params.n_blake_gates,
        n_blake_updates: context.stats.blake_updates,
    }
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
        pp_multiverifier_circuit_from_subcircuit(&pp_cairo_circuit, pcs_config, Some(target_padding));
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

#[derive(Clone)]
pub struct ComponentSizes {
    pub eq: usize,
    pub qm31_ops: usize,
    pub n_blake_gates: usize,
    pub n_blake_updates: usize,
}

impl std::fmt::Display for ComponentSizes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "eq={}, qm31_ops={}, n_blake={}, blake_compress={}",
            self.eq, self.qm31_ops, self.n_blake_gates, self.n_blake_updates
        ))
    }
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

/// Inlined copy of `circuit_cairo_verifier::test::verify_cairo_with_component_set`
/// (which lives behind `#[cfg(test)]` in the cairo_verifier crate, so it isn't
/// callable from outside). Verifies a `CairoProof` with the given component
/// set and returns the resulting QM31 context.
// TODO: try to move it to cairo verifier crate
fn verify_cairo_with_component_set_inline(
    cairo_proof: &CairoProof<Blake2sM31MerkleHasher>,
    component_set: HashSet<&str>,
) -> Result<Context<QM31>, String> {
    let FlatClaim { component_enable_bits, component_log_sizes: _, public_data: _ } =
        cairo_proof.claim.flatten_claim();
    let components: indexmap::IndexMap<&'static str, Box<dyn CircuitEval<QM31>>> = zip_eq(
        cairo_all_components::<QM31>().into_iter(),
        &component_enable_bits,
    )
    .filter_map(|((component_name, component), &enable_bit)| {
        let component_in_set = component_set.contains(component_name);
        if component_in_set != enable_bit {
            return Some(Err(format!(
                "Proof was produced with the wrong components set: expected '{}' to be {} but it is {} in the proof.",
                component_name,
                if component_in_set { "enabled" } else { "disabled" },
                if enable_bit { "enabled" } else { "disabled" },
            )));
        }
        if enable_bit { Some(Ok((component_name, component))) } else { None }
    })
    .try_collect()?;

    let proof_config = ProofConfig::from_components(
        &components,
        component_enable_bits,
        cairo_proof.preprocessed_trace_variant.to_preprocessed_trace().ids().len(),
        &cairo_proof.extended_stark_proof.proof.config,
        CAIRO_INTERACTION_POW_BITS,
    );

    let (proof, public_data) = prepare_cairo_proof_for_circuit_verifier(cairo_proof, &proof_config);
    let (public_claim, outputs, program) = public_data.pack_into_u32s();
    let outputs = outputs
        .chunks_exact(MEMORY_VALUES_LIMBS)
        .map(|chunk| array::from_fn(|i| M31::from_u32_unchecked(chunk[i])))
        .collect_vec();
    let program = program
        .chunks_exact(MEMORY_VALUES_LIMBS)
        .map(|chunk| array::from_fn(|i| M31::from_u32_unchecked(chunk[i])))
        .collect();

    let ppt_root = cairo_proof.extended_stark_proof.proof.commitments[0];
    let verifier_config = CairoVerifierConfig {
        preprocessed_root: ppt_root.into(),
        proof_config,
        program,
        n_outputs: cairo_proof.claim.public_data.public_memory.output.len(),
        preprocessed_trace_variant: cairo_proof.preprocessed_trace_variant,
    };

    verify_fixed_cairo_subcircuit(&verifier_config, proof, public_claim, outputs)
}

/// Builds & proves the Cairo verifier circuit (post-processed to expose 5
/// outputs in the multiverifier's convention) on the privacy proof fixture.
fn prove_cairo_and_prepare() -> SubCircuitInput<QM31> {
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
    let mut context =
        verify_cairo_with_component_set_inline(&cairo_proof, privacy_components()).unwrap();
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
    );

    let preprocessed_root: HashValue<QM31> =
        circuit_proof.stark_proof.as_ref().expect("proving failed").proof.commitments[0].into();

    let preprocessed_column_ids = preprocessed.preprocessed_trace.ids();
    let proof_config = ProofConfig::from_components(
        &all_circuit_components::<QM31>(),
        vec![true; all_circuit_components::<QM31>().len()],
        preprocessed_column_ids.len(),
        &circuit_proof.pcs_config,
        INTERACTION_POW_BITS,
    );

    let config = CircuitConfig {
        config: circuit_proof.pcs_config,
        output_addresses: preprocessed.params.output_addresses.clone(),
        n_blake_gates: preprocessed.params.n_blake_gates,
        preprocessed_column_ids,
        preprocessed_root,
    };

    let (proof, public_data) =
        prepare_circuit_proof_for_circuit_verifier(circuit_proof, &proof_config);

    SubCircuitInput { proof, circuit_public_data: public_data, config, is_multiverifier: false }
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
    let (pp_multiverifier, _) =
        pp_multiverifier_circuit_from_subcircuit(&pp_cairo_circuit, pcs_config, Some(target_padding));
    let blob = build_multiverifier_blob(&pp_multiverifier, pcs_config.fri_config.log_blowup_factor);
    println!("preprocessed_column_ids: {:?}", blob.preprocessed_column_ids);
    println!("n_blake_gates: {}", blob.n_blake_gates);
    println!("{:?}", build_multiverifier_metadata(blob));
}
// ---------- end of discovery part

const MULTIVERIFIER_N_BLAKE_GATES: usize = 10938;
const MULTIVERIFIER_METADATA_N_BLAKE_GATES_POW_TWO: u32 = 16384;
const MULTIVERIFIER_METADATA_OUTPUT_ADDRESSES: [u32; N_OUTPUTS] = [681800, 681801, 3, 4, 2];
const MULTIVERIFIER_METADATA_PREPROCESSED_ROOT: [u32; 8] =
    [1315161826, 1461131136, 1463491267, 1443815472, 1063125447, 1738526006, 632815449, 1636391323];
const MULTIVERIFIER_N_PREPROCESSED_COLUMNS: usize = 63;
fn multiverifier_preprocessed_column_ids() -> [PreProcessedColumnId; 63] {
    [
        PreProcessedColumnId { id: "blake_sigma_0".to_string() },
        PreProcessedColumnId { id: "blake_sigma_1".to_string() },
        PreProcessedColumnId { id: "blake_sigma_2".to_string() },
        PreProcessedColumnId { id: "blake_sigma_3".to_string() },
        PreProcessedColumnId { id: "blake_sigma_4".to_string() },
        PreProcessedColumnId { id: "blake_sigma_5".to_string() },
        PreProcessedColumnId { id: "blake_sigma_6".to_string() },
        PreProcessedColumnId { id: "blake_sigma_7".to_string() },
        PreProcessedColumnId { id: "blake_sigma_8".to_string() },
        PreProcessedColumnId { id: "blake_sigma_9".to_string() },
        PreProcessedColumnId { id: "blake_sigma_10".to_string() },
        PreProcessedColumnId { id: "blake_sigma_11".to_string() },
        PreProcessedColumnId { id: "blake_sigma_12".to_string() },
        PreProcessedColumnId { id: "blake_sigma_13".to_string() },
        PreProcessedColumnId { id: "blake_sigma_14".to_string() },
        PreProcessedColumnId { id: "blake_sigma_15".to_string() },
        PreProcessedColumnId { id: "m31_to_u32_input_addr".to_string() },
        PreProcessedColumnId { id: "m31_to_u32_output_addr".to_string() },
        PreProcessedColumnId { id: "m31_to_u32_multiplicity".to_string() },
        PreProcessedColumnId { id: "seq_4".to_string() },
        PreProcessedColumnId { id: "bitwise_xor_4_0".to_string() },
        PreProcessedColumnId { id: "bitwise_xor_4_1".to_string() },
        PreProcessedColumnId { id: "bitwise_xor_4_2".to_string() },
        PreProcessedColumnId { id: "t0".to_string() },
        PreProcessedColumnId { id: "t1".to_string() },
        PreProcessedColumnId { id: "finalize_flag".to_string() },
        PreProcessedColumnId { id: "state_before_addr".to_string() },
        PreProcessedColumnId { id: "state_after_addr".to_string() },
        PreProcessedColumnId { id: "message0_addr".to_string() },
        PreProcessedColumnId { id: "message1_addr".to_string() },
        PreProcessedColumnId { id: "message2_addr".to_string() },
        PreProcessedColumnId { id: "message3_addr".to_string() },
        PreProcessedColumnId { id: "compress_enabler".to_string() },
        PreProcessedColumnId { id: "final_state_addr".to_string() },
        PreProcessedColumnId { id: "blake_output0_addr".to_string() },
        PreProcessedColumnId { id: "blake_output1_addr".to_string() },
        PreProcessedColumnId { id: "blake_output0_mults".to_string() },
        PreProcessedColumnId { id: "blake_output1_mults".to_string() },
        PreProcessedColumnId { id: "seq_14".to_string() },
        PreProcessedColumnId { id: "bitwise_xor_7_0".to_string() },
        PreProcessedColumnId { id: "bitwise_xor_7_1".to_string() },
        PreProcessedColumnId { id: "bitwise_xor_7_2".to_string() },
        PreProcessedColumnId { id: "seq_15".to_string() },
        PreProcessedColumnId { id: "seq_16".to_string() },
        PreProcessedColumnId { id: "bitwise_xor_8_0".to_string() },
        PreProcessedColumnId { id: "bitwise_xor_8_1".to_string() },
        PreProcessedColumnId { id: "bitwise_xor_8_2".to_string() },
        PreProcessedColumnId { id: "eq_in0_address".to_string() },
        PreProcessedColumnId { id: "eq_in1_address".to_string() },
        PreProcessedColumnId { id: "bitwise_xor_9_0".to_string() },
        PreProcessedColumnId { id: "bitwise_xor_9_1".to_string() },
        PreProcessedColumnId { id: "bitwise_xor_9_2".to_string() },
        PreProcessedColumnId { id: "bitwise_xor_10_0".to_string() },
        PreProcessedColumnId { id: "bitwise_xor_10_1".to_string() },
        PreProcessedColumnId { id: "bitwise_xor_10_2".to_string() },
        PreProcessedColumnId { id: "qm31_ops_add_flag".to_string() },
        PreProcessedColumnId { id: "qm31_ops_sub_flag".to_string() },
        PreProcessedColumnId { id: "qm31_ops_mul_flag".to_string() },
        PreProcessedColumnId { id: "qm31_ops_pointwise_mul_flag".to_string() },
        PreProcessedColumnId { id: "qm31_ops_in0_address".to_string() },
        PreProcessedColumnId { id: "qm31_ops_in1_address".to_string() },
        PreProcessedColumnId { id: "qm31_ops_out_address".to_string() },
        PreProcessedColumnId { id: "qm31_ops_mults".to_string() },
    ]
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
    let subcircuit_input = prove_cairo_and_prepare();
    assert_eq!(subcircuit_input.config.output_addresses.len(), N_OUTPUTS);
    let subcircuit_pcs_config = subcircuit_input.config.config;

    // TODO: these metadata configs could be consts.
    let metadata_cairo = Metadata::<QM31>::from_config(&subcircuit_input.config);
    let multiverifier_metadata = make_struct();
    let metadata_tree = MetadataTree::<QM31>::commit(metadata_cairo, multiverifier_metadata);

    // TODO: should be const.
    let subcircuit_config = SubCircuitConfig {
        pcs_config: subcircuit_pcs_config,
        n_outputs: subcircuit_input.config.output_addresses.len(),
        preprocessed_column_ids: subcircuit_input.config.preprocessed_column_ids.clone(),
    };
    let mut multiverifier_context = build_multiverifier_circuit::<QM31>(
        subcircuit_input,
        // TODO: change to subcircuit_input.clone() when Clone becomes available.
        prove_cairo_and_prepare(),
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
    );
    let multi_proof_config = ProofConfig::from_components(
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
    let subcircuit_input = prove_cairo_and_prepare();
    let subcircuit_pcs_config = subcircuit_input.config.config;

    // TODO: these metadata configs could be consts.
    let metadata_cairo = Metadata::<QM31>::from_config(&subcircuit_input.config);
    let multiverifier_metadata = make_struct();
    let metadata_tree = MetadataTree::<QM31>::commit(metadata_cairo, multiverifier_metadata);
    let metadata_root = metadata_tree.root;

    // 3. Reconstruct the multi's expected `CircuitPublicData`. Multi's outputs are `[H_lo, H_hi,
    //    hash_of_payloads_lo, hash_of_payloads_hi, u]`, where `hash_of_payloads =
    //    blake([cairo_payload, cairo_payload], 64)` over the two identical Cairo payload pairs
    //    (Test A's two Cairo proofs are deterministic and identical, so so are this Test's).
    let cairo_payload_lo = subcircuit_input.circuit_public_data.output_values[0];
    let cairo_payload_hi = subcircuit_input.circuit_public_data.output_values[1];
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
        preprocessed_column_ids: multiverifier_preprocessed_column_ids().to_vec(),
        preprocessed_root: HashValue::from(MULTIVERIFIER_METADATA_PREPROCESSED_ROOT),
    };

    // 5. Multi's `ProofConfig` for deserialization.
    let multi_proof_config = ProofConfig::from_components(
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
        circuit_public_data: multi_public_data,
        config: multi_config,
        is_multiverifier: true,
    };
    let cairo_input = SubCircuitInput {
        proof: subcircuit_input.proof,
        circuit_public_data: subcircuit_input.circuit_public_data,
        config: subcircuit_input.config,
        is_multiverifier: false,
    };
    let subcircuit_config = SubCircuitConfig {
        pcs_config: subcircuit_pcs_config,
        n_outputs: N_OUTPUTS,
        preprocessed_column_ids: multiverifier_preprocessed_column_ids().to_vec(),
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
