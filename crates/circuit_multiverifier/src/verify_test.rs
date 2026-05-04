use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_verifier::{
    statement::{INTERACTION_POW_BITS, all_circuit_components},
    verify::{CircuitConfig, CircuitPublicData},
};
use circuits::{
    blake::{HashValue, blake},
    context::Context,
    eval,
    finalize_constants::finalize_constants,
    ivalue::{IValue, NoValue},
    ops::{guess, output},
};
use circuits_stark_verifier::proof::{ProofConfig, empty_proof};
use num_traits::{One, Zero};
use stwo::core::{fields::qm31::QM31, pcs::PcsConfig};

use super::{Input, build_multiverifier_circuit};

/// Builds the same Fibonacci-shaped [`CircuitConfig`] used by the real test, but
/// without running the prover. Driving topology checks through the *real* trace
/// shape avoids the boundary-condition fragility of hand-crafted `pcs_config`s
/// (e.g. landing exactly on `log_trace_size == RELATION_USES_NUM_ROWS_SHIFT`,
/// which makes `verify()` call `combine_bits` on an empty slice).
fn synthetic_circuit_config() -> CircuitConfig {
    let mut context = build_fibonacci_context_with_5_outputs::<NoValue>();
    finalize_constants(&mut context);
    let preprocessed_circuit = PreprocessedCircuit::preprocess_circuit(&mut context);

    // Use the same lifting choice as `prove_circuit_assignment` would pick.
    let mut pcs_config = PcsConfig::default();
    let lifting_log_size = preprocessed_circuit.params.trace_log_size as u32 + pcs_config.fri_config.log_blowup_factor;
    pcs_config.lifting_log_size = Some(lifting_log_size);

    CircuitConfig {
        config: pcs_config,
        output_addresses: preprocessed_circuit.params.output_addresses.clone(),
        n_blake_gates: preprocessed_circuit.params.n_blake_gates,
        preprocessed_column_ids: preprocessed_circuit.preprocessed_trace.ids(),
        // Dummy root.
        preprocessed_root: HashValue(QM31::zero(), QM31::zero()),
    }
}

fn build_novalue_input() -> Input<NoValue> {
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
    Input {
        proof,
        circuit_public_data,
        config,
        is_multiverifier: false,
        other_hash: HashValue(QM31::zero(), QM31::zero()),
    }
}

/// Topology smoke test: build the multiverifier circuit with `NoValue` and verify
/// the structural invariants.
#[test]
fn test_novalue_multiverifier_circuit() {
    let p1 = build_novalue_input();
    let p2 = build_novalue_input();

    let pcs_config = synthetic_circuit_config().config;
    // Dummy metadata root.
    let metadata_root = HashValue(QM31::zero(), QM31::zero());

    let context = build_multiverifier_circuit::<NoValue>(p1, p2, pcs_config, metadata_root);

    context.check_vars_used();
    context.circuit.check_yields();
}

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
pub(super) fn build_fibonacci_context_with_5_outputs<Value: IValue>() -> Context<Value> {
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

    // For the multiverifier circuit to be the *same* circuit when verifying a
    // fib proof and a multi proof, both proofs must have been committed with
    // the same `preprocessed_column_ids` order. After
    // `PreProcessedTrace::sort_by_size`, that order is determined by each
    // component column's `next_power_of_two` size. So fib must land in the
    // same size bracket as multi for every component:
    //   - eq            : multi has ~11k gates → 2^14 = 16384.
    //   - qm31_ops      : multi has ~120k rows → 2^17 = 131072.
    //   - blake_compress: multi pads to 2^12 = 4096.
    //   - m31_to_u32    : both have 0 → minimum 2^4.
    //   - blake_sigma   : fixed 2^4.
    //   - bitwise_xor_* : fixed by the bit width.
    //   - seq_*         : derived from log_n_blake_updates (= 12 here).
    //
    // We add enough fib-side gates to push it into multi's size bracket. The
    // exact counts don't matter — what matters is the `next_power_of_two`
    // bracket. Numbers below are conservative middles of each bracket.
    let zero = context.zero();
    const FIB_BLAKE_PAD: usize = 4096;
    for _ in 0..FIB_BLAKE_PAD {
        blake(&mut context, &[zero], 1);
    }
    const FIB_EQ_PAD: usize = 14_000; // (8193, 16384] → 2^14
    for _ in 0..FIB_EQ_PAD {
        circuits::ops::eq(&mut context, zero, zero);
    }
    const FIB_QM31_OPS_PAD: usize = 120_000; // (65537, 131072] → 2^17
    for _ in 0..FIB_QM31_OPS_PAD {
        let _ = eval!(&mut context, (zero) + (zero));
    }

    context
}
