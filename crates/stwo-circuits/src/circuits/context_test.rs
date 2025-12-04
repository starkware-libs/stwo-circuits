use crate::circuits::context::{Context, TraceContext};
use crate::circuits::ivalue::qm31_from_u32s;

#[test]
fn test_constants() {
    let x = qm31_from_u32s(1, 2, 3, 4);
    let mut context = Context::default();
    let a = context.constant(x);
    let _b = context.constant(x + x);
    // The second time `x` is requested, the same variable is returned.
    let c = context.constant(x);
    assert_eq!(a.idx, c.idx);

    assert_eq!(context.values(), &vec![0.into(), 1.into(), x, x + x]);

    context.finalize_guessed_vars();
    assert_eq!(
        format!("{:?}", context.circuit),
        "[0] = [0] + [0]\n[1] = [1] + [0]\n[2] = [2] + [0]\n[3] = [3] + [0]\n"
    );

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
