use crate::circuits::circuit::{Add, Eq, Mul, Sub};
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;

#[cfg(test)]
#[path = "ops_test.rs"]
pub mod test;

/// Adds an equality gate to the circuit.
pub fn eq(context: &mut Context<impl IValue>, a: Var, b: Var) {
    context.circuit.eq.push(Eq { in0: a.idx, in1: b.idx });
}

/// Adds an addition gate to the circuit, and returns the output variable.
pub fn add(context: &mut Context<impl IValue>, a: Var, b: Var) -> Var {
    let out = context.new_var(context.get(a) + context.get(b));
    context.circuit.add.push(Add { in0: a.idx, in1: b.idx, out: out.idx });
    out
}

/// Adds a subtraction gate to the circuit, and returns the output variable.
pub fn sub(context: &mut Context<impl IValue>, a: Var, b: Var) -> Var {
    let out = context.new_var(context.get(a) - context.get(b));
    context.circuit.sub.push(Sub { in0: a.idx, in1: b.idx, out: out.idx });
    out
}

/// Adds a multiplication gate to the circuit, and returns the output variable.
pub fn mul(context: &mut Context<impl IValue>, a: Var, b: Var) -> Var {
    let out = context.new_var(context.get(a) * context.get(b));
    context.circuit.mul.push(Mul { in0: a.idx, in1: b.idx, out: out.idx });
    out
}

/// Returns a new unconstrained variable with the given value.
pub fn guess<Value: IValue>(c: &mut Context<Value>, value: Value) -> Var {
    let out = c.new_var(value);
    // Add a trivial constraint so that the new variable appears once as a yield.
    c.circuit.add.push(Add { in0: out.idx, in1: c.zero().idx, out: out.idx });
    out
}
