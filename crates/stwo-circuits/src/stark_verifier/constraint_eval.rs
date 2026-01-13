use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::{div, from_partial_evals};
use crate::circuits::simd::Simd;
use crate::eval;
use crate::stark_verifier::circle::denom_inverse;
use crate::stark_verifier::logup::{
    LogupTerm, logup_term, pair_logup_constraint, single_logup_constraint,
};
use crate::stark_verifier::proof::{InteractionAtOods, ProofConfig};
use crate::stark_verifier::statement::EvaluateArgs;
use crate::stark_verifier::verify::MAX_TRACE_SIZE_BITS;
use itertools::{Itertools, izip};
use stwo::core::fields::qm31::SECURE_EXTENSION_DEGREE;

// Data accosiated with a specific compoonent.
pub struct ComponentData<'a> {
    /// The trace columns of the component.
    pub trace_columns: &'a [Var],
    /// The interaction columns of the component.
    pub interaction_columns: &'a [InteractionAtOods<Var>],
    /// The number of instances in the component.
    pub n_instances: Var,
    /// The claimed sum of the component.
    pub claimed_sum: Var,

    /// The index of the component.
    index: usize,

    /// Simd of bits representing the `n_instances` in the component.
    n_instances_bits: &'a [Simd; MAX_TRACE_SIZE_BITS],
}

impl<'a> ComponentData<'a> {
    pub fn get_n_instances_bits(
        &self,
        context: &mut Context<impl IValue>,
    ) -> [Var; MAX_TRACE_SIZE_BITS] {
        self.n_instances_bits.each_ref().map(|bits| Simd::unpack_idx(context, bits, self.index))
    }
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
    pub preprocessed_columns: &'a [Var],
    /// The random coefficient for the composition polynomial.
    pub composition_polynomial_coeff: Var,
    /// The interaction elements for the logup sums constraint.
    pub interaction_elements: [Var; 2],
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
        self.accumulation
    }

    pub fn get_preprocessed_column(&self, preprocessed_column_idx: usize) -> Var {
        self.preprocessed_columns[preprocessed_column_idx]
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

    pub fn finalize_logup_in_pairs(
        &mut self,
        context: &mut Context<impl IValue>,
        interaction_columns: &[InteractionAtOods<Var>],
        component_data: &ComponentData<'_>,
    ) {
        // TODO(Gali): Get the terms from the component instead of storing them in the accumulator.
        let n_batches = self.terms.len().div_ceil(2);
        let (interaction_columns, last_chunk) = interaction_columns.split_last_chunk().unwrap();
        let mut prev_col_cumsum = context.zero();

        for (i, mut chunk_iter) in
            interaction_columns.iter().chunks(SECURE_EXTENSION_DEGREE).into_iter().enumerate()
        {
            let cur_cumsum = from_partial_evals(
                context,
                std::array::from_fn(|_| chunk_iter.next().unwrap().at_oods),
            );
            // All pairs except the last are cumulatively summed in new interaction columns.
            let diff =
                if i > 1 { eval!(context, (cur_cumsum) - (prev_col_cumsum)) } else { cur_cumsum };
            prev_col_cumsum = cur_cumsum;

            let logup_constraint_val =
                pair_logup_constraint(context, self.terms[2 * i], self.terms[2 * i + 1], diff);
            self.add_constraint(context, logup_constraint_val);
        }

        let prev_row_cumsum =
            from_partial_evals(context, last_chunk.each_ref().map(|x| x.at_prev.unwrap()));
        let cur_cumsum = from_partial_evals(context, last_chunk.each_ref().map(|x| x.at_oods));

        let diff = eval!(context, ((cur_cumsum) - (prev_row_cumsum)) - (prev_col_cumsum));
        let cumsum_shift = div(context, component_data.claimed_sum, component_data.n_instances);
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
        component_data: &ComponentData<'_>,
        acc: &mut CompositionConstraintAccumulator<'_>,
    );
}

pub fn get_n_columns<'a, T>(columns: &mut &'a [T], n: usize) -> &'a [T] {
    if let Some(vec) = columns.split_off(..n) { vec } else { panic!("Expected {n} columns") }
}

pub fn compute_composition_polynomial<Value: IValue>(
    context: &mut Context<Value>,
    config: &ProofConfig,
    components: &[Box<dyn CircuitEval<Value>>],
    args: EvaluateArgs<'_>,
) -> Var {
    let EvaluateArgs {
        mut oods_samples,
        pt,
        log_domain_size,
        composition_polynomial_coeff,
        interaction_elements,
        claimed_sums,
        component_sizes,
        n_instances_bits,
    } = args;

    let mut evaluation_accumulator = CompositionConstraintAccumulator {
        preprocessed_columns: oods_samples.preprocessed_columns,
        composition_polynomial_coeff,
        interaction_elements,
        accumulation: context.zero(),
        terms: Vec::new(),
    };

    for (
        component_index,
        (
            component,
            n_trace_columns_in_component,
            n_interaction_columns_in_component,
            &claimed_sum,
            &component_size,
        ),
    ) in izip!(
        components,
        &config.trace_columns_per_component,
        &config.interaction_columns_per_component,
        claimed_sums,
        component_sizes,
    )
    .enumerate()
    {
        let trace_columns = get_n_columns(&mut oods_samples.trace, *n_trace_columns_in_component);
        let interaction_columns =
            get_n_columns(&mut oods_samples.interaction, *n_interaction_columns_in_component);

        let component_data = ComponentData {
            trace_columns,
            interaction_columns,
            claimed_sum,
            n_instances: component_size,
            index: component_index,
            n_instances_bits,
        };

        component.evaluate(context, &component_data, &mut evaluation_accumulator);

        evaluation_accumulator.finalize_logup_in_pairs(
            context,
            interaction_columns,
            &component_data,
        );
    }

    assert!(oods_samples.trace.is_empty(), "unconsumed trace columns");
    assert!(oods_samples.interaction.is_empty(), "unconsumed interaction columns");

    let final_evaluation = evaluation_accumulator.finalize();
    let denom_inverse = denom_inverse(context, pt.x, log_domain_size);
    eval!(context, (final_evaluation) * (denom_inverse))
}
