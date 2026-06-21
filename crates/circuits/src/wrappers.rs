use std::fmt::Debug;

use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use crate::context::{Context, Var};
use crate::eval;
use crate::ivalue::IValue;
use crate::ivalue::NoValue;
use crate::ops::{Guess, guess_m31};

#[cfg(test)]
#[path = "wrappers_test.rs"]
pub mod test;

/// Represents a value that should be in the base field `M31`.
///
/// Using the [Guess] trait on [M31Wrapper] and gates that guarantee that the guessed value is
/// indeed in the base field `M31`.
#[derive(Clone, PartialEq)]
pub struct M31Wrapper<T>(T);

impl<T> M31Wrapper<T> {
    pub fn get(&self) -> &T {
        &self.0
    }

    pub fn new_unsafe(var: T) -> Self {
        Self(var)
    }
}

impl From<M31> for M31Wrapper<QM31> {
    fn from(value: M31) -> Self {
        M31Wrapper(value.into())
    }
}

impl M31Wrapper<Var> {
    /// Adds a multiplication gate to the circuit, and returns the output variable.
    pub fn mul(context: &mut Context<impl IValue>, a: Self, b: Self) -> Self {
        Self(eval!(context, (*a.get()) * (*b.get())))
    }
}

impl From<NoValue> for M31Wrapper<NoValue> {
    fn from(_: NoValue) -> Self {
        M31Wrapper(NoValue)
    }
}

impl<Value: IValue> Guess<Value> for M31Wrapper<Value> {
    type Target = M31Wrapper<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        // `guess_m31` constrains the guessed variable to the base field `M31` during
        // finalization, so no further masking is required here.
        guess_m31(context, self.clone())
    }
}

impl<T: Debug> Debug for M31Wrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "M31({:?})", self.0)
    }
}
