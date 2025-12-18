use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::eval;
use crate::stark_verifier::statement::OodsSamples;

pub struct PointEvaluationAccumulator<'a> {
    pub oods_samples: OodsSamples<'a>,
    pub composition_polynomial_coef: Var,
    pub interaction_elements: [Var; 2],
    pub claimed_sums: &'a [Var],
    pub random_coeff: Var,
    pub accumulation: Var,
}

impl PointEvaluationAccumulator<'_> {
    pub fn accumulate(&mut self, context: &mut Context<impl IValue>, evaluation: Var) {
        let shifted_accumulation = eval!(context, (self.accumulation) * (self.random_coeff));
        self.accumulation = eval!(context, (shifted_accumulation) + (evaluation));
    }

    pub fn finalize(self) -> Var {
        assert!(self.oods_samples.trace.is_empty());
        assert!(self.oods_samples.interaction.is_empty());
        assert!(self.claimed_sums.is_empty());

        self.accumulation
    }
}

/// Represents a component.
pub trait Component {
    /// Evaluates the composition polynomial at the OODS point (after dividing by the domain
    /// polynomial).
    fn evaluate(
        &self,
        context: &mut Context<impl IValue>,
        acc: &mut PointEvaluationAccumulator<'_>,
    );
}
