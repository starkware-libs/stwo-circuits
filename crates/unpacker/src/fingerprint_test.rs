use circuits::circuit::Circuit;
use circuits::context::Context;
use circuits::ivalue::{IValue, NoValue, qm31_from_u32s};
use circuits::ops::guess;
use rstest::rstest;
use stwo::core::fields::qm31::QM31;

use crate::fingerprint::circuit_fingerprint;

/// `(a + b) * b`, with `a` and `b` guessed: three gates whose topology is fixed and whose
/// values are not.
fn tiny_circuit<Value: IValue>(a: u32, b: u32) -> Circuit {
    let mut ctx = Context::<Value>::default();
    let a = guess(&mut ctx, Value::from_qm31(qm31_from_u32s(a, 0, 0, 0)));
    let b = guess(&mut ctx, Value::from_qm31(qm31_from_u32s(b, 0, 0, 0)));
    let sum = circuits::ops::add(&mut ctx, a, b);
    let _product = circuits::ops::mul(&mut ctx, sum, b);
    ctx.finalize(false).context.circuit
}

/// Rebuilding the same shape must fingerprint identically.
#[test]
fn fingerprint_is_stable_across_builds() {
    assert_eq!(
        circuit_fingerprint(&tiny_circuit::<QM31>(3, 5)),
        circuit_fingerprint(&tiny_circuit::<QM31>(3, 5))
    );
}

/// The fingerprint covers topology only: different witness values — and a witness-free
/// build — must all agree.
#[rstest]
#[case(3, 5)]
#[case(7, 11)]
#[case(0, 1)]
fn fingerprint_ignores_witness_values(#[case] a: u32, #[case] b: u32) {
    let reference = circuit_fingerprint(&tiny_circuit::<NoValue>(0, 0));
    assert_eq!(circuit_fingerprint(&tiny_circuit::<QM31>(a, b)), reference);
}

/// Wiring is part of the topology: swapping which variable an operand reads must change the
/// fingerprint, or the pin would not detect a rewired circuit.
#[test]
fn fingerprint_detects_rewiring() {
    let mut ctx = Context::<QM31>::default();
    let a = guess(&mut ctx, qm31_from_u32s(3, 0, 0, 0));
    let b = guess(&mut ctx, qm31_from_u32s(5, 0, 0, 0));
    let sum = circuits::ops::add(&mut ctx, a, b);
    // `(a + b) * a` instead of `(a + b) * b`.
    let _product = circuits::ops::mul(&mut ctx, sum, a);
    let rewired = ctx.finalize(false).context.circuit;

    assert_ne!(circuit_fingerprint(&rewired), circuit_fingerprint(&tiny_circuit::<QM31>(3, 5)));
}
