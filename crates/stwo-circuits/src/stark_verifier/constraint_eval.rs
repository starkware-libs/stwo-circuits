use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::{div, from_partial_evals};
use crate::eval;
use crate::stark_verifier::circle::denom_inverse;
use crate::stark_verifier::logup::{
    LogupTerm, logup_term, pair_logup_constraint, single_logup_constraint,
};
use crate::stark_verifier::statement::{EvaluateArgs, OodsSamples};
use itertools::Itertools;
use stwo::core::fields::qm31::SECURE_EXTENSION_DEGREE;

// Data accosiated with a specific compoonent.
pub struct ComponentData {
    /// The number of instances in the component.
    pub n_instances: Var,
    /// The claimed sum of the component.
    pub claimed_sum: Var,
}

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
    /// The data associated with each component.
    pub component_data: &'a [ComponentData],
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
        assert!(self.component_data.is_empty(), "unconsumed component data");

        self.accumulation
    }

    pub fn get_preprocessed_columns<const N_PP_COLUMNS: usize>(
        &mut self,
        preprocessed_column_indices: [usize; N_PP_COLUMNS],
    ) -> [Var; N_PP_COLUMNS] {
        std::array::from_fn(|idx| {
            self.oods_samples.preprocessed_columns[preprocessed_column_indices[idx]]
        })
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

    pub fn finalize_logup_in_pairs(&mut self, context: &mut Context<impl IValue>) {
        // TODO(Gali): Get the terms from the component instead of storing them in the accumulator.
        let n_batches = self.terms.len().div_ceil(2);
        let n_interacion_columns = n_batches * SECURE_EXTENSION_DEGREE;
        let Some(interaction_columns) =
            self.oods_samples.interaction.split_off(..n_interacion_columns)
        else {
            panic!("Expected {n_interacion_columns} interaction values");
        };
        let (prev_logup_sums, cur_logup_sums): (Vec<Var>, Vec<Var>) = interaction_columns
            .iter()
            .chunks(SECURE_EXTENSION_DEGREE)
            .into_iter()
            .map(|chunk| {
                let chunk_vec: Vec<_> = chunk.collect();
                let [prev_limb0, prev_limb1, prev_limb2, prev_limb3] =
                    std::array::from_fn(|i| chunk_vec[i].at_prev);
                let [cur_limb0, cur_limb1, cur_limb2, cur_limb3] =
                    std::array::from_fn(|i| chunk_vec[i].at_oods);
                (
                    from_partial_evals(context, [prev_limb0, prev_limb1, prev_limb2, prev_limb3]),
                    from_partial_evals(context, [cur_limb0, cur_limb1, cur_limb2, cur_limb3]),
                )
            })
            .collect();

        let mut prev_col_cumsum = context.zero();

        // All pairs except the last are cumulatively summed in new interaction columns.
        (0..(n_batches - 1)).for_each(|i| {
            let cur_cumsum = cur_logup_sums[i];
            let diff = eval!(context, (cur_cumsum) - (prev_col_cumsum));
            prev_col_cumsum = cur_cumsum;

            let logup_constraint_val =
                pair_logup_constraint(context, self.terms[2 * i], self.terms[2 * i + 1], diff);
            self.add_constraint(context, logup_constraint_val);
            context.mark_as_unused(prev_logup_sums[i]);
        });

        let [prev_row_cumsum, cur_cumsum] =
            [prev_logup_sums[n_batches - 1], cur_logup_sums[n_batches - 1]];

        let diff = eval!(context, ((cur_cumsum) - (prev_row_cumsum)) - (prev_col_cumsum));

        let &ComponentData { claimed_sum, n_instances } =
            self.component_data.split_off_first().unwrap();
        let cumsum_shift = div(context, claimed_sum, n_instances);
        // Instead of checking diff = num / denom, check diff = num / denom - cumsum_shift.
        // This makes (num / denom - cumsum_shift) have sum zero, which makes the constraint
        // uniform - apply on all rows.
        let shifted_diff = eval!(context, (diff) + (cumsum_shift));

        let logup_constraint_val = if self.terms.len().is_multiple_of(2) {
            pair_logup_constraint(
                context,
                self.terms[2 * (n_batches - 1)],
                self.terms[2 * (n_batches - 1) + 1],
                shifted_diff,
            )
        } else {
            single_logup_constraint(context, self.terms[2 * (n_batches - 1)], shifted_diff)
        };
        self.add_constraint(context, logup_constraint_val);

        self.terms.clear();
    }
}

/// A trait for evaluating at some point or row.
pub trait CircuitEval<Value: IValue> {
    /// Evaluates the composition polynomial at the OODS point (after dividing by the domain
    /// polynomial).
    fn evaluate(
        &self,
        context: &mut Context<Value>,
        acc: &mut CompositionConstraintAccumulator<'_>,
    );
}




pub fn compute_composition_polynomial<Value: IValue>(context: &mut Context<Value>, components: &[Box<dyn CircuitEval<Value>>], args: EvaluateArgs<'_>) -> Var {
    let EvaluateArgs {
        oods_samples,
        pt,
        log_domain_size,
        composition_polynomial_coeff,
        interaction_elements,
        component_data,
    } = args;

    let mut evaluation_accumulator = CompositionConstraintAccumulator {
        oods_samples,
        composition_polynomial_coeff,
        interaction_elements,
        component_data,
        accumulation: context.zero(),
        terms: Vec::new(),
    };

    for component in components {
        component.evaluate(context, &mut evaluation_accumulator);
    }

    let final_evaluation = evaluation_accumulator.finalize();

    let denom_inverse = denom_inverse(context, pt.x, log_domain_size);
    eval!(context, (final_evaluation) * (denom_inverse))
}