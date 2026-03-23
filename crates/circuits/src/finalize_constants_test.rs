use crate::context::TraceContext;
use crate::ivalue::qm31_from_u32s;

use super::*;

#[test]
fn test_no_constants_beyond_defaults() {
    let mut context = TraceContext::default();
    finalize_constants(&mut context);
    context.finalize_guessed_vars();
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
    context.finalize_guessed_vars();
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_large_m31_decomposition() {
    let mut context = TraceContext::default();
    for i in 0u32..100 {
        context.constant(i.into());
    }
    // Request constants that need decomposition (base = chain length = 99).
    context.constant(5000u32.into());
    context.constant(100000u32.into());
    finalize_constants(&mut context);
    context.finalize_guessed_vars();
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_qm31_constants() {
    let mut context = TraceContext::default();
    for i in 0u32..20 {
        context.constant(i.into());
    }
    context.constant(qm31_from_u32s(3, 7, 11, 5));
    finalize_constants(&mut context);
    context.finalize_guessed_vars();
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
    context.finalize_guessed_vars();
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
    context.finalize_guessed_vars();
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
    context.finalize_guessed_vars();
    context.circuit.check_yields();
    context.validate_circuit();
}

#[test]
fn test_intermediate_shadows_constant() {
    // Regression: decomposing 30000 with base=100 creates intermediate a*base=300.
    // If constant 300 is also requested, it must get its yield from that intermediate gate,
    // not be skipped.
    let mut context = TraceContext::default();
    for i in 0u32..101 {
        context.constant(i.into());
    }
    context.constant(300u32.into());
    context.constant(30000u32.into());
    finalize_constants(&mut context);
    context.finalize_guessed_vars();
    context.circuit.check_yields();
    context.validate_circuit();
}
