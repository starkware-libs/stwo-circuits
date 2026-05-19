use crate::circuit::Add;
use crate::context::{Context, TraceContext};
use crate::finalize_constants::finalize_constants;
use crate::ivalue::qm31_from_u32s;
use crate::ops::add;

#[test]
fn test_constants() {
    let x = qm31_from_u32s(1, 2, 3, 4);
    let mut context = Context::default();
    let a = context.constant(x);
    let _b = context.constant(x + x);
    // The second time `x` is requested, the same variable is returned.
    let c = context.constant(x);
    assert_eq!(a.idx, c.idx);

    let u = qm31_from_u32s(0, 0, 1, 0);
    assert_eq!(context.values(), &vec![0.into(), 1.into(), u, x, x + x]);

    finalize_constants(&mut context);
    context.finalize_guessed_vars();
    context.validate_circuit();
}

/// Reserves a slot, references it as the output of a gate built afterwards, then fulfills it
/// with the value implied by that gate. Verifies the circuit holds and the slot is yielded.
#[test]
fn test_reserve_fulfill_round_trip() {
    let mut context = TraceContext::default();

    let a = context.constant(qm31_from_u32s(3, 0, 0, 0));
    let b = context.constant(qm31_from_u32s(5, 0, 0, 0));

    let reserved = context.reserve();
    // Use the reserved Var as the output of an Add gate before the value is known.
    context.circuit.add.push(Add { in0: a.idx, in1: b.idx, out: reserved.idx });
    // ...later, fulfill with the value the gate forces.
    context.fulfill(reserved, qm31_from_u32s(8, 0, 0, 0));

    // The reserved idx is unchanged after fulfillment, and now readable.
    assert_eq!(context.get(reserved), qm31_from_u32s(8, 0, 0, 0));

    finalize_constants(&mut context);
    context.finalize_guessed_vars();
    context.validate_circuit();
}

#[test]
#[should_panic(expected = "reserved variable")]
fn test_unfulfilled_reservation_panics_at_finalize() {
    let mut context = TraceContext::default();
    let _r = context.reserve();
    context.finalize_guessed_vars();
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "read of reserved variable")]
fn test_read_before_fulfill_panics_in_debug() {
    let mut context = TraceContext::default();
    let r = context.reserve();
    let _ = context.get(r);
}

/// A reserved variable that is never yielded by any gate is caught by `check_yields`.
#[test]
#[should_panic]
fn test_reserved_without_yield_fails_check_yields() {
    let mut context = TraceContext::default();
    let r = context.reserve();
    // Use the reserved var as an input, but never yield it.
    let zero = context.zero();
    let _ = add(&mut context, r, zero);
    let _ = context.fulfill(r, qm31_from_u32s(0, 0, 0, 0));
    finalize_constants(&mut context);
    context.finalize_guessed_vars();
    context.circuit.check_yields();
}

#[test]
fn test_zero_and_one() {
    let mut context = TraceContext::default();

    let zero = context.constant(0.into());
    let one = context.constant(1.into());

    assert_eq!(context.zero().idx, zero.idx);
    assert_eq!(context.one().idx, one.idx);
}
