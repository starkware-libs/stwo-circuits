use num_traits::One;
use stwo::core::fields::m31::M31;

use super::simple_air::FIB_SEQUENCE_LENGTH;
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::div;
use crate::eval;
use crate::examples::simple_air::{LOG_SIZE_LONG, LOG_SIZE_SHORT};
use crate::stark_verifier::circle::denom_inverse;
use crate::stark_verifier::constraint_eval::{CircuitEval, CompositionConstraintAccumulator};
use crate::stark_verifier::logup::combine_term;
use crate::stark_verifier::statement::{EvaluateArgs, Statement};

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
        let [const_val] = acc.get_preprocessed_columns::<1>([self.preprocessed_column_idx]);
        let [a, b, c, d] = acc.get_trace::<4>();

        // Constraints.
        let constraint0_val = eval!(context, (c) - ((((a) * (a)) + ((b) * (b))) + (const_val)));
        acc.add_constraint(context, constraint0_val);

        let constraint1_val = eval!(context, (d) - ((((b) * (b)) + ((c) * (c))) + (const_val)));
        acc.add_constraint(context, constraint1_val);

        // Logup constraint.
        acc.add_to_relation(context, context.one(), &[c, d]);
        acc.add_to_relation(context, context.one(), &[c, d]);
        acc.add_to_relation(context, context.one(), &[c, d]);
        acc.finalize_logup_in_pairs(context);
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
            let denom1 = combine_term(context, &elements, interaction_elements);

            let denom = eval!(context, (denom1) * (denom1));
            let numerator = eval!(context, (denom1) + (denom1));

            let frac0 = div(context, numerator, denom);
            let frac1 = div(context, context.one(), denom1);
            let frac = eval!(context, (frac0) + (frac1));

            // Note that the sum is negated because we want to use the values that are yielded in
            // the witness.
            sum = eval!(context, (sum) - (frac));
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
        let interaction_column_log_sizes = vec![
            size_0, size_0, size_0, size_0, size_0, size_0, size_0, size_0, size_1, size_1, size_1,
            size_1, size_1, size_1, size_1, size_1,
        ];

        [trace_column_log_sizes, interaction_column_log_sizes]
    }
}
