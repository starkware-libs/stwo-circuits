use crate::context::TraceContext;
use crate::ivalue::qm31_from_u32s;

use super::*;

#[test]
fn test_no_constants_beyond_defaults() {
    let mut context = TraceContext::default();
    finalize_constants(&mut context);
    // Skip constant idxs in finalize_guessed_vars.
    finalize_non_constant_guessed_vars(&mut context);
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_small_consecutive_m31_constants() {
    let mut context = TraceContext::default();
    for i in 0u32..10 {
        context.constant(i.into());
    }
    finalize_constants(&mut context);
    finalize_non_constant_guessed_vars(&mut context);
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_large_m31_decomposition() {
    let mut context = TraceContext::default();
    // Build a chain of 0..100.
    for i in 0u32..100 {
        context.constant(i.into());
    }
    // Request constants that need decomposition (base will be 64).
    context.constant(5000u32.into());
    context.constant(100000u32.into());
    finalize_constants(&mut context);
    finalize_non_constant_guessed_vars(&mut context);
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_qm31_constants() {
    let mut context = TraceContext::default();
    // Need chain >=2 for QM31 base construction (i = u^2 - 2).
    for i in 0u32..20 {
        context.constant(i.into());
    }
    context.constant(qm31_from_u32s(3, 7, 11, 5));
    finalize_constants(&mut context);
    finalize_non_constant_guessed_vars(&mut context);
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_qm31_basis_elements_as_constants() {
    let mut context = TraceContext::default();
    for i in 0u32..20 {
        context.constant(i.into());
    }
    // Request i and iu as constants (u is already default).
    context.constant(qm31_from_u32s(0, 1, 0, 0)); // i
    context.constant(qm31_from_u32s(0, 0, 0, 1)); // iu
    finalize_constants(&mut context);
    finalize_non_constant_guessed_vars(&mut context);
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_mixed_m31_and_qm31_constants() {
    let mut context = TraceContext::default();
    for i in 0u32..500 {
        context.constant(i.into());
    }
    context.constant(qm31_from_u32s(100, 200, 300, 400));
    context.constant(10000u32.into());
    finalize_constants(&mut context);
    finalize_non_constant_guessed_vars(&mut context);
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_broadcast_constants() {
    let mut context = TraceContext::default();
    for i in 0u32..20 {
        context.constant(i.into());
    }
    // (5, 5, 5, 5) should use broadcast path: 5 * (1,1,1,1)
    context.constant(qm31_from_u32s(5, 5, 5, 5));
    context.constant(qm31_from_u32s(12, 12, 12, 12));
    finalize_constants(&mut context);
    finalize_non_constant_guessed_vars(&mut context);
    context.circuit.check_yields();
    context.validate_circuit();
}

/// Helper: finalize non-constant guessed vars (mimics the updated finalize_guessed_vars).
fn finalize_non_constant_guessed_vars(context: &mut TraceContext) {
    let constant_idxs: std::collections::HashSet<usize> =
        context.constants().values().map(|v| v.idx).collect();
    for idx in context.guessed_vars.take().unwrap().iter() {
        if constant_idxs.contains(idx) {
            continue;
        }
        context.circuit.add.push(crate::circuit::Add {
            in0: *idx,
            in1: context.zero().idx,
            out: *idx,
        });
    }
}
