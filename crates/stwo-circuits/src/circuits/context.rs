use hashbrown::HashSet;
use indexmap::IndexMap;
use itertools::Itertools;
use num_traits::{One, Zero};
use stwo::core::fields::qm31::QM31;

use crate::circuits::circuit::{Add, Circuit, Permutation};
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
    values: Vec<Value>,
    pub stats: Stats,
    /// Set of variables that were marked by the code as "unused".
    /// Used in [Self::check_vars_used].
    unused_vars: HashSet<usize>,
    /// Set of variables that are not the result of a gate, but instead are provided by the prover.
    ///
    /// `None` if the set of guessed variables has already been finalized.
    ///
    /// See [guess].
    pub guessed_vars: Option<Vec<usize>>,
}
impl<Value: IValue> Context<Value> {
    pub fn values(&self) -> &Vec<Value> {
        &self.values
    }

    pub fn constants(&self) -> &IndexMap<QM31, Var> {
        &self.constants
    }

    pub fn zero(&self) -> Var {
        Var { idx: 0 }
    }

    pub fn one(&self) -> Var {
        Var { idx: 1 }
    }

    /// Creates a new variable.
    pub fn new_var(&mut self, value: Value) -> Var {
        let idx = self.circuit.n_vars;
        self.circuit.n_vars += 1;
        self.values.push(value);
        Var { idx }
    }

    /// Returns the value of a variable.
    pub fn get(&self, var: Var) -> Value {
        self.values[var.idx]
    }


    /// Permutes the input values using the given function and returns the new variables.
    pub fn permute(
        &mut self,
        inputs: &[Var],
        permute_fn: impl FnOnce(&[Value]) -> Vec<Value>,
    ) -> Vec<Var> {
        let outputs: Vec<Var> = permute_fn(&inputs.iter().map(|var| self.get(*var)).collect_vec())
            .iter()
            .map(|value| self.new_var(*value))
            .collect();
        self.circuit.permutation.push(Permutation {
            inputs: inputs.iter().map(|var| var.idx).collect(),
            outputs: outputs.iter().map(|var| var.idx).collect(),
        });
        outputs
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

    /// Marks a variable as unused.
    ///
    /// See [Self::check_vars_used].
    pub fn mark_as_unused(&mut self, var: Var) {
        assert!(self.unused_vars.insert(var.idx));
    }

    /// Checks that all the variables that were defined are used in the circuit by some gate.
    ///
    /// This is a sanity check for the correction of the circuit.
    ///
    /// Variables that were marked as unused by the code are excluded.
    pub fn check_vars_used(&self) {
        let var_uses = self.circuit.compute_multiplicities().0;
        for (idx, uses) in var_uses.iter().enumerate() {
            let unused = *uses == 0;
            let marked_as_unused = self.unused_vars.contains(&idx);
            if unused && !marked_as_unused {
                panic!("Variable {idx} is unused but not marked as unused");
            }
            if !unused && marked_as_unused {
                panic!("Variable {idx} is used but marked as unused");
            }
        }
    }

    /// Finalizes the set of guessed variables by adding a trivial constraint for each guessed
    /// variable.
    ///
    /// Each gate in the circuit has lookups for its inputs (use lookups) and outputs (yield
    /// lookups).
    /// For the lookup constraints to hold and be sound, we need to make sure that each variable
    /// appears exactly once as a yield lookup.
    /// For guessed value, add a trivial constraint so that the new variable appears once as
    /// a yield.
    pub fn finalize_guessed_vars(&mut self) {
        for idx in self.guessed_vars.take().unwrap().iter() {
            self.circuit.add.push(Add { in0: *idx, in1: self.zero().idx, out: *idx });
        }
    }
}

impl<Value: IValue> Default for Context<Value> {
    fn default() -> Self {
        let mut res = Self {
            circuit: Circuit::default(),
            constants: IndexMap::new(),
            values: vec![],
            stats: Stats::default(),
            unused_vars: HashSet::new(),
            guessed_vars: Some(vec![]),
        };
        // Register zero and one as the first constants.
        res.constant(QM31::zero());
        res.constant(QM31::one());
        res
    }
}

/// A context with real QM31 values.
pub type TraceContext = Context<QM31>;

impl TraceContext {
    /// Returns `true` if the values satisfy the circuit.
    pub fn is_circuit_valid(&self) -> bool {
        self.circuit.check(self.values()).is_ok()
    }

    /// Validates that the values satisfy the circuit.
    pub fn validate_circuit(&self) {
        self.circuit.check(self.values()).unwrap();
    }
}
