use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::eval;
use crate::stark_verifier::statement::OodsSamples;

/// Accumulates a psuedo-random linear combination of constraint evaluations at the OODS point.
///
/// Maintains the running value:
///   accumulation <- accumulation * random_coeff + c_i(oods)
/// so that after N constraints:
///   accumulation = Σ_{i=0..N-1} random_coeff^{N-1-i} * c_i(oods).
pub struct CompositionConstraintAccumulator<'a> {
    pub oods_samples: OodsSamples<'a>,
    pub composition_polynomial_coef: Var,
    pub interaction_elements: [Var; 2],
    pub claimed_sums: &'a [Var],
    pub random_coeff: Var,
    /// Running accumulator over constraint evaluations at the OODS point.
    pub accumulation: Var,
}

impl CompositionConstraintAccumulator<'_> {
    /// Incorporate the next constraint evaluation at the OODS point.
    pub fn accumulate(&mut self, context: &mut Context<impl IValue>, constraint_eval_at_oods: Var) {
        let shifted_accumulation = eval!(context, (self.accumulation) * (self.random_coeff));
        self.accumulation = eval!(context, (shifted_accumulation) + (constraint_eval_at_oods));
    }

    /// Finish accumulation and return the combined value.
    ///
    /// Panics if not all expected samples/claimed sums have been consumed.
    pub fn finalize(self) -> Var {
        assert!(self.oods_samples.trace.is_empty(), "unconsumed trace OODS samples");
        assert!(self.oods_samples.interaction.is_empty(), "unconsumed interaction OODS samples");
        assert!(self.claimed_sums.is_empty(), "unconsumed claimed sums");

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
        acc: &mut CompositionConstraintAccumulator<'_>,
    );

    fn public_logup_sum(
        &self,
        context: &mut Context<impl IValue>,
        prev_sum: Var,
        interaction_elements: [Var; 2],
    ) -> Var;
}
