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
    CairoVerifierConfig, build_cairo_verifier_circuit, enabled_components,
    prepare_cairo_proof_for_circuit_verifier, verify_fixed_cairo_circuit,
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
use circuits::circuit::{Add, Output};
use circuits::context::Context;
use circuits::finalize_constants::finalize_constants;
use circuits::ivalue::{IValue, NoValue, qm31_from_u32s};
use circuits::wrappers::M31Wrapper;
use circuits_stark_verifier::constraint_eval::CircuitEval;
use circuits_stark_verifier::proof::{Proof, ProofConfig, empty_proof};
use itertools::{Itertools, zip_eq};
use num_traits::{One, Zero};
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;

use super::verify_test::build_fibonacci_context_with_5_outputs;
use super::{Input, Metadata, SubCircuitConfig, build_multiverifier_circuit};
use crate::padding::pad_components_to_target_counts;

/// Targets matching the Cairo-verifier circuit's component-size brackets
/// (verified empirically — see `explore_cairo_vs_multi_preprocessed_column_ids`
/// in `verify_test.rs`).
///
/// Each value is the *natural* (pre-`pad_*`) count we want the multi-verifier
/// to land at. Choosing the bracket maxima (powers of two) means
/// `from_finalized_circuit`'s `pad_*` functions add nothing on top.
mod cairo_targets {
    pub(super) const EQ: usize = 1 << 17; // 131_072
    pub(super) const QM31_OPS: usize = 1 << 21; // 2_097_152
    pub(super) const N_BLAKE_GATES: usize = 1 << 13; // 8_192
    pub(super) const N_BLAKE_COMPRESS_ROWS: usize = 1 << 14; // 16_384
}

/// Builds a NoValue multiverifier circuit (verifying two Fibonacci-shaped
/// inputs) and pads it up into Cairo's column-size brackets.
fn build_padded_multi_for_cairo() -> Context<NoValue> {
    // Derive a Fibonacci `CircuitConfig` so the inner `verify()` calls have a
    // realistic shape. The actual values don't matter under `NoValue`.
    let mut fib_ctx = build_fibonacci_context_with_5_outputs::<NoValue>();
    finalize_constants(&mut fib_ctx);
    let pp_fib = PreprocessedCircuit::preprocess_circuit(&mut fib_ctx);

    let log_blowup = PcsConfig::default().fri_config.log_blowup_factor;
    let inner_pcs_config = PcsConfig {
        lifting_log_size: Some(pp_fib.params.trace_log_size as u32 + log_blowup),
        ..PcsConfig::default()
    };

    let placeholder_root = HashValue(QM31::zero(), QM31::zero());
    let make_input = || {
        let proof_config = ProofConfig::from_components(
            &all_circuit_components::<NoValue>(),
            vec![true; all_circuit_components::<NoValue>().len()],
            pp_fib.preprocessed_trace.ids().len(),
            &inner_pcs_config,
            INTERACTION_POW_BITS,
        );
        Input {
            proof: empty_proof(&proof_config),
            circuit_public_data: CircuitPublicData {
                output_values: vec![QM31::zero(); 4],
            },
            config: CircuitConfig {
                config: inner_pcs_config,
                output_addresses: pp_fib.params.output_addresses.clone(),
                n_blake_gates: pp_fib.params.n_blake_gates,
                preprocessed_column_ids: pp_fib.preprocessed_trace.ids(),
                preprocessed_root: placeholder_root,
            },
            is_multiverifier: false,
            other_hash: placeholder_root,
        }
    };
    let subcircuit_config = SubCircuitConfig {
        pcs_config: inner_pcs_config,
        n_outputs: 5,
        preprocessed_column_ids: pp_fib.preprocessed_trace.ids(),
    };

    let mut multi_ctx = build_multiverifier_circuit::<NoValue>(
        make_input(),
        make_input(),
        subcircuit_config,
        placeholder_root,
    );

    // The padding helper is meant to be called *after* `build_multiverifier_circuit`
    // (which has already run `finalize_constants` + `finalize_guessed_vars`) and
    // *before* `PreprocessedCircuit::preprocess_circuit`. The new gates we add
    // here yield their own outputs (via Add/Blake) so don't break yield
    // bookkeeping; the freshly-yielded vars are unused (no consumer), which
    // would fail `check_vars_used` but doesn't matter for a topology-only
    // comparison.
    pad_components_to_target_counts(
        &mut multi_ctx,
        cairo_targets::EQ,
        cairo_targets::QM31_OPS,
        cairo_targets::N_BLAKE_GATES,
        cairo_targets::N_BLAKE_COMPRESS_ROWS,
    );

    multi_ctx
}

/// After padding multi up into Cairo's brackets, both circuits should produce
/// identical `preprocessed_column_ids` (in the same order) when preprocessed.
#[test]
fn test_padded_multi_matches_cairo_preprocessed_column_ids() {
    // --- Cairo verifier ---
    let cairo_proof_log_blowup_factor = 3;
    let cairo_verifier_config = privacy_cairo_verifier_config(cairo_proof_log_blowup_factor);
    let mut cairo_ctx = build_cairo_verifier_circuit(&cairo_verifier_config);
    let pp_cairo = PreprocessedCircuit::preprocess_circuit(&mut cairo_ctx);
    let cairo_ids = pp_cairo.preprocessed_trace.ids();

    // --- Padded multiverifier ---
    let mut multi_ctx = build_padded_multi_for_cairo();
    let pp_multi = PreprocessedCircuit::preprocess_circuit(&mut multi_ctx);
    let multi_ids = pp_multi.preprocessed_trace.ids();

    println!(
        "cairo: trace_log_size = {}, n_columns = {}",
        pp_cairo.params.trace_log_size,
        cairo_ids.len(),
    );
    println!(
        "multi: trace_log_size = {}, n_columns = {}",
        pp_multi.params.trace_log_size,
        multi_ids.len(),
    );

    if cairo_ids != multi_ids {
        // Print a side-by-side diff so the failure mode is obvious.
        let max_len = std::cmp::max(cairo_ids.len(), multi_ids.len());
        let mut first_diff = None;
        println!(
            "\n{:>3}  {:<35}  {:<35}  match",
            "i", "cairo verifier", "multiverifier (padded)",
        );
        println!("{}", "-".repeat(90));
        for i in 0..max_len {
            let c = cairo_ids.get(i).map(|x| x.id.as_str()).unwrap_or("(none)");
            let m = multi_ids.get(i).map(|x| x.id.as_str()).unwrap_or("(none)");
            let same = c == m;
            if !same && first_diff.is_none() {
                first_diff = Some(i);
            }
            println!(
                "{:>3}  {:<35}  {:<35}  {}",
                i,
                c,
                m,
                if same { "✓" } else { "✗" },
            );
        }
        panic!("preprocessed_column_ids differ; first mismatch at index {first_diff:?}");
    }

    println!(
        "\npreprocessed_column_ids match across all {} entries.",
        cairo_ids.len(),
    );
}

// ---------- Tests A & B: prove → save → load → verify (Cairo edition) ----------

const MULTI_OF_CAIRO_PROOF_PATH: &str =
    "/tmp/circuit_multiverifier_test_multi_of_cairo_proof.bin";

/// Variant of `ProofBundle` from `multi_fibonacci_test`, local to this module.
struct ProofBundle {
    proof: Proof<QM31>,
    public_data: CircuitPublicData,
    config: CircuitConfig,
}

/// The Cairo verifier circuit naturally has 3 outputs:
///   `[output_hash.0, output_hash.1, u]`.
/// To fit the multiverifier's 5-slot `[H_or_0, H_or_0, payload, payload, u]`
/// convention without modifying the Cairo verifier source, we post-process
/// the context after `build_cairo_verifier_circuit` / `verify_cairo_*`
/// returns: insert 2 zero-yielded outputs at the *front* of the output list.
///
/// The new `dummy0`/`dummy1` vars are created via `new_var` (so they don't
/// touch `guessed_vars`, which `verify_fixed_cairo_circuit` has already
/// drained) and yielded via trivial `Add { 0, 0, dummy_i }` gates so the
/// logup balance survives.
///
/// After this, the output order becomes
/// `[dummy0=0, dummy1=0, output_hash.0, output_hash.1, u]` — exactly the
/// multiverifier's expected `[H_or_0, H_or_0, payload, payload, u]` layout
/// for variant A (a leaf circuit).
fn add_two_dummy_outputs_at_front<Value: IValue>(context: &mut Context<Value>) {
    let zero_idx = context.zero().idx;
    let dummy0 = context.new_var(Value::from_qm31(QM31::zero()));
    let dummy1 = context.new_var(Value::from_qm31(QM31::zero()));
    context
        .circuit
        .add
        .push(Add { in0: zero_idx, in1: zero_idx, out: dummy0.idx });
    context
        .circuit
        .add
        .push(Add { in0: zero_idx, in1: zero_idx, out: dummy1.idx });
    // Insert order matters: `insert(0, ...)` shifts everything else right, so
    // pushing dummy1 first (at index 0) then dummy0 (also at index 0) leaves
    // [dummy0, dummy1, ...originals...].
    context.circuit.output.insert(0, Output { in0: dummy1.idx });
    context.circuit.output.insert(0, Output { in0: dummy0.idx });
}

/// Inlined copy of `circuit_cairo_verifier::test::verify_cairo_with_component_set`
/// (which lives behind `#[cfg(test)]` in the cairo_verifier crate, so it isn't
/// callable from outside). Verifies a `CairoProof` with the given component
/// set and returns the resulting QM31 context.
fn verify_cairo_with_component_set_inline(
    cairo_proof: &CairoProof<Blake2sM31MerkleHasher>,
    component_set: HashSet<&str>,
) -> Result<Context<QM31>, String> {
    let FlatClaim {
        component_enable_bits,
        component_log_sizes: _,
        public_data: _,
    } = cairo_proof.claim.flatten_claim();
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
        cairo_proof
            .preprocessed_trace_variant
            .to_preprocessed_trace()
            .ids()
            .len(),
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

    verify_fixed_cairo_circuit(&verifier_config, proof, public_claim, outputs)
}

/// Builds & proves the Cairo verifier circuit (post-processed to expose 5
/// outputs in the multiverifier's convention) on the privacy proof fixture.
fn prove_cairo_and_prepare() -> ProofBundle {
    // Load the cached Cairo proof.
    let proof_path = get_proof_file_path("privacy");
    let proof_file = File::open(proof_path).expect("test_data/privacy/proof.bin must exist");
    let cairo_proof = binary_deserialize_from_file(&proof_file).expect("read cairo proof");

    let cairo_proof_log_blowup_factor = 3;
    let const_config = privacy_cairo_verifier_config(cairo_proof_log_blowup_factor);

    // NoValue topology for preprocessing — must be modified the *same* way as
    // the QM31 context below, otherwise the preprocessed_root won't match the
    // commitment that the prover ends up producing.
    let mut novalue_context = build_cairo_verifier_circuit(&const_config);
    add_two_dummy_outputs_at_front(&mut novalue_context);
    let preprocessed = PreprocessedCircuit::preprocess_circuit(&mut novalue_context);

    // QM31 context with values from the proof.
    let mut context =
        verify_cairo_with_component_set_inline(&cairo_proof, privacy_components()).unwrap();
    add_two_dummy_outputs_at_front(&mut context);
    pad_components_to_powers_of_two(&mut context);

    let circuit_proof = prove_circuit_assignment(
        context.values(),
        &preprocessed,
        &BaseColumnPool::<SimdBackend>::new(),
        PcsConfig::default(),
    );

    let preprocessed_root: HashValue<QM31> = circuit_proof
        .stark_proof
        .as_ref()
        .expect("proving failed")
        .proof
        .commitments[0]
        .into();

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

    ProofBundle { proof, public_data, config }
}

/// Hashes a `Metadata<QM31>` exactly the way `Metadata::hash` does in-circuit.
fn hash_metadata(m: Metadata<QM31>) -> HashValue<QM31> {
    let qm31s = m.serialize_to_qm31();
    blake_qm31(&qm31s, 16 * qm31s.len())
}

/// Hashes two leaf hashes into a parent node, matching the in-circuit
/// `merkle::hash_node` (`blake([left.0, left.1, right.0, right.1], 64)`).
fn hash_node(left: HashValue<QM31>, right: HashValue<QM31>) -> HashValue<QM31> {
    blake_qm31(&[left.0, left.1, right.0, right.1], 64)
}

/// Builds a `Metadata<QM31>` for the multiverifier circuit (variant B).
fn multiverifier_metadata(
    output_addresses: &[usize],
    n_blake_gates: usize,
    preprocessed_root: HashValue<QM31>,
) -> Metadata<QM31> {
    Metadata {
        bit: M31Wrapper::new_unsafe(QM31::one()),
        n_blake_gates_pow_two: M31Wrapper::new_unsafe(QM31::from(
            n_blake_gates.next_power_of_two(),
        )),
        output_addresses: output_addresses
            .iter()
            .map(|x| M31Wrapper::new_unsafe(QM31::from(*x)))
            .collect(),
        preprocessed_root,
    }
}

/// Structural metadata of the *padded* multiverifier circuit (the one whose
/// `preprocessed_column_ids` match the Cairo verifier's). Same shape Tests A
/// and B both target — the recursion fixed point is at the same trace size as
/// Cairo (`trace_log_size = 21`).
struct MultiCircuitMetadata {
    preprocessed_root: HashValue<QM31>,
    output_addresses: Vec<usize>,
    n_blake_gates: usize,
    preprocessed_column_ids:
        Vec<stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId>,
    /// `pcs_config` at which the multi proof itself is/was generated.
    outer_pcs_config: PcsConfig,
}

/// Builds a NoValue padded multi (verifying two Cairo-shaped Inputs) and
/// extracts every piece of metadata Tests A and B need. Mirrors
/// `extract_multi_metadata` in `multi_fibonacci_test`, but inputs use the
/// Cairo `CircuitConfig` and the multi gets padded after construction.
fn extract_padded_multi_metadata(cairo_config: &CircuitConfig) -> MultiCircuitMetadata {
    use stwo::core::poly::circle::CanonicCoset;
    use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
    use stwo::prover::CommitmentTreeProver;
    use stwo::prover::poly::circle::PolyOps;

    let inner_pcs_config = cairo_config.config;
    let placeholder_root = HashValue(QM31::zero(), QM31::zero());
    let proof_config = ProofConfig::from_components(
        &all_circuit_components::<NoValue>(),
        vec![true; all_circuit_components::<NoValue>().len()],
        cairo_config.preprocessed_column_ids.len(),
        &inner_pcs_config,
        INTERACTION_POW_BITS,
    );
    let make_novalue_input = || Input {
        proof: empty_proof(&proof_config),
        circuit_public_data: CircuitPublicData {
            output_values: vec![QM31::zero(); 4],
        },
        config: CircuitConfig {
            config: inner_pcs_config,
            output_addresses: cairo_config.output_addresses.clone(),
            n_blake_gates: cairo_config.n_blake_gates,
            preprocessed_column_ids: cairo_config.preprocessed_column_ids.clone(),
            preprocessed_root: cairo_config.preprocessed_root,
        },
        is_multiverifier: false,
        other_hash: placeholder_root,
    };
    let subcircuit_config = SubCircuitConfig {
        pcs_config: inner_pcs_config,
        n_outputs: cairo_config.output_addresses.len(),
        preprocessed_column_ids: cairo_config.preprocessed_column_ids.clone(),
    };
    let mut multi_ctx = build_multiverifier_circuit::<NoValue>(
        make_novalue_input(),
        make_novalue_input(),
        subcircuit_config,
        placeholder_root,
    );
    // Pad up into Cairo's brackets so the multi's preprocessed_column_ids
    // match (verified by `test_padded_multi_matches_cairo_preprocessed_column_ids`).
    pad_components_to_target_counts(
        &mut multi_ctx,
        cairo_targets::EQ,
        cairo_targets::QM31_OPS,
        cairo_targets::N_BLAKE_GATES,
        cairo_targets::N_BLAKE_COMPRESS_ROWS,
    );
    let pp_multi = PreprocessedCircuit::preprocess_circuit(&mut multi_ctx);

    // Outer pcs_config at which the multi proof will be made (auto-lifted).
    let mut outer_pcs_config = PcsConfig::default();
    let trace_log_size = pp_multi.params.trace_log_size;
    let log_blowup = outer_pcs_config.fri_config.log_blowup_factor;
    let lifting_log_size = trace_log_size + log_blowup;
    outer_pcs_config.lifting_log_size = Some(lifting_log_size);

    // Compute the multi's preprocessed_root via the commitment tree.
    let twiddles = SimdBackend::precompute_twiddles(
        CanonicCoset::new(trace_log_size + std::cmp::max(log_blowup, 1))
            .circle_domain()
            .half_coset,
    );
    let preprocessed_trace = pp_multi.preprocessed_trace.get_trace::<SimdBackend>();
    let preprocessed_trace_polys = SimdBackend::interpolate_columns(preprocessed_trace, &twiddles);
    let preprocessed_tree = CommitmentTreeProver::<SimdBackend, Blake2sM31MerkleChannel>::new(
        preprocessed_trace_polys,
        log_blowup,
        &twiddles,
        true,
        Some(lifting_log_size),
        &BaseColumnPool::<SimdBackend>::new(),
    );
    let preprocessed_root: HashValue<QM31> = preprocessed_tree.commitment.root().into();

    MultiCircuitMetadata {
        preprocessed_root,
        output_addresses: pp_multi.params.output_addresses.clone(),
        n_blake_gates: pp_multi.params.n_blake_gates,
        preprocessed_column_ids: pp_multi.preprocessed_trace.ids(),
        outer_pcs_config,
    }
}

/// Test A (Cairo): build & prove a *padded* multiverifier verifying two Cairo
/// proofs, with the correct recursion-tree `metadata_root H = hash_node(h_cairo, h_multi)`,
/// and write the resulting `Proof<QM31>` bytes to disk for Test B to consume.
#[test]
fn test_a_prove_multiverifier_of_cairos_and_save() {
    // 1. Two Cairo proofs (deterministic — same fixture proven twice).
    let bundle1 = prove_cairo_and_prepare();
    let bundle2 = prove_cairo_and_prepare();
    assert_eq!(
        bundle1.config.preprocessed_root, bundle2.config.preprocessed_root,
        "Cairo proving must be deterministic"
    );
    assert_eq!(bundle1.config.output_addresses.len(), 5, "expected 5 outputs after dummy injection");
    let inner_pcs_config = bundle1.config.config;

    // 2. h_cairo (variant A descriptor hash).
    let h_cairo = hash_metadata(Metadata::<QM31>::from_config(&bundle1.config, false));

    // 3. Padded multi structural metadata + h_multi.
    let multi_meta = extract_padded_multi_metadata(&bundle1.config);
    let h_multi = hash_metadata(multiverifier_metadata(
        &multi_meta.output_addresses,
        multi_meta.n_blake_gates,
        multi_meta.preprocessed_root,
    ));

    // 4. Recursion-tree-wide metadata root.
    let metadata_root = hash_node(h_cairo, h_multi);

    // 5. Build the QM31 padded multi verifying both Cairo proofs.
    let p1 = Input {
        proof: bundle1.proof,
        circuit_public_data: bundle1.public_data,
        config: bundle1.config,
        is_multiverifier: false,
        other_hash: h_multi,
    };
    let p2 = Input {
        proof: bundle2.proof,
        circuit_public_data: bundle2.public_data,
        config: bundle2.config,
        is_multiverifier: false,
        other_hash: h_multi,
    };
    let subcircuit_config = SubCircuitConfig {
        pcs_config: inner_pcs_config,
        n_outputs: p1.config.output_addresses.len(),
        preprocessed_column_ids: p1.config.preprocessed_column_ids.clone(),
    };
    let mut multi_ctx =
        build_multiverifier_circuit::<QM31>(p1, p2, subcircuit_config, metadata_root);
    // Apply the same Cairo-target padding the metadata extraction did.
    pad_components_to_target_counts(
        &mut multi_ctx,
        cairo_targets::EQ,
        cairo_targets::QM31_OPS,
        cairo_targets::N_BLAKE_GATES,
        cairo_targets::N_BLAKE_COMPRESS_ROWS,
    );
    multi_ctx.validate_circuit();

    // 6. Prove the multi.
    let pp_multi = PreprocessedCircuit::preprocess_circuit(&mut multi_ctx);
    let multi_circuit_proof = prove_circuit_assignment(
        multi_ctx.values(),
        &pp_multi,
        &BaseColumnPool::<SimdBackend>::new(),
        PcsConfig::default(),
    );
    let resolved_outer_pcs_config = multi_circuit_proof.pcs_config;
    assert_eq!(resolved_outer_pcs_config, multi_meta.outer_pcs_config);
    assert_eq!(
        multi_meta.preprocessed_root,
        Into::<HashValue<QM31>>::into(
            multi_circuit_proof.stark_proof.as_ref().unwrap().proof.commitments[0],
        ),
        "padded multi preprocessed_root must match what the prover commits",
    );

    // 7. Serialize the multi proof to disk.
    let multi_proof_config = ProofConfig::from_components(
        &all_circuit_components::<QM31>(),
        vec![true; all_circuit_components::<QM31>().len()],
        multi_meta.preprocessed_column_ids.len(),
        &resolved_outer_pcs_config,
        INTERACTION_POW_BITS,
    );
    let (multi_proof, _multi_public_data) =
        prepare_circuit_proof_for_circuit_verifier(multi_circuit_proof, &multi_proof_config);
    let mut serialized = vec![];
    multi_proof.serialize(&mut serialized);
    std::fs::write(MULTI_OF_CAIRO_PROOF_PATH, &serialized).expect("write multi-of-cairo proof");

    println!(
        "Saved multi-of-cairo proof ({} bytes) to {MULTI_OF_CAIRO_PROOF_PATH}",
        serialized.len()
    );
    println!("multi trace_log_size: {}", pp_multi.params.trace_log_size);
    println!("resolved outer pcs_config: {:?}", resolved_outer_pcs_config);
}

/// Test B (Cairo): load Test A's multi proof, prove a fresh Cairo proof, build
/// a *second-level* padded multi that verifies `[multi_proof, cairo_proof]`,
/// and value-validate it. The second-level multi is the same circuit as the
/// first-level one (same padding, same inner pcs_config) — recursion fixed
/// point at `trace_log_size = 21`.
#[test]
fn test_b_verify_multi_proof_and_cairo_proof_with_multiverifier() {
    use circuit_serialize::deserialize::deserialize_proof_with_config;

    // 1. Fresh Cairo proof (also gives us the cairo_config for everything else).
    let cairo_bundle = prove_cairo_and_prepare();
    let inner_pcs_config = cairo_bundle.config.config;

    // 2. Re-derive h_cairo, h_multi, H deterministically.
    let h_cairo = hash_metadata(Metadata::<QM31>::from_config(&cairo_bundle.config, false));
    let multi_meta = extract_padded_multi_metadata(&cairo_bundle.config);
    let h_multi = hash_metadata(multiverifier_metadata(
        &multi_meta.output_addresses,
        multi_meta.n_blake_gates,
        multi_meta.preprocessed_root,
    ));
    let metadata_root = hash_node(h_cairo, h_multi);

    // 3. Reconstruct the multi's expected `CircuitPublicData`. Multi's outputs
    //    are `[H_lo, H_hi, hash_of_payloads_lo, hash_of_payloads_hi, u]`,
    //    where `hash_of_payloads = blake([cairo_payload, cairo_payload], 64)`
    //    over the two identical Cairo payload pairs (Test A's two Cairo proofs
    //    are deterministic and identical, so so are this Test's).
    let cairo_payload_lo = cairo_bundle.public_data.output_values[2];
    let cairo_payload_hi = cairo_bundle.public_data.output_values[3];
    let hash_of_payloads = blake_qm31(
        &[
            cairo_payload_lo,
            cairo_payload_hi,
            cairo_payload_lo,
            cairo_payload_hi,
        ],
        64,
    );
    let u_value = qm31_from_u32s(0, 0, 1, 0);
    let multi_public_data = CircuitPublicData {
        output_values: vec![
            metadata_root.0,
            metadata_root.1,
            hash_of_payloads.0,
            hash_of_payloads.1,
            u_value,
        ],
    };

    // 4. Multi's `CircuitConfig` (same shape as Test A produced).
    let multi_config = CircuitConfig {
        config: multi_meta.outer_pcs_config,
        output_addresses: multi_meta.output_addresses.clone(),
        n_blake_gates: multi_meta.n_blake_gates,
        preprocessed_column_ids: multi_meta.preprocessed_column_ids.clone(),
        preprocessed_root: multi_meta.preprocessed_root,
    };

    // 5. Multi's `ProofConfig` for deserialization.
    let multi_proof_config = ProofConfig::from_components(
        &all_circuit_components::<QM31>(),
        vec![true; all_circuit_components::<QM31>().len()],
        multi_meta.preprocessed_column_ids.len(),
        &multi_meta.outer_pcs_config,
        INTERACTION_POW_BITS,
    );

    // 6. Load Test A's saved proof.
    let bytes = std::fs::read(MULTI_OF_CAIRO_PROOF_PATH)
        .expect("Test A (cairo) must run first to write the multi proof to disk");
    let mut slice = bytes.as_slice();
    let multi_proof = deserialize_proof_with_config(&mut slice, &multi_proof_config)
        .expect("deserialize multi-of-cairo proof");

    // 7. Build the second-level padded multi:
    //    - The multi proof sits at leaf index 1 (`bit = 1`), sibling = `h_cairo`.
    //    - The cairo proof sits at leaf index 0 (`bit = 0`), sibling = `h_multi`.
    let multi_input = Input {
        proof: multi_proof,
        circuit_public_data: multi_public_data,
        config: multi_config,
        is_multiverifier: true,
        other_hash: h_cairo,
    };
    let cairo_input = Input {
        proof: cairo_bundle.proof,
        circuit_public_data: cairo_bundle.public_data,
        config: cairo_bundle.config,
        is_multiverifier: false,
        other_hash: h_multi,
    };
    let subcircuit_config = SubCircuitConfig {
        pcs_config: inner_pcs_config,
        n_outputs: multi_meta.output_addresses.len(),
        preprocessed_column_ids: multi_meta.preprocessed_column_ids.clone(),
    };
    let mut context = build_multiverifier_circuit::<QM31>(
        multi_input,
        cairo_input,
        subcircuit_config,
        metadata_root,
    );
    // Same padding as Test A — keeps the second-level multi byte-identical to
    // the first-level one (recursion fixed point).
    pad_components_to_target_counts(
        &mut context,
        cairo_targets::EQ,
        cairo_targets::QM31_OPS,
        cairo_targets::N_BLAKE_GATES,
        cairo_targets::N_BLAKE_COMPRESS_ROWS,
    );

    // Note: skip `check_vars_used` here — `pad_components_to_target_counts`
    // intentionally adds dummy gates whose outputs are unconsumed (`check_vars_used`
    // would flag them). `validate_circuit` is the value-level invariant we
    // care about.
    context.circuit.check_yields();
    context.validate_circuit();
}
