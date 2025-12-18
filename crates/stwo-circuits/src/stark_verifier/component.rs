use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::stark_verifier::statement::EvaluateArgs;

/// Represents a component.
pub trait Component {
    /// Evaluates the composition polynomial at the OODS point (after dividing by the domain
    /// polynomial).
    fn evaluate(&self, context: &mut Context<impl IValue>, prev_sum: Var, args: &mut EvaluateArgs<'_>) -> Var;
}
