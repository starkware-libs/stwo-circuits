//! Compares the multiverifier's `preprocessed_column_ids` against the Cairo
//! verifier circuit's, after padding the multiverifier up into Cairo's column
//! brackets via [`pad_components_to_target_counts`]. The goal is to confirm
//! that — once padded — both circuits sort to the *same* column order, which
//! is the precondition for a single multiverifier circuit body to verify both
//! Cairo-verifier proofs and multiverifier proofs.

use circuit_cairo_verifier::privacy::privacy_cairo_verifier_config;
use circuit_cairo_verifier::verify::build_cairo_verifier_circuit;
use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_verifier::statement::{INTERACTION_POW_BITS, all_circuit_components};
use circuit_verifier::verify::{CircuitConfig, CircuitPublicData};
use circuits::blake::HashValue;
use circuits::context::Context;
use circuits::finalize_constants::finalize_constants;
use circuits::ivalue::NoValue;
use circuits_stark_verifier::proof::{ProofConfig, empty_proof};
use num_traits::Zero;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;

use super::verify_test::build_fibonacci_context_with_5_outputs;
use super::{Input, SubCircuitConfig, build_multiverifier_circuit};
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
