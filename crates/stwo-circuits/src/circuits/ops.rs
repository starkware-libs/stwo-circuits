use crate::circuits::circuit::{Add, Eq, Mul, Sub};
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;

#[cfg(test)]
#[path = "ops_test.rs"]
pub mod test;

/// Adds an equality gate to the circuit.
pub fn eq(c: &mut Context<impl IValue>, a: Var, b: Var) {
    c.circuit.eq.push(Eq { in0: a.idx, in1: b.idx });
}

/// Adds an addition gate to the circuit, and returns the output variable.
pub fn add(c: &mut Context<impl IValue>, a: Var, b: Var) -> Var {
    let out = c.new_var(c.get(a) + c.get(b));
    c.circuit.add.push(Add { in0: a.idx, in1: b.idx, out: out.idx });
    out
}

/// Adds a subtraction gate to the circuit, and returns the output variable.
pub fn sub(c: &mut Context<impl IValue>, a: Var, b: Var) -> Var {
    let out = c.new_var(c.get(a) - c.get(b));
    c.circuit.sub.push(Sub { in0: a.idx, in1: b.idx, out: out.idx });
    out
}

/// Adds a multiplication gate to the circuit, and returns the output variable.
pub fn mul(c: &mut Context<impl IValue>, a: Var, b: Var) -> Var {
    let out = c.new_var(c.get(a) * c.get(b));
    c.circuit.mul.push(Mul { in0: a.idx, in1: b.idx, out: out.idx });
    out
}
