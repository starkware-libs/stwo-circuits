use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_prover::prover::{
    BaseColumnPool, SimdBackend, prepare_circuit_proof_for_circuit_verifier,
    prove_circuit_assignment,
};
use circuit_verifier::{
    statement::{INTERACTION_POW_BITS, all_circuit_components},
    verify::{CircuitConfig, CircuitPublicData},
};
use circuits::{
    blake::{HashValue, blake_qm31},
    context::Context,
    eval,
    finalize_constants::finalize_constants,
    ivalue::{IValue, NoValue},
    ops::{guess, output},
    wrappers::M31Wrapper,
};
use circuits_stark_verifier::proof::{ProofConfig, empty_proof};
use num_traits::{One, Zero};
use stwo::core::{fields::qm31::QM31, pcs::PcsConfig};

use super::{Input, Metadata, build_multiverifier_circuit};

/// Builds the same Fibonacci-shaped [`CircuitConfig`] used by the real test, but
/// without running the prover. Driving topology checks through the *real* trace
/// shape avoids the boundary-condition fragility of hand-crafted `pcs_config`s
/// (e.g. landing exactly on `log_trace_size == RELATION_USES_NUM_ROWS_SHIFT`,
/// which makes `verify()` call `combine_bits` on an empty slice).
fn synthetic_circuit_config() -> CircuitConfig {
    let mut context = build_fibonacci_context_with_5_outputs::<NoValue>();
    finalize_constants(&mut context);
    let pp = PreprocessedCircuit::preprocess_circuit(&mut context);

    // Use the same lifting choice as `prove_circuit_assignment` would pick.
    let log_blowup = PcsConfig::default().fri_config.log_blowup_factor as u32;
    let lifting_log_size = pp.params.trace_log_size as u32 + log_blowup;
    let pcs_config =
        PcsConfig { lifting_log_size: Some(lifting_log_size), ..PcsConfig::default() };

    CircuitConfig {
        config: pcs_config,
        output_addresses: pp.params.output_addresses.clone(),
        n_blake_gates: pp.params.n_blake_gates,
        preprocessed_column_ids: pp.preprocessed_trace.ids(),
        // Topology test only — value isn't checked under `NoValue`.
        preprocessed_root: HashValue(QM31::zero(), QM31::zero()),
    }
}

fn make_input() -> Input<NoValue> {
    let config = synthetic_circuit_config();
    let components = all_circuit_components::<NoValue>();
    let proof_config = ProofConfig::from_components(
        &components,
        vec![true; components.len()],
        config.preprocessed_column_ids.len(),
        &config.config,
        INTERACTION_POW_BITS,
    );
    let proof = empty_proof(&proof_config);
    let circuit_public_data = CircuitPublicData {
        // Multiverifier reads slots [2] and [3]; provide four dummies.
        output_values: vec![QM31::zero(); 4],
    };
    Input { proof, circuit_public_data, config }
}

/// Topology smoke test: build the multiverifier circuit with `NoValue` and verify
/// the structural invariants.
#[test]
fn test_multiverifier_circuit_builds() {
    let p1 = make_input();
    let p2 = make_input();

    let pcs_config = synthetic_circuit_config().config;
    // For NoValue we just need *some* metadata_root; values aren't checked.
    let metadata_root = HashValue(QM31::zero(), QM31::zero());

    let context = build_multiverifier_circuit::<NoValue>(p1, p2, pcs_config, metadata_root);

    context.check_vars_used();
    context.circuit.check_yields();
    println!("{:?}", context.stats);
    println!("{:?}", context.circuit.output);
}

// ---------- Fibonacci end-to-end test ----------

/// Number of Fibonacci iterations. Mirrors the constant in `circuit_prover::prover_test`.
/// Not a power of two so component padding is exercised.
const FIB_N: usize = 1030;

/// Replica of `build_fibonacci_context` from `circuit_prover::prover_test`,
/// shaped so that after `finalize_constants` the outputs match the multiverifier's
/// `[H_lo, H_hi, payload_lo, payload_hi, u]` 5-slot convention:
///
/// 1. `output(dummy_h0 = 0)` — H slot, padding zero (since this leaf isn't a
///    multiverifier and has no `H` to forward).
/// 2. `output(dummy_h1 = 0)` — H slot, padding zero.
/// 3. `output(fib_a)`        — payload slot.
/// 4. `output(fib_b)`        — payload slot.
/// 5. `output(u)`            — appended by `finalize_constants` (last slot).
///
/// The dummies use `guess(0)` rather than `new_var(0)` so `finalize_guessed_vars`
/// emits a trivial `Add { var, 0, var }` yield — without that the `Output` gates
/// (which `use` their input) would leave the logup unbalanced.
fn build_fibonacci_context_with_5_outputs<Value: IValue>() -> Context<Value> {
    let mut context = Context::<Value>::default();

    let dummy_h0 = guess(&mut context, Value::from_qm31(QM31::zero()));
    let dummy_h1 = guess(&mut context, Value::from_qm31(QM31::zero()));
    output(&mut context, dummy_h0);
    output(&mut context, dummy_h1);

    let (mut a, mut b) = (
        guess(&mut context, Value::from_qm31(QM31::zero())),
        guess(&mut context, Value::from_qm31(QM31::one())),
    );
    for _ in 2..FIB_N {
        (a, b) = (b, eval!(&mut context, (a) + (b)));
    }
    output(&mut context, a);
    output(&mut context, b);

    context
}

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
    let metadata: Metadata<QM31> = Metadata {
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
    let p1 = Input { proof: bundle1.proof, circuit_public_data: bundle1.public_data, config: bundle1.config };
    let p2 = Input { proof: bundle2.proof, circuit_public_data: bundle2.public_data, config: bundle2.config };

    let context = build_multiverifier_circuit::<QM31>(p1, p2, pcs_config, metadata_root);

    context.check_vars_used();
    context.circuit.check_yields();
    context.validate_circuit();
}
