use stwo::core::circle::CirclePoint;

use crate::circuits::circuit::{Add, Eq, Mul, PointwiseMul, Sub};
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::{IValue, qm31_from_u32s};

#[cfg(test)]
#[path = "ops_test.rs"]
pub mod test;

/// A macro for writing arithmetic expressions on circuit variables.
///
/// Usage: `eval!(context, expression)`
///
/// Note: Parentheses are required for all arithmetic operations, including literals and variables.
///
/// Example:
/// ```plain
/// let result = eval!(context, ((value) * (value)) - (1));
/// ```
#[macro_export]
macro_rules! eval {
    ($ctx:expr, ($($a:tt)+) + ($($b:tt)+)) => {{
        let __tmp0 = $crate::eval!($ctx, $($a)+);
        let __tmp1 = $crate::eval!($ctx, $($b)+);
        $crate::circuits::ops::add($ctx, __tmp0, __tmp1)
    }};

    ($ctx:expr, ($($a:tt)+) - ($($b:tt)+)) => {{
        let __tmp0 = $crate::eval!($ctx, $($a)+);
        let __tmp1 = $crate::eval!($ctx, $($b)+);
        $crate::circuits::ops::sub($ctx, __tmp0, __tmp1)
    }};

    ($ctx:expr, ($($a:tt)+) * ($($b:tt)+)) => {{
        let __tmp0 = $crate::eval!($ctx, $($a)+);
        let __tmp1 = $crate::eval!($ctx, $($b)+);
        $crate::circuits::ops::mul($ctx, __tmp0, __tmp1)
    }};

    ($ctx:expr, $lit:literal) => {
        $ctx.constant($lit.into())
    };

    ($ctx:expr, $id:expr) => {
        $id
    };
}

/// Adds an equality gate to the circuit.
pub fn eq(context: &mut Context<impl IValue>, a: Var, b: Var) {
    context.stats.equals += 1;
    context.circuit.eq.push(Eq { in0: a.idx, in1: b.idx });
}

/// Adds an addition gate to the circuit, and returns the output variable.
pub fn add(context: &mut Context<impl IValue>, a: Var, b: Var) -> Var {
    context.stats.add += 1;
    let out = context.new_var(context.get(a) + context.get(b));
    context.circuit.add.push(Add { in0: a.idx, in1: b.idx, out: out.idx });
    out
}

/// Adds a subtraction gate to the circuit, and returns the output variable.
pub fn sub(context: &mut Context<impl IValue>, a: Var, b: Var) -> Var {
    context.stats.sub += 1;
    let out = context.new_var(context.get(a) - context.get(b));
    context.circuit.sub.push(Sub { in0: a.idx, in1: b.idx, out: out.idx });
    out
}

/// Adds a multiplication gate to the circuit, and returns the output variable.
pub fn mul(context: &mut Context<impl IValue>, a: Var, b: Var) -> Var {
    context.stats.mul += 1;
    let out = context.new_var(context.get(a) * context.get(b));
    context.circuit.mul.push(Mul { in0: a.idx, in1: b.idx, out: out.idx });
    out
}

/// Computes `a / b` by guessing the result and adding a multiplication gate to the circuit to
/// validate its correctness.
///
/// The caller must ensure that `b` is not zero.
pub fn div(context: &mut Context<impl IValue>, a: Var, b: Var) -> Var {
    context.stats.div += 1;
    let out = guess(context, context.get(a) / context.get(b));
    let mul_res = mul(context, out, b);
    eq(context, mul_res, a);
    out
}

pub fn pointwise_mul<Value: IValue>(context: &mut Context<Value>, a: Var, b: Var) -> Var {
    context.stats.pointwise_mul += 1;
    let out = context.new_var(Value::pointwise_mul(context.get(a), context.get(b)));
    context.circuit.pointwise_mul.push(PointwiseMul { in0: a.idx, in1: b.idx, out: out.idx });
    out
}

/// Returns `(a, b)` if `selector` is 0, and `(b, a)` if `selector` is 1.
/// Assumption: `selector` is either 0 or 1.
pub fn cond_flip(context: &mut Context<impl IValue>, selector: Var, a: Var, b: Var) -> (Var, Var) {
    let diff = eval!(context, (selector) * ((b) - (a)));
    let res_a = eval!(context, (a) + (diff));
    let res_b = eval!(context, (b) - (diff));
    (res_a, res_b)
}

/// Computes the conjugate (with respect to `CM31`) of a `QM31` value:
///   `a + b * i + c * u + d * iu -> a + b * i - c * u - d * iu`.
pub fn conj(c: &mut Context<impl IValue>, a: Var) -> Var {
    let coefs = c.constant(qm31_from_u32s(1, 1, 0, 0) - qm31_from_u32s(0, 0, 1, 1));
    pointwise_mul(c, a, coefs)
}

/// Returns a new unconstrained variable with the given value.
pub fn guess<Value: IValue>(context: &mut Context<Value>, value: Value) -> Var {
    context.stats.guess += 1;
    let out = context.new_var(value);
    context.guessed_vars.as_mut().unwrap().push(out.idx);
    out
}

/// Computes the map `(a, b, c, d) -> a + b * i + c * u + d * iu`. Note that the input values are
/// not necessarily in the base field `M31`.
pub fn from_partial_evals(context: &mut Context<impl IValue>, values: [Var; 4]) -> Var {
    let i = context.constant(qm31_from_u32s(0, 1, 0, 0));
    let u = context.constant(qm31_from_u32s(0, 0, 1, 0));
    let iu = context.constant(qm31_from_u32s(0, 0, 0, 1));

    eval!(
        context,
        (((values[0]) + ((values[1]) * (i))) + ((values[2]) * (u))) + ((values[3]) * (iu))
    )
}

/// A trait for creating [Var]s from values in a recursive structure.
///
/// For example, given a `Vec<Vec<QM31>>` we can create a `Vec<Vec<Var>>`.
pub trait Guess<Value: IValue> {
    type Target;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target;
}

/// Implementation of [Guess] for a single value.
impl<Value: IValue> Guess<Value> for Value {
    type Target = Var;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        guess(context, *self)
    }
}

/// Implementation of [Guess] for [Vec].
impl<Value: IValue, T: Guess<Value>> Guess<Value> for Vec<T> {
    type Target = Vec<T::Target>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        self.iter().map(|value| value.guess(context)).collect()
    }
}

/// Implementation of [Guess] for `[T; N]`.
impl<Value: IValue, T: Guess<Value>, const N: usize> Guess<Value> for [T; N] {
    type Target = [T::Target; N];

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        self.each_ref().map(|value| value.guess(context))
    }
}

/// Implementation of [Guess] for `(T, S)`.
impl<Value: IValue, T: Guess<Value>, S: Guess<Value>> Guess<Value> for (T, S) {
    type Target = (T::Target, S::Target);

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        (self.0.guess(context), self.1.guess(context))
    }
}

impl<Value: IValue, T: Guess<Value>> Guess<Value> for CirclePoint<T> {
    type Target = CirclePoint<T::Target>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        CirclePoint { x: self.x.guess(context), y: self.y.guess(context) }
    }
}
