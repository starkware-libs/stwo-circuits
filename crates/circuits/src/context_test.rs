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
fn test_zero_and_one() {
    let mut context = TraceContext::default();

    let zero = context.constant(0.into());
    let one = context.constant(1.into());

    assert_eq!(context.zero().idx, zero.idx);
    assert_eq!(context.one().idx, one.idx);
}
