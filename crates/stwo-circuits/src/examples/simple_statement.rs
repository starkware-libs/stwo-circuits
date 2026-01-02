use num_traits::One;
use stwo::core::fields::m31::M31;

use super::simple_air::FIB_SEQUENCE_LENGTH;
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::{div, from_partial_evals};
use crate::eval;
use crate::examples::simple_air::{LOG_SIZE_LONG, LOG_SIZE_SHORT};
use crate::stark_verifier::circle::denom_inverse;
use crate::stark_verifier::component::{CircuitEval, CompositionConstraintAccumulator};
use crate::stark_verifier::statement::{EvaluateArgs, Statement};

/// Computes the denominator of a logup term.
fn combine_term(
    context: &mut Context<impl IValue>,
    element: &[Var],
    interaction_elements: [Var; 2],
) -> Var {
    let mut element_iter = element.iter().rev();
    let mut value = *element_iter.next().unwrap();
    for elm in element_iter {
        value = eval!(context, (value) * (interaction_elements[1]));
        value = eval!(context, (value) + (*elm));
    }
    eval!(context, (value) - (interaction_elements[0]))
}

/// Computes the constraint polynomial for a single logup term.
fn single_logup_term(
    context: &mut Context<impl IValue>,
    element: &[Var],
    shifted_diff: Var,
    interaction_elements: [Var; 2],
) -> Var {
    let denominator = combine_term(context, element, interaction_elements);
    eval!(context, ((shifted_diff) * (denominator)) - (1))
}

pub struct SimpleStatement {
    pub long_fib_component: SquaredFibonacciComponent,
    pub short_fib_component: SquaredFibonacciComponent,
}

impl Default for SimpleStatement {
    fn default() -> Self {
        Self {
            long_fib_component: SquaredFibonacciComponent {
                log_n_instances: LOG_SIZE_LONG,
                preprocessed_column_idx: 1,
            },
            short_fib_component: SquaredFibonacciComponent {
                log_n_instances: LOG_SIZE_SHORT,
                preprocessed_column_idx: 0,
            },
        }
    }
}

pub struct SquaredFibonacciComponent {
    // TODO(ilya): Take this from the proof instead of the component.
    pub log_n_instances: u32,
    pub preprocessed_column_idx: usize,
}
impl CircuitEval for SquaredFibonacciComponent {
    fn evaluate(
        &self,
        context: &mut Context<impl IValue>,
        acc: &mut CompositionConstraintAccumulator<'_>,
    ) {
        let const_val = acc.oods_samples.preprocessed_columns[self.preprocessed_column_idx];
        let Some([a, b, c, d]) = acc.oods_samples.trace.split_off(..4) else {
            panic!("Expected 4 trace values");
        };
        let Some([interaction0, interaction1, interaction2, interaction3]) =
            acc.oods_samples.interaction.split_off(..4)
        else {
            panic!("Expected 4 interaction values");
        };
        let Some([claimed_sum]) = acc.claimed_sums.split_off(..1) else {
            panic!("Expected 1 claimed sum");
        };
        let Some([n_instances]) = acc.component_sizes.split_off(..1) else {
            panic!("Expected 1 component size");
        };

        // Constraints.
        let constraint0_val =
            eval!(context, (*c) - ((((*a) * (*a)) + ((*b) * (*b))) + (const_val)));
        let constraint1_val =
            eval!(context, (*d) - ((((*b) * (*b)) + ((*c) * (*c))) + (const_val)));

        // Logup constraint.
        let prev_logup_sum = from_partial_evals(
            context,
            [
                interaction0.at_prev,
                interaction1.at_prev,
                interaction2.at_prev,
                interaction3.at_prev,
            ],
        );
        let cur_logup_sum = from_partial_evals(
            context,
            [
                interaction0.at_oods,
                interaction1.at_oods,
                interaction2.at_oods,
                interaction3.at_oods,
            ],
        );
        let cumsum_shift = div(context, *claimed_sum, *n_instances);
        let diff = eval!(context, (cur_logup_sum) - (prev_logup_sum));
        let shifted_diff = eval!(context, (diff) + (cumsum_shift));
        let logup_constraint_val =
            single_logup_term(context, &[*c, *d], shifted_diff, acc.interaction_elements);

        acc.accumulate(context, constraint0_val);
        acc.accumulate(context, constraint1_val);
        acc.accumulate(context, logup_constraint_val);
    }
}

impl SquaredFibonacciComponent {
    fn public_logup_sum(
        &self,
        context: &mut Context<impl IValue>,
        prev_sum: Var,
        interaction_elements: [Var; 2],
    ) -> Var {
        let mut sum = prev_sum;
        for j in 0..(1 << self.log_n_instances) {
            let mut a: M31 = M31::one();
            let mut b: M31 = j.into();
            for _ in 0..(FIB_SEQUENCE_LENGTH - 2) {
                (a, b) = (b, a * a + b * b + M31::from(j));
            }
            let elements = [context.constant(a.into()), context.constant(b.into())];
            let denom = combine_term(context, &elements, interaction_elements);

            let inv = div(context, context.one(), denom);

            // Note that the sum is negated because we want to use the values that are yielded in
            // the witness.
            sum = eval!(context, (sum) - (inv));
        }
        sum
    }
}

impl Statement for SimpleStatement {
    fn public_logup_sum(
        &self,
        context: &mut Context<impl IValue>,
        interaction_elements: [Var; 2],
    ) -> Var {
        let prev_sum = context.zero();
        let prev_sum =
            self.long_fib_component.public_logup_sum(context, prev_sum, interaction_elements);
        self.short_fib_component.public_logup_sum(context, prev_sum, interaction_elements)
    }

    fn evaluate(&self, context: &mut Context<impl IValue>, args: EvaluateArgs<'_>) -> Var {
        let EvaluateArgs {
            oods_samples,
            pt,
            log_domain_size,
            composition_polynomial_coeff,
            interaction_elements,
            claimed_sums,
            component_sizes,
        } = args;

        let mut evaluation_accumulator = CompositionConstraintAccumulator {
            oods_samples,
            composition_polynomial_coeff,
            interaction_elements,
            claimed_sums,
            component_sizes,
            accumulation: context.zero(),
        };

        self.long_fib_component.evaluate(context, &mut evaluation_accumulator);
        self.short_fib_component.evaluate(context, &mut evaluation_accumulator);
        let final_evaluation = evaluation_accumulator.finalize();

        let denom_inverse = denom_inverse(context, pt.x, log_domain_size);
        eval!(context, (final_evaluation) * (denom_inverse))
    }

    fn column_log_sizes(&self, component_log_sizes: Vec<Var>) -> [Vec<Var>; 2] {
        let [size_0, size_1] = component_log_sizes[..] else {
            panic!("Expected 2 component log sizes");
        };

        let trace_column_log_sizes =
            vec![size_0, size_0, size_0, size_0, size_1, size_1, size_1, size_1];
        let interaction_column_log_sizes =
            vec![size_0, size_0, size_0, size_0, size_1, size_1, size_1, size_1];

        [trace_column_log_sizes, interaction_column_log_sizes]
    }
}
