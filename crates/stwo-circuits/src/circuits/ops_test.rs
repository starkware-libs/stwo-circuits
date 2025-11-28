use expect_test::expect;

use crate::circuits::context::TraceContext;
use crate::circuits::ivalue::qm31_from_u32s;
use crate::circuits::ops::{div, eq, from_partial_evals, guess, pointwise_mul};
use crate::circuits::stats::Stats;
use crate::eval;

#[test]
fn test_basic_ops() {
    let x = qm31_from_u32s(1, 2, 3, 4);
    let y = qm31_from_u32s(0, 5, 8, 20);

    let mut context = TraceContext::default();
    let a = context.new_var(x);
    let b = context.new_var(y);
    let c = eval!(&mut context, ((a) + (b)) * ((a) - (b)));
    assert_eq!(context.get(c), (x + y) * (x - y));

    assert_eq!(context.values(), &vec![0.into(), 1.into(), x, y, x + y, x - y, (x + y) * (x - y)]);

    context.validate_circuit();
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

#[test]
fn test_eval_macro() {
    let mut context = TraceContext::default();
    let a = context.new_var(10.into());
    let res = eval!(&mut context, (((a) * (20)) - ((2) * (3))) - (10));
    assert_eq!(context.get(res), 184.into());
}

#[test]
fn test_div() {
    let mut context = TraceContext::default();
    let x = guess(&mut context, 10.into());
    let y = guess(&mut context, 2.into());
    let res = div(&mut context, x, y);
    assert_eq!(context.get(res), 5.into());

    expect![[r#"
        [0] = [0] + [0]
        [1] = [1] + [0]
        [2] = [2] + [0]
        [3] = [3] + [0]
        [4] = [4] + [0]
        [5] = [4] * [3]
        [5] = [2]

    "#]]
    .assert_debug_eq(&context.circuit);

    context.validate_circuit();
    assert_eq!(context.circuit.compute_multiplicities(context.n_vars()).0, vec![6, 1, 2, 2, 2, 1]);
    context.circuit.check_yields(context.n_vars());
}

#[test]
fn test_pointwise_mul() {
    let mut context = TraceContext::default();
    let x = guess(&mut context, qm31_from_u32s(1, 2, 3, 4));
    let y = guess(&mut context, qm31_from_u32s(5, 6, 7, 8));
    let res = pointwise_mul(&mut context, x, y);
    assert_eq!(context.get(res), qm31_from_u32s(5, 12, 21, 32));
    context.validate_circuit();
}

#[test]
fn test_from_partial_evals() {
    let mut context = TraceContext::default();
    let values = [
        guess(&mut context, qm31_from_u32s(1, 10, 100, 1000)),
        guess(&mut context, 2.into()),
        guess(&mut context, 3.into()),
        guess(&mut context, 4.into()),
    ];
    let res = from_partial_evals(&mut context, values);
    assert_eq!(context.get(res), qm31_from_u32s(1, 12, 103, 1004));
    context.validate_circuit();
}

#[test]
fn test_stats() {
    let mut context = TraceContext::default();

    // 2 guesses are from the zero and one constants.
    let stats = Stats { guess: 2, ..Stats::default() };
    assert_eq!(context.stats, stats);

    let x = guess(&mut context, 5.into());
    let y = context.constant(25.into());

    let x_sqr = eval!(&mut context, (x) * (x));
    let stats = Stats { mul: 1, guess: 4, ..stats };
    assert_eq!(context.stats, stats);

    let x_sqr_minus_y = eval!(&mut context, (x_sqr) - (y));
    let stats = Stats { sub: 1, ..stats };
    assert_eq!(context.stats, stats);

    let zero = context.zero();
    eq(&mut context, x_sqr_minus_y, zero);
    let stats = Stats { equals: 1, ..stats };
    assert_eq!(context.stats, stats);

    eval!(&mut context, (0) + (0));
    let stats = Stats { add: 1, ..stats };
    assert_eq!(context.stats, stats);

    div(&mut context, x, y);
    let stats = Stats {
        div: 1,
        mul: stats.mul + 1,
        guess: stats.guess + 1,
        equals: stats.equals + 1,
        ..stats
    };
    assert_eq!(context.stats, stats);

    pointwise_mul(&mut context, x, y);
    let stats = Stats { pointwise_mul: 1, ..stats };
    assert_eq!(context.stats, stats);

    context.validate_circuit();
}
