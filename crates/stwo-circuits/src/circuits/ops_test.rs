use crate::circuits::context::TraceContext;
use crate::circuits::ivalue::qm31_from_u32s;
use crate::circuits::ops::{add, eq, mul, sub};

#[test]
fn test_basic_ops() {
    let x = qm31_from_u32s(1, 2, 3, 4);
    let y = qm31_from_u32s(0, 5, 8, 20);

    let mut context = TraceContext::default();
    let a = context.new_var(x);
    let b = context.new_var(y);
    let c = add(&mut context, a, b);
    let d = sub(&mut context, a, b);
    let e = mul(&mut context, c, d);
    assert_eq!(context.get(e), (x + y) * (x - y));

    assert_eq!(context.values(), &vec![0.into(), 1.into(), x, y, x + y, x - y, (x + y) * (x - y)]);

    context.circuit.check(context.values()).unwrap();
}

#[test]
fn test_eq() {
    let x = qm31_from_u32s(1, 2, 3, 4);

    let mut context = TraceContext::default();
    let a = context.new_var(x);
    let b = context.new_var(x + x);
    // The following equality is wrong, it'll be caught by the circuit checker.
    eq(&mut context, a, b);
    context.circuit.check(context.values()).unwrap_err();
    context.circuit.check(&[0.into(), 1.into(), x, x]).unwrap();
}
