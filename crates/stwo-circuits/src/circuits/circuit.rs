use itertools::chain;
use stwo::core::fields::qm31::QM31;

use crate::circuits::blake::blake_qm31;
use crate::circuits::ivalue::IValue;

#[cfg(test)]
#[path = "circuit_test.rs"]
pub mod test;

/// Represents a variable in a [Circuit].
///
/// A [Var] represents a `QM31` value.
/// In some cases, it may be restricted to an `M31` or a boolean value by adding constraints to the
/// circuit. For example, `x = x * x` will enforce that `x` is either `0` or `1`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Var {
    pub idx: usize,
}
impl std::fmt::Debug for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.idx)
    }
}
impl std::fmt::Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.idx)
    }
}

fn check_eq(a: QM31, b: QM31) -> Result<(), String> {
    if a != b {
        return Err(format!("{a} != {b}"));
    }
    Ok(())
}

pub trait Gate: std::fmt::Debug {
    fn check(&self, values: &[QM31]) -> Result<(), String>;

    /// Returns the variables that are "used" by the gate (in the context of lookup terms).
    fn uses(&self) -> Vec<Var>;

    /// Returns the variables that are "yielded" by the gate (in the context of lookup terms).
    fn yields(&self) -> Vec<Var>;
}

/// Represents an addition gate in the circuit: `[in0] + [in1] = [out]`.
#[derive(PartialEq, Eq)]
pub struct Add {
    pub in0: Var,
    pub in1: Var,
    pub out: Var,
}
impl Gate for Add {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        check_eq(values[self.in0.idx] + values[self.in1.idx], values[self.out.idx])
    }

    fn uses(&self) -> Vec<Var> {
        vec![self.in0, self.in1]
    }

    fn yields(&self) -> Vec<Var> {
        vec![self.out]
    }
}

impl std::fmt::Debug for Add {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] = [{}] + [{}]", self.out, self.in0, self.in1)
    }
}

/// Represents a subtraction gate in the circuit: `[in0] - [in1] = [out]`.
#[derive(PartialEq, Eq)]
pub struct Sub {
    pub in0: Var,
    pub in1: Var,
    pub out: Var,
}
impl Gate for Sub {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        check_eq(values[self.in0.idx] - values[self.in1.idx], values[self.out.idx])
    }

    fn uses(&self) -> Vec<Var> {
        vec![self.in0, self.in1]
    }

    fn yields(&self) -> Vec<Var> {
        vec![self.out]
    }
}

impl std::fmt::Debug for Sub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] = [{}] - [{}]", self.out, self.in0, self.in1)
    }
}

/// Represents a multiplication gate in the circuit: `[in0] * [in1] = [out]`.
#[derive(PartialEq, Eq)]
pub struct Mul {
    pub in0: Var,
    pub in1: Var,
    pub out: Var,
}
impl Gate for Mul {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        check_eq(values[self.in0.idx] * values[self.in1.idx], values[self.out.idx])
    }

    fn uses(&self) -> Vec<Var> {
        vec![self.in0, self.in1]
    }

    fn yields(&self) -> Vec<Var> {
        vec![self.out]
    }
}

impl std::fmt::Debug for Mul {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] = [{}] * [{}]", self.out, self.in0, self.in1)
    }
}

/// Represents a pointwise multiplication gate in the circuit.
/// See [IValue::pointwise_mul] for more details.
#[derive(PartialEq, Eq)]
pub struct PointwiseMul {
    pub in0: Var,
    pub in1: Var,
    pub out: Var,
}
impl Gate for PointwiseMul {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        check_eq(
            QM31::pointwise_mul(values[self.in0.idx], values[self.in1.idx]),
            values[self.out.idx],
        )
    }

    fn uses(&self) -> Vec<Var> {
        vec![self.in0, self.in1]
    }

    fn yields(&self) -> Vec<Var> {
        vec![self.out]
    }
}

impl std::fmt::Debug for PointwiseMul {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] = [{}] x [{}]", self.out, self.in0, self.in1)
    }
}

/// Represents an equality gate in the circuit: `[in0] = [in1]`.
#[derive(PartialEq, Eq)]
pub struct Eq {
    pub in0: Var,
    pub in1: Var,
}
impl Gate for Eq {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        check_eq(values[self.in0.idx], values[self.in1.idx])
    }

    fn uses(&self) -> Vec<Var> {
        vec![self.in0, self.in1]
    }

    fn yields(&self) -> Vec<Var> {
        vec![]
    }
}

impl std::fmt::Debug for Eq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] = [{}]", self.in0, self.in1)
    }
}

/// Represents a blake hash gate in the circuit: `([out0], [out1]) = blake([input]; n_bytes)`.
#[derive(PartialEq, Eq)]
pub struct Blake {
    pub input: Vec<[Var; 4]>,
    pub n_bytes: usize,
    pub out0: Var,
    pub out1: Var,
}
impl Gate for Blake {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        let input = self.input.iter().flatten().map(|var| values[var.idx]).collect::<Vec<_>>();
        let n_effective_vars = self.n_bytes.div_ceil(16);
        let main_part = &input[..n_effective_vars];
        let remaining_part = &input[n_effective_vars..];

        let expected_output = blake_qm31(main_part, self.n_bytes);
        check_eq(values[self.out0.idx], expected_output.0)?;
        check_eq(values[self.out1.idx], expected_output.1)?;

        // Sanity check: Check that the remaining input is zero.
        for val in remaining_part {
            check_eq(*val, 0.into())?;
        }

        Ok(())
    }

    fn uses(&self) -> Vec<Var> {
        self.input.iter().flatten().copied().collect()
    }

    fn yields(&self) -> Vec<Var> {
        vec![self.out0, self.out1]
    }
}

impl std::fmt::Debug for Blake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "([{}], [{}]) = blake({:?}; {})", self.out0, self.out1, self.input, self.n_bytes)
    }
}

/// Represents a circuit.
#[derive(Default, PartialEq, Eq)]
pub struct Circuit {
    pub n_vars: usize,
    pub add: Vec<Add>,
    pub sub: Vec<Sub>,
    pub mul: Vec<Mul>,
    pub pointwise_mul: Vec<PointwiseMul>,
    pub eq: Vec<Eq>,
    pub blake: Vec<Blake>,
}

impl Circuit {
    /// Returns an iterator over all the gates in the circuit.
    pub fn all_gates(&self) -> impl Iterator<Item = &dyn Gate> {
        let Circuit { n_vars: _, add, sub, mul, pointwise_mul, eq, blake } = self;
        chain!(
            add.iter().map(|g| g as &dyn Gate),
            sub.iter().map(|g| g as &dyn Gate),
            mul.iter().map(|g| g as &dyn Gate),
            pointwise_mul.iter().map(|g| g as &dyn Gate),
            eq.iter().map(|g| g as &dyn Gate),
            blake.iter().map(|g| g as &dyn Gate),
        )
    }

    /// Checks if the circuit is satisfied by the given values.
    pub fn check(&self, values: &[QM31]) -> Result<(), String> {
        for gate in self.all_gates() {
            gate.check(values)?;
        }
        Ok(())
    }

    /// Returns the number of uses and number of yields for each variable (in the context of lookup
    /// terms).
    pub fn compute_multiplicities(&self) -> (Vec<usize>, Vec<usize>) {
        let mut n_uses = vec![0; self.n_vars];
        let mut n_yields = vec![0; self.n_vars];

        for gate in self.all_gates() {
            for use_var in gate.uses() {
                n_uses[use_var.idx] += 1;
            }
            for yield_var in gate.yields() {
                n_yields[yield_var.idx] += 1;
            }
        }

        (n_uses, n_yields)
    }

    /// Verifies that each variable appears exactly once as a yield.
    pub fn check_yields(&self) {
        for (idx, n_yields) in self.compute_multiplicities().1.iter().enumerate() {
            assert!(*n_yields == 1, "Variable {idx} appears {n_yields} times as a yield");
        }
    }
}

impl std::fmt::Debug for Circuit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for gate in self.all_gates() {
            writeln!(f, "{gate:?}")?;
        }
        Ok(())
    }
}
