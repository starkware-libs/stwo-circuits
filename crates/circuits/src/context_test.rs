use itertools::Itertools;

use crate::context::{Context, TraceContext};
use crate::finalize_constants::finalize_constants;
use crate::ivalue::qm31_from_u32s;

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

#[test]
fn test_set_outputs() {
    let mut context = TraceContext::default();

    let a = context.constant(qm31_from_u32s(3, 0, 0, 0));

    let reserved0 = context.reserve();
    let reserved1 = context.reserve();
    context.set_outputs(&[a, a]);
    assert_eq!(context.get(reserved0), qm31_from_u32s(3, 0, 0, 0));
    assert_eq!(context.get(reserved1), qm31_from_u32s(3, 0, 0, 0));

    for (actual, expected) in context.circuit.output.iter().zip_eq([reserved0.idx, reserved1.idx]) {
        assert_eq!(actual.in0, expected);
    }
    finalize_constants(&mut context);
    context.finalize_guessed_vars();
    context.circuit.check_yields();

    context.validate_circuit();
}

#[test]
#[should_panic(expected = "were never assigned")]
fn test_unfulfilled_reservation_panics_at_finalize() {
    let mut context = TraceContext::default();
    context.reserve();
    context.finalize_guessed_vars();
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "read of reserved variable")]
fn test_read_before_fulfill_panics_in_debug() {
    let mut context = TraceContext::default();
    let r = context.reserve();
    context.get(r);
}

#[test]
fn test_zero_and_one() {
    let mut context = TraceContext::default();

    let zero = context.constant(0.into());
    let one = context.constant(1.into());

    assert_eq!(context.zero().idx, zero.idx);
    assert_eq!(context.one().idx, one.idx);
}
