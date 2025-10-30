use stwo::core::fields::qm31::QM31;

use crate::circuits::circuit::Circuit;
use crate::circuits::ivalue::IValue;

/// Represents a variable in a [Circuit].
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
pub struct Context<Value: IValue> {
    pub circuit: Circuit,
    n_vars: usize,
    values: Vec<Value>,
}
impl<Value: IValue> Context<Value> {
    pub fn values(&self) -> &Vec<Value> {
        &self.values
    }

    /// The number of variables allocated so far.
    pub fn n_vars(&self) -> usize {
        self.n_vars
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
}

impl<Value: IValue> Default for Context<Value> {
    fn default() -> Self {
        Self { circuit: Circuit::default(), n_vars: 0, values: vec![] }
    }
}

/// A context with real QM31 values.
pub type TraceContext = Context<QM31>;
