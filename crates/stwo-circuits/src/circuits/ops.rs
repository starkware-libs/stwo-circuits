use crate::circuits::circuit::{Add, Eq, Mul, Sub};
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;

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
    context.stats.eq += 1;
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

/// Returns a new unconstrained variable with the given value.
pub fn guess<Value: IValue>(context: &mut Context<Value>, value: Value) -> Var {
    context.stats.guess += 1;
    let out = context.new_var(value);
    // Add a trivial constraint so that the new variable appears once as a yield.
    context.circuit.add.push(Add { in0: out.idx, in1: context.zero().idx, out: out.idx });
    out
}
