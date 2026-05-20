use hashbrown::HashSet;
use indexmap::{IndexMap, IndexSet};
use num_traits::{One, Zero};
use stwo::core::fields::qm31::QM31;

use crate::circuit::{Add, Circuit};
use crate::ivalue::{IValue, qm31_from_u32s};
use crate::stats::Stats;

#[cfg(test)]
#[path = "context_test.rs"]
pub mod test;

/// Variable index reserved for the QM31 constant `u = (0, 0, 1, 0)`. `Context::default()` registers
/// `zero`, `one`, and `u` as the first three constants.
///
/// Keep in sync with the registration order in `Context::default`.
pub const U_ADDRESS: usize = 2;

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
    /// [Self::check_vars_used] checks that these are indeed unused.
    unused_vars: HashSet<usize>,
    /// Set of variables that were marked by the code as "maybe unused". These skip
    /// the checks in [Self::check_vars_used] entirely.
    maybe_unused_vars: HashSet<usize>,
    /// Set of variables that are not the result of a gate, but instead are provided by the prover.
    ///
    /// `None` if the set of guessed variables has already been finalized.
    ///
    /// See [crate::ops::guess].
    pub guessed_vars: Option<Vec<usize>>,
    /// Variables allocated by [Self::reserve] whose values have not yet been supplied by
    /// [Self::fulfill]. Reading these via [Self::get] trips a debug assertion, and
    /// [Self::finalize_guessed_vars] panics if any are still pending.
    reserved_vars: IndexSet<usize>,
    /// Debug only. If true, equality is asserted when adding the `eq` gate; if false, no
    /// assertion is made during construction and equality can be checked later at validation.
    pub assert_eq_on_eval: bool,
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

    pub fn u(&self) -> Var {
        Var { idx: U_ADDRESS }
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
        debug_assert!(
            !self.reserved_vars.contains(&var.idx),
            "read of reserved variable [{}] before it was fulfilled",
            var.idx,
        );
        self.values[var.idx]
    }

    /// Allocates a fresh variable with no concrete value yet.
    ///
    /// The returned [`ReservedVar`] can be used as a gate input/output immediately — its
    /// [`Var::idx`] is fixed at this call. The actual value must be supplied later via
    /// [`Self::fulfill`]. The caller is responsible for arranging a gate whose `out` is the
    /// reserved variable so that the single-yield invariant holds; [`Self::fulfill`] does not
    /// add any gate by itself.
    ///
    /// Reading the value before fulfillment trips a debug assertion in [`Self::get`].
    /// [`Self::finalize_guessed_vars`] panics if any reservation is still outstanding.
    pub fn reserve(&mut self) -> Var {
        let reserved = self.new_var(Value::placeholder());
        let inserted = self.reserved_vars.insert(reserved.idx);
        debug_assert!(inserted);
        reserved
    }

    /// Returns the currently outstanding reservations.
    pub fn reserved(&self) -> Vec<Var> {
        self.reserved_vars.iter().map(|&x| Var { idx: x }).collect()
    }

    /// Supplies the value for a previously [reserved](Self::reserve) variable and returns the
    /// underlying [`Var`] for further use.
    ///
    /// Panics if the variable has already been fulfilled or was never reserved.
    fn fill_reserved(&mut self, reserved: Var, value: Value) {
        let removed = self.reserved_vars.shift_remove(&reserved.idx);
        assert!(removed, "variable [{}] was not reserved or was already fulfilled", reserved.idx);
        self.values[reserved.idx] = value;
    }

    pub fn copy_into_reserved(&mut self, reserved: Var, var: Var) {
        let value = self.get(var);
        self.fill_reserved(reserved, value);
        let zero = self.zero();
        // Yield and constrain the reserved var.
        self.circuit.add.push(Add { in0: var.idx, in1: zero.idx, out: reserved.idx });
    }

    pub fn constant(&mut self, value: QM31) -> Var {
        if let Some(var) = self.constants.get(&value) {
            *var
        } else {
            let var = self.new_var(Value::from_qm31(value));
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

    /// Marks a variable as "maybe unused".
    ///
    /// See [Self::check_vars_used].
    pub fn mark_as_maybe_unused(&mut self, var: &Var) {
        assert!(self.maybe_unused_vars.insert(var.idx));
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
            if !unused && marked_as_unused {
                panic!("Variable {idx} is used but marked as unused");
            }
            if unused && !(marked_as_unused || self.maybe_unused_vars.contains(&idx)) {
                panic!("Variable {idx} is unused but not marked as unused");
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
        assert!(
            self.reserved_vars.is_empty(),
            "{} reserved variable(s) were never fulfilled (idxs: {:?})",
            self.reserved_vars.len(),
            self.reserved_vars,
        );
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
            maybe_unused_vars: HashSet::new(),
            guessed_vars: Some(vec![]),
            reserved_vars: IndexSet::new(),
            assert_eq_on_eval: false,
        };
        // Register zero, one, and u as the first constants.
        res.constant(QM31::zero());
        res.constant(QM31::one());
        res.constant(qm31_from_u32s(0, 0, 1, 0)); // u at idx 2

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

    pub fn enable_assert_eq_on_eval(&mut self) {
        self.assert_eq_on_eval = true;
    }
}
