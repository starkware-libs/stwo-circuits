use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::eval;
use crate::stark_verifier::logup::{LogupTerm, logup_term};
use crate::stark_verifier::statement::OodsSamples;
use itertools::Itertools;

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
    /// The sizes of the components.
    pub component_sizes: &'a [Var],
    /// Running accumulator over constraint evaluations at the OODS point and the previous point.
    pub accumulation: Var,
    pub terms: Vec<LogupTerm>,
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
        self.terms.push(logup_term(context, self.interaction_elements, numerator, element));
    }

    pub fn finalize_logup_in_pairs(&mut self, _context: &mut Context<impl IValue>, _log_size: u32) {
        // TODO(Gali): Implement.
    }
}

/// A trait for evaluating at some point or row.
pub trait CircuitEval {
    /// Evaluates the composition polynomial at the OODS point (after dividing by the domain
    /// polynomial).
    fn evaluate(
        &self,
        context: &mut Context<impl IValue>,
        acc: &mut CompositionConstraintAccumulator<'_>,
    );
}
