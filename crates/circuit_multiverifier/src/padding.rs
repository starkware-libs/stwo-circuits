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
use circuits::ivalue::IValue;
use circuits::ops::{add, eq};

pub fn pad_components_to_target_counts<Value: IValue>(
    context: &mut Context<Value>,
    target_eq: usize,
    target_qm31_ops: usize,
    target_n_blake_gates: usize,
    target_n_blake_compress_rows: usize,
) {
    let zero = context.zero();

    let current_eq = context.circuit.eq.len();
    assert!(target_eq >= current_eq);
    for _ in 0..(target_eq - current_eq) {
        eq(context, zero, zero);
    }

    let current_qm31_ops = qm31_ops_n_rows(&context.circuit);
    assert!(target_qm31_ops >= current_qm31_ops);
    for _ in 0..(target_qm31_ops - current_qm31_ops) {
        add(context, zero, zero);
    }
    // TODO: make padding correct + be sure that a subsequent call to finalize context won't change
    // anything in the counts.

    // The purpose is to add a certain number of blake gates so that after the addition we have
    // n_blake_gates \in `(target_n_blake_gates / 2, target_n_blake_gates]`
    // n_blake_updates \in `(target_n_blake_updates / 2, target_n_blake_updates]`
    // Each addition of a blake gate increases n_blake_updates by at least 1.
    let current_blake_gates = context.circuit.blake.len();
    let mut current_blake_updates = context.stats.blake_updates;
    assert!(current_blake_gates <= target_n_blake_gates);
    assert!(current_blake_updates <= target_n_blake_compress_rows);

    let lower_bound_blake_gates = (target_n_blake_gates / 2) + 1;
    let lower_bound_blake_updates = (target_n_blake_compress_rows / 2) + 1;

    if lower_bound_blake_gates <= current_blake_gates
        && lower_bound_blake_updates <= current_blake_updates
    {
        return;
    }
    if (current_blake_gates == target_n_blake_gates)
        || (current_blake_updates == target_n_blake_gates)
    {
        panic!("Unable to pad blake.")
    }
    let need_gates = ((target_n_blake_gates / 2) + 1).saturating_sub(current_blake_gates);
    if need_gates > 0 {
        for _ in 0..need_gates - 1 {
            blake(context, &[zero], 1);
            current_blake_updates += 1;
        }
    }

    if current_blake_updates > target_n_blake_compress_rows {
        panic!("Unable to pad blake.")
    }
    let need_compress =
        ((target_n_blake_compress_rows / 2) + 1).saturating_sub(current_blake_updates);
    if need_compress > 0 {
        blake(context, &vec![zero; need_compress * 4], need_compress * 64);
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
