//! Helpers to inflate the size of a [`Context`]'s circuit components so they
//! land in a specified `next_power_of_two` bracket. Used to align the
//! multiverifier's column sizes with another circuit's (e.g. the Cairo
//! verifier's), so that after `PreProcessedTrace::sort_by_size` both end up
//! with the same `preprocessed_column_ids` order — the precondition for one
//! multiverifier circuit body to verify both.
//!
//! Padding is one-way: every target must be `>=` the current count, otherwise
//! the function asserts. The helper expects to be called *after* the regular
//! circuit construction (i.e. after `build_multiverifier_circuit` has finished
//! adding its own gates) and *before* `PreprocessedCircuit::preprocess_circuit`.

use circuits::blake::blake;
use circuits::circuit::Circuit;
use circuits::context::Context;
use circuits::eval;
use circuits::ivalue::IValue;
use circuits::ops::eq;

/// Pads `context`'s components so their natural (pre-`pad_*`) counts reach
/// the specified targets:
///   - `eq`: number of `Eq` gates.
///   - `qm31_ops`: total `add + sub + mul + pointwise_mul + 2 * permutation` rows (the "qm31 ops"
///     component's natural row count).
///   - `n_blake_gates`: number of `Blake` gates (drives the `blake_output` component's column
///     size).
///   - `n_blake_compress_rows`: total compression rows across all blake gates (drives the
///     `blake_compress` columns and `log_n_blake_updates`, which decides the `seq_<log>` column
///     id).
///
/// Blake padding is filled with a mix of 1-, 2-, 3-, and 4-chunk dummies so
/// that *both* the gate count and the compression-row count hit their
/// respective targets. This requires `n_blake_gates <= n_blake_compress_rows
/// <= 4 * n_blake_gates` (post-pad); otherwise the function asserts.
pub fn pad_components_to_target_counts<Value: IValue>(
    context: &mut Context<Value>,
    target_eq: usize,
    target_qm31_ops: usize,
    target_n_blake_gates: usize,
    target_n_blake_compress_rows: usize,
) {
    let zero = context.zero();

    // --- eq ---
    let current_eq = context.circuit.eq.len();
    assert!(target_eq >= current_eq, "target_eq ({target_eq}) below current count ({current_eq})",);
    for _ in 0..(target_eq - current_eq) {
        eq(context, zero, zero);
    }

    // --- qm31_ops ---
    let current_qm31_ops = qm31_ops_n_rows(&context.circuit);
    assert!(
        target_qm31_ops >= current_qm31_ops,
        "target_qm31_ops ({target_qm31_ops}) below current count ({current_qm31_ops})",
    );
    for _ in 0..(target_qm31_ops - current_qm31_ops) {
        // `Add(zero, zero, new_var)` — single qm31_ops row, fresh out var.
        let _ = eval!(context, (zero) + (zero));
    }

    // --- blake (1-chunk + 2-chunk mix) ---
    let current_blakes = context.circuit.blake.len();
    let current_blake_compress: usize = context.circuit.blake.iter().map(|g| g.input.len()).sum();
    assert!(
        target_n_blake_gates >= current_blakes,
        "target_n_blake_gates ({target_n_blake_gates}) below current ({current_blakes})",
    );
    assert!(
        target_n_blake_compress_rows >= current_blake_compress,
        "target_n_blake_compress_rows ({target_n_blake_compress_rows}) below current ({current_blake_compress})",
    );
    let need_gates = target_n_blake_gates - current_blakes;
    let need_compress = target_n_blake_compress_rows - current_blake_compress;
    assert!(
        need_compress >= need_gates,
        "blake compress target requires fewer rows than gates would produce ({need_compress} < {need_gates}); each blake gate has at least one chunk",
    );
    // Solve for non-negative `(x_1, x_2, x_3, x_4)` with
    //   x_1 + x_2 + x_3 + x_4 = need_gates,
    //   1*x_1 + 2*x_2 + 3*x_3 + 4*x_4 = need_compress.
    // Equivalently (rows beyond 1 per gate):
    //   x_2 + 2*x_3 + 3*x_4 = need_compress - need_gates.
    // Greedy: maximise 4-chunk gates. This works whenever
    //   need_compress <= 4 * need_gates.
    let extra_rows = need_compress - need_gates;
    let four_chunk = extra_rows / 3;
    let remainder = extra_rows % 3;
    // Distribute the remainder as 1 two-chunk (extra=1) or 1 three-chunk
    // (extra=2). Either gives a small fixup.
    let (two_chunk, three_chunk) = match remainder {
        0 => (0, 0),
        1 => (1, 0),
        2 => (0, 1),
        _ => unreachable!(),
    };
    assert!(
        need_gates >= four_chunk + three_chunk + two_chunk,
        "blake gates target ({need_gates}) too low to reach compress target ({need_compress}); would need 5+ chunks per gate",
    );
    let one_chunk = need_gates - four_chunk - three_chunk - two_chunk;

    for _ in 0..one_chunk {
        // 1 input QM31 → 1 chunk → 1 compression row.
        blake(context, &[zero], 1);
    }
    let eight_zeros = vec![zero; 8];
    for _ in 0..two_chunk {
        // 8 input QM31s → 2 chunks → 2 compression rows. n_bytes = 8 * 16.
        blake(context, &eight_zeros, 128);
    }
    let twelve_zeros = vec![zero; 12];
    for _ in 0..three_chunk {
        // 12 input QM31s → 3 chunks → 3 compression rows. n_bytes = 12 * 16.
        blake(context, &twelve_zeros, 192);
    }
    let sixteen_zeros = vec![zero; 16];
    for _ in 0..four_chunk {
        // 16 input QM31s → 4 chunks → 4 compression rows. n_bytes = 16 * 16.
        blake(context, &sixteen_zeros, 256);
    }
}

/// Mirrors the `qm31_ops_n_rows` formula used by `pad_qm31_ops` in
/// `circuit_common::finalize`.
pub fn qm31_ops_n_rows(circuit: &Circuit) -> usize {
    circuit.add.len()
        + circuit.sub.len()
        + circuit.mul.len()
        + circuit.pointwise_mul.len()
        + circuit.permutation.iter().map(|p| p.inputs.len() + p.outputs.len()).sum::<usize>()
}
