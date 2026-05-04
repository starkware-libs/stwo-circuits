use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_prover::prover::{
    BaseColumnPool, SimdBackend, prepare_circuit_proof_for_circuit_verifier,
    prove_circuit_assignment,
};
use circuit_serialize::serialize::CircuitSerialize;
use circuit_verifier::{
    statement::{INTERACTION_POW_BITS, all_circuit_components},
    verify::{CircuitConfig, CircuitPublicData},
};
use circuits::{
    blake::{HashValue, blake_qm31},
    finalize_constants::finalize_constants,
    ivalue::NoValue,
    wrappers::M31Wrapper,
};
use circuits_stark_verifier::proof::{ProofConfig, empty_proof};
use num_traits::{One, Zero};
use stwo::core::{fields::qm31::QM31, pcs::PcsConfig};

use super::verify_test::build_fibonacci_context_with_5_outputs;
use super::{Input, Metadata, build_multiverifier_circuit};

/// Result of proving a single Fibonacci circuit, bundled so the caller can derive
/// both the per-proof [`Input`] data and the structural metadata needed to compute
/// the recursion-tree-wide `metadata_root`.
struct FibProofBundle {
    proof: circuits_stark_verifier::proof::Proof<QM31>,
    public_data: CircuitPublicData,
    config: CircuitConfig,
    pcs_config: PcsConfig,
}

fn prove_fibonacci_and_prepare() -> FibProofBundle {
    // Build the witnessed Fibonacci context.
    let mut context = build_fibonacci_context_with_5_outputs::<QM31>();
    finalize_constants(&mut context);
    context.finalize_guessed_vars();
    context.validate_circuit();

    let preprocessed_circuit = PreprocessedCircuit::preprocess_circuit(&mut context);
    let pcs_config = PcsConfig::default();
    let circuit_proof = prove_circuit_assignment(
        context.values(),
        &preprocessed_circuit,
        &BaseColumnPool::<SimdBackend>::new(),
        pcs_config,
    );

    // The actual preprocessed root that was committed during proving — read it back
    // from `commitments[0]` rather than relying on a stale hardcoded constant, since
    // the modified Fibonacci context (5 outputs) changes the trace shape.
    let preprocessed_root: HashValue<QM31> = circuit_proof
        .stark_proof
        .as_ref()
        .expect("proving failed")
        .proof
        .commitments[0]
        .into();
    // pcs_config inside `circuit_proof` has had `lifting_log_size` set by
    // `prove_circuit_assignment`; that's the size the multiverifier must expect.
    let resolved_pcs_config = circuit_proof.pcs_config;

    let preprocessed_column_ids = preprocessed_circuit.preprocessed_trace.ids();
    let proof_config = ProofConfig::from_components(
        &all_circuit_components::<QM31>(),
        vec![true; all_circuit_components::<QM31>().len()],
        preprocessed_column_ids.len(),
        &resolved_pcs_config,
        INTERACTION_POW_BITS,
    );

    let config = CircuitConfig {
        config: resolved_pcs_config,
        output_addresses: preprocessed_circuit.params.output_addresses.clone(),
        n_blake_gates: preprocessed_circuit.params.n_blake_gates,
        preprocessed_column_ids,
        preprocessed_root,
    };

    let (proof, public_data) =
        prepare_circuit_proof_for_circuit_verifier(circuit_proof, &proof_config);

    FibProofBundle { proof, public_data, config, pcs_config: resolved_pcs_config }
}

// ---------- Helpers for metadata hashing ----------

/// Builds a `Metadata<QM31>` for a leaf circuit from its [`CircuitConfig`]
/// (variant A: `bit = 0`).
fn leaf_metadata(config: &CircuitConfig) -> Metadata<QM31> {
    Metadata {
        bit: M31Wrapper::new_unsafe(QM31::zero()),
        n_blake_gates_pow_two: M31Wrapper::new_unsafe(QM31::from(
            config.n_blake_gates.next_power_of_two(),
        )),
        output_addresses: config
            .output_addresses
            .iter()
            .map(|x| M31Wrapper::new_unsafe(QM31::from(*x)))
            .collect(),
        preprocessed_root: config.preprocessed_root,
    }
}

/// Builds a `Metadata<QM31>` for the multiverifier circuit (variant B:
/// `bit = 1`) given its preprocessed shape and root.
fn multi_metadata(
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

/// All structural metadata of the multiverifier circuit at a given inner
/// `pcs_config`. Computed once and reused by Tests A and B so they end up with
/// identical multi-circuit shapes (a precondition for true recursion: the
/// second-level multi has to be the *same* circuit as the first-level one).
struct MultiCircuitMetadata {
    preprocessed_root: HashValue<QM31>,
    output_addresses: Vec<usize>,
    n_blake_gates: usize,
    preprocessed_column_ids: Vec<stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId>,
    /// `pcs_config` at which the multi proof itself is/was generated. The
    /// `lifting_log_size` here is the auto-lift used by `prove_circuit_assignment`,
    /// i.e. `multi_trace_log_size + log_blowup_factor`.
    outer_pcs_config: PcsConfig,
}

/// Builds a NoValue multi verifying two Fibonacci-shaped Inputs and extracts
/// every piece of multi structural metadata Tests A and B need.
fn extract_multi_metadata(
    fib_config: &CircuitConfig,
    inner_pcs_config: PcsConfig,
) -> MultiCircuitMetadata {
    use stwo::core::poly::circle::CanonicCoset;
    use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
    use stwo::prover::CommitmentTreeProver;
    use stwo::prover::poly::circle::PolyOps;

    let placeholder_root = HashValue(QM31::zero(), QM31::zero());
    let make_novalue_input = || {
        let proof_config = ProofConfig::from_components(
            &all_circuit_components::<NoValue>(),
            vec![true; all_circuit_components::<NoValue>().len()],
            fib_config.preprocessed_column_ids.len(),
            &inner_pcs_config,
            INTERACTION_POW_BITS,
        );
        Input {
            proof: empty_proof(&proof_config),
            circuit_public_data: CircuitPublicData {
                output_values: vec![QM31::zero(); 4],
            },
            config: CircuitConfig {
                config: fib_config.config,
                output_addresses: fib_config.output_addresses.clone(),
                n_blake_gates: fib_config.n_blake_gates,
                preprocessed_column_ids: fib_config.preprocessed_column_ids.clone(),
                preprocessed_root: fib_config.preprocessed_root,
            },
            is_multiverifier: false,
            other_hash: placeholder_root,
        }
    };
    let mut multi_ctx = build_multiverifier_circuit::<NoValue>(
        make_novalue_input(),
        make_novalue_input(),
        inner_pcs_config,
        placeholder_root,
    );
    let pp_multi = PreprocessedCircuit::preprocess_circuit(&mut multi_ctx);

    // The outer pcs_config (used to prove the multi itself) auto-lifts to
    // `multi_trace_log_size + log_blowup_factor`, matching what
    // `prove_circuit_assignment` will do.
    let mut outer_pcs_config = PcsConfig::default();
    let trace_log_size = pp_multi.params.trace_log_size;
    let log_blowup = outer_pcs_config.fri_config.log_blowup_factor;
    let lifting_log_size = trace_log_size + log_blowup;
    outer_pcs_config.lifting_log_size = Some(lifting_log_size);

    // Compute the multi's `preprocessed_root` directly via the commitment tree
    // (same code path the prover would take, but without the rest of proving).
    let twiddles = SimdBackend::precompute_twiddles(
        CanonicCoset::new(
            trace_log_size + std::cmp::max(log_blowup, /* COMPOSITION_POLYNOMIAL_LOG_DEGREE_BOUND */ 1),
        )
        .circle_domain()
        .half_coset,
    );
    let preprocessed_trace = pp_multi.preprocessed_trace.get_trace::<SimdBackend>();
    let preprocessed_trace_polys = SimdBackend::interpolate_columns(preprocessed_trace, &twiddles);
    let preprocessed_tree =
        CommitmentTreeProver::<SimdBackend, Blake2sM31MerkleChannel>::new(
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

/// Diagnostic: build a NoValue multiverifier with the given inner-proof lifting,
/// preprocess it, and report its `trace_log_size`. Used to find the fixed point
/// `L_uniform = multi_trace_log_size + log_blowup_factor` when configuring the
/// recursion so that the multi can verify its own proofs.
fn measure_multi_trace_log_size_at_inner_lifting(inner_lifting_log_size: u32) -> u32 {
    // Build a NoValue Fibonacci context at the same inner lifting so its config
    // (preprocessed_column_ids, output_addresses, n_blake_gates) matches what
    // a real Fibonacci proof at this lifting would have.
    let mut fib_ctx = build_fibonacci_context_with_5_outputs::<NoValue>();
    finalize_constants(&mut fib_ctx);
    let pp_fib = PreprocessedCircuit::preprocess_circuit(&mut fib_ctx);
    let log_blowup = PcsConfig::default().fri_config.log_blowup_factor;
    let inner_pcs_config = PcsConfig {
        lifting_log_size: Some(inner_lifting_log_size),
        ..PcsConfig::default()
    };
    let fib_config = CircuitConfig {
        config: inner_pcs_config,
        output_addresses: pp_fib.params.output_addresses.clone(),
        n_blake_gates: pp_fib.params.n_blake_gates,
        preprocessed_column_ids: pp_fib.preprocessed_trace.ids(),
        preprocessed_root: HashValue(QM31::zero(), QM31::zero()),
    };

    let make_input = || {
        let proof_config = ProofConfig::from_components(
            &all_circuit_components::<NoValue>(),
            vec![true; all_circuit_components::<NoValue>().len()],
            fib_config.preprocessed_column_ids.len(),
            &inner_pcs_config,
            INTERACTION_POW_BITS,
        );
        Input {
            proof: empty_proof(&proof_config),
            circuit_public_data: CircuitPublicData {
                output_values: vec![QM31::zero(); 4],
            },
            config: CircuitConfig {
                config: fib_config.config,
                output_addresses: fib_config.output_addresses.clone(),
                n_blake_gates: fib_config.n_blake_gates,
                preprocessed_column_ids: fib_config.preprocessed_column_ids.clone(),
                preprocessed_root: fib_config.preprocessed_root,
            },
            is_multiverifier: false,
            other_hash: HashValue(QM31::zero(), QM31::zero()),
        }
    };
    let mut multi_ctx = build_multiverifier_circuit::<NoValue>(
        make_input(),
        make_input(),
        inner_pcs_config,
        HashValue(QM31::zero(), QM31::zero()),
    );
    let pp_multi = PreprocessedCircuit::preprocess_circuit(&mut multi_ctx);
    let _ = log_blowup;
    pp_multi.params.trace_log_size
}

/// Diagnostic: print the `preprocessed_column_ids` (post-`sort_by_size`) for
/// the Cairo verifier circuit and the multiverifier circuit, side by side, so
/// we can see whether they agree on order — which is the precondition for a
/// single multiverifier circuit body to verify both Cairo-verifier proofs and
/// multiverifier proofs.
#[test]
fn explore_cairo_vs_multi_preprocessed_column_ids() {
    use circuit_cairo_verifier::privacy::privacy_cairo_verifier_config;
    use circuit_cairo_verifier::verify::build_cairo_verifier_circuit;

    // --- Cairo verifier circuit ---
    let cairo_proof_log_blowup_factor = 3;
    let cairo_verifier_config = privacy_cairo_verifier_config(cairo_proof_log_blowup_factor);
    let mut cairo_ctx = build_cairo_verifier_circuit(&cairo_verifier_config);
    let pp_cairo = PreprocessedCircuit::preprocess_circuit(&mut cairo_ctx);
    let cairo_ids = pp_cairo.preprocessed_trace.ids();

    println!("Cairo verifier:");
    println!("  trace_log_size: {}", pp_cairo.params.trace_log_size);
    println!("  n_blake_gates:  {}", pp_cairo.params.n_blake_gates);
    println!("  n columns:      {}", cairo_ids.len());

    // --- Multiverifier circuit (built using the existing helper that uses fib's config) ---
    // Re-derive a fib config so we can build the NoValue multi.
    let mut fib_ctx = build_fibonacci_context_with_5_outputs::<NoValue>();
    finalize_constants(&mut fib_ctx);
    let pp_fib = PreprocessedCircuit::preprocess_circuit(&mut fib_ctx);
    let inner_pcs_config = PcsConfig {
        lifting_log_size: Some(21),
        ..PcsConfig::default()
    };
    let fib_config = CircuitConfig {
        config: inner_pcs_config,
        output_addresses: pp_fib.params.output_addresses.clone(),
        n_blake_gates: pp_fib.params.n_blake_gates,
        preprocessed_column_ids: pp_fib.preprocessed_trace.ids(),
        preprocessed_root: HashValue(QM31::zero(), QM31::zero()),
    };
    let multi_meta = extract_multi_metadata(&fib_config, inner_pcs_config);
    let multi_ids = &multi_meta.preprocessed_column_ids;

    println!("Multiverifier:");
    println!("  n_blake_gates:  {}", multi_meta.n_blake_gates);
    println!("  n columns:      {}", multi_ids.len());

    // --- Side-by-side diff ---
    let max_len = std::cmp::max(cairo_ids.len(), multi_ids.len());
    println!("\n{:>3}  {:<35}  {:<35}  match", "i", "cairo verifier", "multiverifier");
    println!("{}", "-".repeat(90));
    let mut n_match = 0;
    let mut first_diff = None;
    for i in 0..max_len {
        let c = cairo_ids.get(i).map(|x| x.id.as_str()).unwrap_or("(none)");
        let m = multi_ids.get(i).map(|x| x.id.as_str()).unwrap_or("(none)");
        let same = c == m;
        if same {
            n_match += 1;
        } else if first_diff.is_none() {
            first_diff = Some(i);
        }
        println!("{:>3}  {:<35}  {:<35}  {}", i, c, m, if same { "✓" } else { "✗" });
    }
    println!(
        "\n{n_match}/{max_len} positions match. First mismatch at index {:?}.",
        first_diff,
    );
}

#[test]
fn explore_blake_gate_counts() {
    // Print fib's and multi's blake-gate counts so we know how much to pad.
    let mut fib_ctx = build_fibonacci_context_with_5_outputs::<NoValue>();
    finalize_constants(&mut fib_ctx);
    let pp_fib = PreprocessedCircuit::preprocess_circuit(&mut fib_ctx);
    println!("fib: n_blake_gates = {}, trace_log_size = {}", pp_fib.params.n_blake_gates, pp_fib.params.trace_log_size);

    let inner_pcs_config = PcsConfig {
        lifting_log_size: Some(21),
        ..PcsConfig::default()
    };
    let fib_config = CircuitConfig {
        config: inner_pcs_config,
        output_addresses: pp_fib.params.output_addresses.clone(),
        n_blake_gates: pp_fib.params.n_blake_gates,
        preprocessed_column_ids: pp_fib.preprocessed_trace.ids(),
        preprocessed_root: HashValue(QM31::zero(), QM31::zero()),
    };
    let multi_meta = extract_multi_metadata(&fib_config, inner_pcs_config);
    println!(
        "multi: n_blake_gates = {}, log_n_blake_updates ~ {}",
        multi_meta.n_blake_gates,
        multi_meta.n_blake_gates.next_power_of_two().ilog2(),
    );
}

#[test]
fn explore_multi_lifting_fixed_point() {
    let log_blowup = PcsConfig::default().fri_config.log_blowup_factor;
    println!("log_blowup = {log_blowup}");
    // Inner liftings below 17 hit a known issue inside `verify()`:
    // `component_sizes_bits.get(RELATION_USES_NUM_ROWS_SHIFT=16..)` returns `None`,
    // which falls back to `Simd::one(...)` whose chunks are reused across the two
    // `verify()` invocations and trip the `mark_as_maybe_unused` assertion. Skip
    // those — they're below the relevant range anyway because the preprocessed
    // `bitwise_xor_10` columns force `trace_log_size >= 20`.
    for inner_lift in [21u32, 22, 23, 24, 25] {
        let trace_log = measure_multi_trace_log_size_at_inner_lifting(inner_lift);
        let natural_lift = trace_log + log_blowup;
        let fp = if natural_lift == inner_lift { " <-- fixed point" } else { "" };
        println!(
            "inner_lift = {inner_lift:>2} | multi trace_log_size = {trace_log:>2} | \
             multi natural_lift = {natural_lift:>2}{fp}"
        );
    }
}

// ---------- Tests A & B: prove → save → load → verify ----------

const MULTI_PROOF_PATH: &str = "/tmp/circuit_multiverifier_test_multi_proof.bin";

/// Test A: prove a multiverifier circuit that verifies two Fibonacci proofs,
/// using the *correct* recursion-tree `metadata_root H = hash_node(h_fib, h_multi)`,
/// and write the resulting `Proof<QM31>` bytes to disk so Test B can consume them.
///
/// Note: each Fib leaf sits at index 0 of the H-tree (`bit = 0`) with sibling
/// `h_multi` (so the in-circuit Merkle equation reduces to
/// `hash_node(h_fib, h_multi) == H`).
#[test]
fn test_a_prove_multiverifier_of_fibs_and_save() {
    // 1. Prove two Fibonacci circuits (each at fib's natural lifting = 21).
    let bundle1 = prove_fibonacci_and_prepare();
    let bundle2 = prove_fibonacci_and_prepare();
    let pcs_config = bundle1.pcs_config; // inner pcs_config (lifting=21)

    // 2. h_fib (variant A descriptor hash).
    let h_fib = hash_metadata(leaf_metadata(&bundle1.config));

    // 3. Multi structural metadata + h_multi.
    let multi_meta = extract_multi_metadata(&bundle1.config, pcs_config);
    let h_multi = hash_metadata(multi_metadata(
        &multi_meta.output_addresses,
        multi_meta.n_blake_gates,
        multi_meta.preprocessed_root,
    ));

    // 4. Recursion-tree-wide metadata root: H = hash_node(h_fib, h_multi).
    let metadata_root = hash_node(h_fib, h_multi);

    // 5. Build the multi (QM31) verifying both Fib proofs at the correct H,
    //    each with sibling `h_multi`.
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
    let mut multi_ctx = build_multiverifier_circuit::<QM31>(p1, p2, pcs_config, metadata_root);
    multi_ctx.validate_circuit();

    // 6. Prove the multiverifier circuit itself. `prove_circuit_assignment`
    //    auto-lifts to multi_trace_log_size + log_blowup; that should match
    //    `multi_meta.outer_pcs_config.lifting_log_size`.
    let pp_multi = PreprocessedCircuit::preprocess_circuit(&mut multi_ctx);
    let multi_circuit_proof = prove_circuit_assignment(
        multi_ctx.values(),
        &pp_multi,
        &BaseColumnPool::<SimdBackend>::new(),
        PcsConfig::default(),
    );
    let resolved_outer_pcs_config = multi_circuit_proof.pcs_config;
    assert_eq!(resolved_outer_pcs_config, multi_meta.outer_pcs_config);
    // Sanity: the preprocessed_root we extracted offline should match the
    // commitments[0] the prover just produced.
    assert_eq!(
        multi_meta.preprocessed_root,
        Into::<HashValue<QM31>>::into(
            multi_circuit_proof.stark_proof.as_ref().unwrap().proof.commitments[0],
        ),
    );

    // 7. Convert to circuit-shaped (Proof<QM31>) form and serialize.
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
    std::fs::write(MULTI_PROOF_PATH, &serialized).expect("write multi proof");

    println!(
        "Saved multi proof ({} bytes) to {MULTI_PROOF_PATH}",
        serialized.len()
    );
    println!("multi trace_log_size: {}", pp_multi.params.trace_log_size);
    println!("resolved outer pcs_config: {:?}", resolved_outer_pcs_config);
}

/// Test B: load Test A's multi proof, prove a fresh Fibonacci proof (at the
/// same lifting — fib's natural lifting equals the multi's natural lifting,
/// 21, so no custom lifting is needed), build a *second-level* multi that
/// verifies `[multi_proof, fib_proof]`, and value-validate it.
///
/// Critical recursion property: the second-level multi must be the *same
/// circuit* as the first-level one. That holds here because both use the same
/// inner `pcs_config` (lifting = 21) and the multi's natural lifting is also
/// 21 — a fixed point. See `explore_multi_lifting_fixed_point`.
#[test]
fn test_b_verify_multi_proof_and_fibonacci_proof_with_multiverifier() {
    use circuit_serialize::deserialize::deserialize_proof_with_config;

    // 1. Prove a fresh Fibonacci. We need its bundle for the second-level
    //    multi's fib Input *and* for deriving fib's CircuitConfig (the same
    //    shape Test A used to compute h_multi).
    let fib_bundle = prove_fibonacci_and_prepare();
    let inner_pcs_config = fib_bundle.pcs_config; // lifting=21

    // 2. Re-derive h_fib, h_multi, H deterministically.
    let h_fib = hash_metadata(leaf_metadata(&fib_bundle.config));
    let multi_meta = extract_multi_metadata(&fib_bundle.config, inner_pcs_config);
    let h_multi = hash_metadata(multi_metadata(
        &multi_meta.output_addresses,
        multi_meta.n_blake_gates,
        multi_meta.preprocessed_root,
    ));
    let metadata_root = hash_node(h_fib, h_multi);

    // 3. Reconstruct the multi's `CircuitPublicData`. The multi's outputs are
    //    `[H_lo, H_hi, hash_of_payloads_lo, hash_of_payloads_hi, u]`, where
    //    `hash_of_payloads = blake([fib_a, fib_b, fib_a, fib_b], 64)` over the
    //    two Fibonacci payload pairs (Test A's two fibs are identical, since
    //    Fibonacci is deterministic — so so are this Test's).
    let fib_payload_lo = fib_bundle.public_data.output_values[2];
    let fib_payload_hi = fib_bundle.public_data.output_values[3];
    let hash_of_payloads = blake_qm31(
        &[fib_payload_lo, fib_payload_hi, fib_payload_lo, fib_payload_hi],
        64,
    );
    let u_value = circuits::ivalue::qm31_from_u32s(0, 0, 1, 0);
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
    let bytes = std::fs::read(MULTI_PROOF_PATH)
        .expect("Test A must run first to write the multi proof to disk");
    let mut slice = bytes.as_slice();
    let multi_proof = deserialize_proof_with_config(&mut slice, &multi_proof_config)
        .expect("deserialize multi proof");

    // 7. Build the second-level multi:
    //    - The multi proof sits at leaf index 1 of the H-tree (`bit = 1`),
    //      so its sibling is `h_fib`.
    //    - The fib proof sits at leaf index 0 (`bit = 0`), sibling is `h_multi`.
    //    The same `inner_pcs_config` (lifting=21) verifies both proofs — the
    //    pcs-config-must-be-the-same constraint is satisfied because both the
    //    multi proof and the fib proof are at lifting=21 (multi's natural
    //    lifting, which is also fib's natural lifting).
    let multi_input = Input {
        proof: multi_proof,
        circuit_public_data: multi_public_data,
        config: multi_config,
        is_multiverifier: true,
        other_hash: h_fib,
    };
    let fib_input = Input {
        proof: fib_bundle.proof,
        circuit_public_data: fib_bundle.public_data,
        config: fib_bundle.config,
        is_multiverifier: false,
        other_hash: h_multi,
    };
    // Note: `inner_pcs_config` (lifting=21) is what's "baked" into the
    // second-level multi's verify() calls — same as the first-level multi.
    // This is what makes the two circuits identical (the recursion fixed
    // point). `metadata_root` is the same H as Test A used.
    let context = build_multiverifier_circuit::<QM31>(
        multi_input,
        fib_input,
        inner_pcs_config,
        metadata_root,
    );

    context.check_vars_used();
    context.circuit.check_yields();
    context.validate_circuit();
}

/// End-to-end test: verify two real STARK proofs of the Fibonacci circuit through
/// the multiverifier. Builds and validates with `Value = QM31`, so this exercises
/// both topology (`check_yields` / `check_vars_used`) and value-level constraints
/// (`is_circuit_valid`) — the latter checks the inner `verify()` sub-circuits, the
/// metadata-hash Merkle path against `metadata_root`, the output-value matching
/// (incl. `u` and the leaf padding zeros), and the final hash-of-payloads.
///
/// Both inputs are the same Fibonacci proof (deterministic, identical witnesses).
/// Since the multiverifier currently hardcodes `bit = zero` and `other_hash = (0,0)`,
/// the in-circuit Merkle equation reduces to:
///   metadata_root == hash_node(metadata.hash(), HashValue(0, 0))
/// which we compute out-of-circuit via `blake_qm31` and pass in.
#[test]
fn test_multiverifier_verifies_two_fibonacci_proofs() {
    // Prove twice (deterministic; we need two owned `Proof<QM31>` values — `Proof` is
    // not `Clone`, and `prepare_circuit_proof_for_circuit_verifier` consumes its
    // `CircuitProof`).
    let bundle1 = prove_fibonacci_and_prepare();
    let bundle2 = prove_fibonacci_and_prepare();
    assert_eq!(bundle1.config.preprocessed_root, bundle2.config.preprocessed_root);
    assert_eq!(bundle1.config.output_addresses, bundle2.config.output_addresses);

    // Build the metadata hash out-of-circuit, mirroring what `Metadata::hash` does
    // in-circuit (`blake` over `serialize_to_qm31`, byte-length = 16 * n_qm31s).
    // `bit = 0` because Fibonacci is a leaf (variant A).
    let metadata: Metadata<QM31> = Metadata {
        bit: M31Wrapper::new_unsafe(QM31::zero()),
        n_blake_gates_pow_two: M31Wrapper::new_unsafe(QM31::from(
            bundle1.config.n_blake_gates.next_power_of_two(),
        )),
        output_addresses: bundle1
            .config
            .output_addresses
            .iter()
            .map(|x| M31Wrapper::new_unsafe(QM31::from(*x)))
            .collect(),
        preprocessed_root: bundle1.config.preprocessed_root,
    };
    let metadata_qm31s = metadata.serialize_to_qm31();
    let metadata_hash = blake_qm31(&metadata_qm31s, 16 * metadata_qm31s.len());

    // The multiverifier's verify_merkle_path with `bit = zero` and
    // `other_hash = HashValue(0, 0)` expects:
    //   root == hash_node(metadata_hash, HashValue(0, 0))
    //         == blake([metadata_hash.0, metadata_hash.1, 0, 0], 64).
    let metadata_root = blake_qm31(
        &[metadata_hash.0, metadata_hash.1, QM31::zero(), QM31::zero()],
        64,
    );

    let pcs_config = bundle1.pcs_config;
    let zero_sibling = HashValue(QM31::zero(), QM31::zero());
    let p1 = Input {
        proof: bundle1.proof,
        circuit_public_data: bundle1.public_data,
        config: bundle1.config,
        is_multiverifier: false,
        other_hash: zero_sibling,
    };
    let p2 = Input {
        proof: bundle2.proof,
        circuit_public_data: bundle2.public_data,
        config: bundle2.config,
        is_multiverifier: false,
        other_hash: zero_sibling,
    };

    let context = build_multiverifier_circuit::<QM31>(p1, p2, pcs_config, metadata_root);

    context.check_vars_used();
    context.circuit.check_yields();
    context.validate_circuit();
}
