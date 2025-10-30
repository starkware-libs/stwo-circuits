use itertools::chain;
use stwo::core::fields::qm31::QM31;

#[cfg(test)]
#[path = "circuit_test.rs"]
pub mod test;

fn check_eq(a: QM31, b: QM31) -> Result<(), String> {
    if a != b {
        return Err(format!("{a} != {b}"));
    }
    Ok(())
}

pub trait Gate: std::fmt::Debug {
    fn check(&self, values: &[QM31]) -> Result<(), String>;
}

/// Represents an addition gate in the circuit: `[in0] + [in1] = [out]`.
#[derive(PartialEq, Eq)]
pub struct Add {
    pub in0: usize,
    pub in1: usize,
    pub out: usize,
}
impl Gate for Add {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        check_eq(values[self.in0] + values[self.in1], values[self.out])
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
    pub in0: usize,
    pub in1: usize,
    pub out: usize,
}
impl Gate for Sub {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        check_eq(values[self.in0] - values[self.in1], values[self.out])
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
    pub in0: usize,
    pub in1: usize,
    pub out: usize,
}
impl Gate for Mul {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        check_eq(values[self.in0] * values[self.in1], values[self.out])
    }
}

impl std::fmt::Debug for Mul {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] = [{}] * [{}]", self.out, self.in0, self.in1)
    }
}

/// Represents an equality gate in the circuit: `[in0] = [in1]`.
#[derive(PartialEq, Eq)]
pub struct Eq {
    pub in0: usize,
    pub in1: usize,
}
impl Gate for Eq {
    fn check(&self, values: &[QM31]) -> Result<(), String> {
        check_eq(values[self.in0], values[self.in1])
    }
}

impl std::fmt::Debug for Eq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] = [{}]", self.in0, self.in1)
    }
}

/// Represents a circuit.
#[derive(Default, PartialEq, Eq)]
pub struct Circuit {
    pub add: Vec<Add>,
    pub sub: Vec<Sub>,
    pub mul: Vec<Mul>,
    pub eq: Vec<Eq>,
}

impl Circuit {
    /// Returns an iterator over all the gates in the circuit.
    pub fn all_gates(&self) -> impl Iterator<Item = &dyn Gate> {
        chain!(
            self.add.iter().map(|g| g as &dyn Gate),
            self.sub.iter().map(|g| g as &dyn Gate),
            self.mul.iter().map(|g| g as &dyn Gate),
            self.eq.iter().map(|g| g as &dyn Gate)
        )
    }

    /// Checks if the circuit is satisfied by the given values.
    pub fn check(&self, values: &[QM31]) -> Result<(), String> {
        for gate in self.all_gates() {
            gate.check(values)?;
        }
        Ok(())
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
