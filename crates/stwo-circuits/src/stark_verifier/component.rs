use itertools::Itertools;
use stwo::core::fields::qm31::SECURE_EXTENSION_DEGREE;

use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::{div, from_partial_evals};
use crate::eval;
use crate::stark_verifier::logup::{Frac, get_frac, single_logup_term};
use crate::stark_verifier::statement::OodsSamples;

/// Accumulates a psuedo-random linear combination of constraint evaluations at the OODS point and
/// the previous point.
///
/// Maintains the running value:
///   accumulation <- accumulation * composition_polynomial_coeff + c_i
/// so that after N constraints:
///   accumulation = Σ_{i=0..N-1} composition_polynomial_coeff^{N-1-i} * c_i.
pub struct CompositionConstraintAccumulator<'a> {
    /// The OODS samples for the preprocessed columns, trace, and interaction.
    /// Each component will consume a subset of these samples.
    pub oods_samples: OodsSamples<'a>,
    /// The random coefficient for the composition polynomial.
    pub composition_polynomial_coeff: Var,
    /// The interaction elements for the logup sums constraint.
    pub interaction_elements: [Var; 2],
    /// The claimed sums for the component.
    /// Each component consumes one claimed sum.
    pub claimed_sums: &'a [Var],
    /// Running accumulator over constraint evaluations at the OODS point and the previous point.
    pub accumulation: Var,
    pub fracs: Vec<Frac>,
}

impl CompositionConstraintAccumulator<'_> {
    /// Incorporate the next constraint evaluation at the OODS point.
    pub fn accumulate(&mut self, context: &mut Context<impl IValue>, constraint_eval_at_oods: Var) {
        let shifted_accumulation =
            eval!(context, (self.accumulation) * (self.composition_polynomial_coeff));
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

    pub fn get_preprocessed_columns<const N_PP_COLUMNS: usize>(
        &mut self,
        preprocessed_column_indices: [usize; N_PP_COLUMNS],
    ) -> [Var; N_PP_COLUMNS] {
        preprocessed_column_indices
            .iter()
            .map(|idx| self.oods_samples.preprocessed_columns[*idx])
            .collect_vec()
            .try_into()
            .unwrap_or_else(|_| panic!("Expected {N_PP_COLUMNS} preprocessed columns"))
    }

    pub fn get_trace<const N_TRACE_COLUMNS: usize>(&mut self) -> [Var; N_TRACE_COLUMNS] {
        if let Some(vec) = self.oods_samples.trace.split_off(..N_TRACE_COLUMNS) {
            vec.try_into().unwrap()
        } else {
            panic!("Expected {N_TRACE_COLUMNS} trace values")
        }
    }

    pub fn add_constraint(
        &mut self,
        context: &mut Context<impl IValue>,
        constraint_eval_at_oods: Var,
    ) {
        self.accumulate(context, constraint_eval_at_oods);
    }

    pub fn add_to_relation(
        &mut self,
        context: &mut Context<impl IValue>,
        numerator: Var,
        element: &[Var],
    ) {
        self.fracs.push(get_frac(context, self.interaction_elements, numerator, element));
    }

    pub fn finalize_logup_in_pairs(&mut self, context: &mut Context<impl IValue>, log_size: u32) {
        // TODO(Gali): Add more fracs.
        let Some(
            [interaction_0_limb0, interaction_0_limb1, interaction_0_limb2, interaction_0_limb3],
        ) = self
            .oods_samples
            .interaction
            .split_off(..self.fracs.len().div_ceil(2) * SECURE_EXTENSION_DEGREE)
        else {
            panic!(
                "Expected {} interaction values",
                self.fracs.len().div_ceil(2) * SECURE_EXTENSION_DEGREE
            );
        };
        let Some([claimed_sum]) = self.claimed_sums.split_off(..1) else {
            panic!("Expected 1 claimed sum");
        };

        let prev_logup_sum = from_partial_evals(
            context,
            [
                interaction_0_limb0.at_prev,
                interaction_0_limb1.at_prev,
                interaction_0_limb2.at_prev,
                interaction_0_limb3.at_prev,
            ],
        );
        let cur_logup_sum = from_partial_evals(
            context,
            [
                interaction_0_limb0.at_oods,
                interaction_0_limb1.at_oods,
                interaction_0_limb2.at_oods,
                interaction_0_limb3.at_oods,
            ],
        );
        let n_instances = context.constant((1 << log_size).into());
        let cumsum_shift = div(context, *claimed_sum, n_instances);
        let diff = eval!(context, (cur_logup_sum) - (prev_logup_sum));
        let shifted_diff = eval!(context, (diff) + (cumsum_shift));

        let logup_constraint_val = single_logup_term(context, self.fracs[0].clone(), shifted_diff);
        self.add_constraint(context, logup_constraint_val);
        self.fracs.clear();
    }
}
/// Represents a component.
pub trait CircuitEval {
    /// Evaluates the composition polynomial at the OODS point (after dividing by the domain
    /// polynomial).
    fn evaluate(
        &self,
        context: &mut Context<impl IValue>,
        acc: &mut CompositionConstraintAccumulator<'_>,
    );
}
