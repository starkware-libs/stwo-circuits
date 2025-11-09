use indexmap::IndexMap;
use num_traits::{One, Zero};
use stwo::core::fields::qm31::QM31;

use crate::circuits::circuit::Circuit;
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::guess;
use crate::circuits::stats::Stats;

#[cfg(test)]
#[path = "context_test.rs"]
pub mod test;

/// Represents a variable in a [Circuit].
///
/// A [Var] represents a `QM31` value.
/// In some cases, it may be restricted to an `M31` or a boolean value by adding constraints to the
/// circuit. For example, `x = x * x` will enforce that `x` is either `0` or `1`.
#[derive(Clone, Copy)]
pub struct Var {
    pub idx: usize,
}
impl std::fmt::Debug for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.idx)
    }
}

/// Represents the information required to build a [Circuit].
///
/// Context can be used in two ways:
/// 1. To build just the [Circuit] with no concrete values for the variables.
/// 2. To build the [Circuit] with concrete `QM31` values for the variables.
///
/// In the first case the type `Value` is set to `NoValue`, and in the second case it is set to
/// `QM31`.
pub struct Context<Value: IValue> {
    pub circuit: Circuit,
    constants: IndexMap<QM31, Var>,
    n_vars: usize,
    values: Vec<Value>,
    pub stats: Stats,
}
impl<Value: IValue> Context<Value> {
    pub fn values(&self) -> &Vec<Value> {
        &self.values
    }

    pub fn constants(&self) -> &IndexMap<QM31, Var> {
        &self.constants
    }

    /// The number of variables allocated so far.
    pub fn n_vars(&self) -> usize {
        self.n_vars
    }

    pub fn zero(&self) -> Var {
        Var { idx: 0 }
    }

    pub fn one(&self) -> Var {
        Var { idx: 1 }
    }

    /// Creates a new variable.
    pub fn new_var(&mut self, value: Value) -> Var {
        let idx = self.n_vars;
        self.n_vars += 1;
        self.values.push(value);
        Var { idx }
    }

    /// Returns the value of a variable.
    pub fn get(&self, var: Var) -> Value {
        self.values[var.idx]
    }

    pub fn constant(&mut self, value: QM31) -> Var {
        if let Some(var) = self.constants.get(&value) {
            *var
        } else {
            let var = guess(self, Value::from_qm31(value));
            self.constants.insert(value, var);
            var
        }
    }
}

impl<Value: IValue> Default for Context<Value> {
    fn default() -> Self {
        let mut res = Self {
            circuit: Circuit::default(),
            constants: IndexMap::new(),
            n_vars: 0,
            values: vec![],
            stats: Stats::default(),
        };
        // Register zero and one as the first constants.
        res.constant(QM31::zero());
        res.constant(QM31::one());
        res
    }
}

/// A context with real QM31 values.
pub type TraceContext = Context<QM31>;
