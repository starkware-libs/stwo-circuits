use num_traits::One;
use stwo::core::fields::m31::M31;

use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::{div, from_partial_evals};
use crate::eval;
use crate::stark_verifier::circle::double_x;
use crate::stark_verifier::component::Component;
use crate::stark_verifier::statement::{EvaluateArgs, Statement};

use super::simple_air::{FIB_SEQUENCE_LENGTH, LOG_N_INSTANCES};

/// Computes the polynomial that vanishes on the canonical coset of size `2^log_trace_size`.
///
/// The polynomial is `pi^{log_trace_size - 1}(x) = pi(...(pi(x))...)`.
fn coset_vanishing_poly(
    context: &mut Context<impl IValue>,
    mut x: Var,
    log_trace_size: usize,
) -> Var {
    assert!(log_trace_size >= 1);

    for _ in 0..(log_trace_size - 1) {
        x = double_x(context, x);
    }
    x
}

/// Computes the inverse of the domain polynomial at `x`. See [coset_vanishing_poly].
fn denom_inverse(context: &mut Context<impl IValue>, x: Var, log_trace_size: usize) -> Var {
    let one = context.one();
    let denom = coset_vanishing_poly(context, x, log_trace_size);
    div(context, one, denom)
}

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

#[derive(Default)]
pub struct SimpleStatement {
    pub fib_component: SquaredFibonacciComponent,
}
#[derive(Default)]
pub struct SquaredFibonacciComponent {}
impl Component for SquaredFibonacciComponent {
    fn evaluate(&self, context: &mut Context<impl IValue>, prev_sum: Var, args: &mut EvaluateArgs<'_>) -> Var {
        let EvaluateArgs {
            oods_samples,
            pt,
            log_domain_size,
            composition_polynomial_coef,
            interaction_elements,
            claimed_sums,
        } = args;
        let [const_val] = oods_samples.preprocessed_columns[..].try_into().unwrap();
        let [a, b, c, d] = oods_samples.trace.split_off(..4).unwrap() else {
            panic!("Expected 4 trace values");
        };
        let interaction = oods_samples.interaction.split_off(..4).unwrap();
        let [claimed_sum] = claimed_sums.split_off(..1).unwrap() else {
            panic!("Expected 1 claimed sum");
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
                interaction[0].at_prev,
                interaction[1].at_prev,
                interaction[2].at_prev,
                interaction[3].at_prev,
            ],
        );
        let cur_logup_sum = from_partial_evals(
            context,
            [
                interaction[0].at_oods,
                interaction[1].at_oods,
                interaction[2].at_oods,
                interaction[3].at_oods,
            ],
        );
        let n_instances = context.constant((1 << LOG_N_INSTANCES).into());
        let cumsum_shift = div(context, *claimed_sum, n_instances);
        let diff = eval!(context, (cur_logup_sum) - (prev_logup_sum));
        let shifted_diff = eval!(context, (diff) + (cumsum_shift));
        let logup_constraint_val =
            single_logup_term(context, &[*c, *d], shifted_diff, *interaction_elements);

        let denom_inverse = denom_inverse(context, pt.x, *log_domain_size);

        let constraint_val = prev_sum;
        let constraint_val = eval!(context, (constraint_val) * (*composition_polynomial_coef));
        let constraint_val = eval!(context, (constraint_val) + (constraint0_val));
        let constraint_val = eval!(context, (constraint_val) * (*composition_polynomial_coef));
        let constraint_val = eval!(context, (constraint_val) + (constraint1_val));
        let constraint_val = eval!(context, (constraint_val) * (*composition_polynomial_coef));
        let constraint_val = eval!(context, (constraint_val) + (logup_constraint_val));

        eval!(context, (constraint_val) * (denom_inverse))
    }
}

impl Statement for SimpleStatement {
    fn public_logup_sum(
        &self,
        context: &mut Context<impl IValue>,
        interaction_elements: [Var; 2],
    ) -> Var {
        let mut sum = context.zero();
        for j in 0..(1 << LOG_N_INSTANCES) {
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

    fn evaluate(&self, context: &mut Context<impl IValue>, mut args: EvaluateArgs<'_>) -> Var {
        let prev_sum = context.zero();
        let result1 = self.fib_component.evaluate(context, prev_sum, &mut args);
        let result2 = self.fib_component.evaluate(context, result1, &mut args);

        assert!(args.oods_samples.trace.is_empty());
        assert!(args.oods_samples.interaction.is_empty());
        assert!(args.claimed_sums.is_empty());
        result2
    }
}
