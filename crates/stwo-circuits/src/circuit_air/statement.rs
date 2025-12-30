use crate::circuit_air::components::{eq, qm31_ops};
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::div;
use crate::eval;
use crate::stark_verifier::circle::denom_inverse;
use crate::stark_verifier::component::CompositionConstraintAccumulator;
use crate::stark_verifier::statement::{EvaluateArgs, Statement};

pub struct CircuitStatement {
    pub qm31_ops_statement: qm31_ops::Eval,
    pub eq_statement: eq::Eval,
}

impl Default for CircuitStatement {
    fn default() -> Self {
        Self { qm31_ops_statement: qm31_ops::Eval::default(), eq_statement: eq::Eval::default() }
    }
}
impl Statement for CircuitStatement {
    fn evaluate(&self, context: &mut Context<impl IValue>, args: EvaluateArgs<'_>) -> Var {
        let EvaluateArgs {
            oods_samples,
            pt,
            log_domain_size,
            composition_polynomial_coeff,
            interaction_elements,
            claimed_sums,
        } = args;

        let mut evaluation_accumulator = CompositionConstraintAccumulator {
            oods_samples,
            composition_polynomial_coeff,
            interaction_elements,
            claimed_sums,
            accumulation: context.zero(),
        };

        self.qm31_ops_statement.evaluate(context, &mut evaluation_accumulator);
        self.eq_statement.evaluate(context, &mut evaluation_accumulator);

        let final_evaluation = evaluation_accumulator.finalize();

        let denom_inverse = denom_inverse(context, pt.x, log_domain_size);
        eval!(context, (final_evaluation) * (denom_inverse))
    }

    fn public_logup_sum(
        &self,
        context: &mut Context<impl IValue>,
        interaction_elements: [Var; 2],
    ) -> Var {
        let mut sum = context.zero();
        sum = self.qm31_ops_statement.public_logup_sum(context, sum, interaction_elements);
        sum = self.eq_statement.public_logup_sum(context, sum, interaction_elements);
        sum
    }
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
pub fn single_logup_term(
    context: &mut Context<impl IValue>,
    frac: Frac,
    shifted_diff: Var,
) -> Var {
    eval!(context, ((shifted_diff) * (frac.denominator)) - (frac.numerator))
}

/// Computes the constraint polynomial for a pair logup term.
pub fn pair_logup_term(
    context: &mut Context<impl IValue>,
    frac0: Frac,
    frac1: Frac,
    shifted_diff: Var,
) -> Var {
    let denominator = eval!(context, (frac0.denominator) * (frac1.denominator));
    let numerator = eval!(context, ((frac1.numerator) * (frac0.denominator)) + ((frac0.numerator) * (frac1.denominator)));

    eval!(context, ((shifted_diff) * (denominator)) - (numerator))
}

pub fn get_frac(
    context: &mut Context<impl IValue>,
    interaction_elements: [Var; 2],
    numerator: Var,
    element: &[Var],
) -> Frac {
    let denominator = combine_term(context, element, interaction_elements);
    Frac { numerator, denominator }
}

struct Frac {
    pub numerator: Var,
    pub denominator: Var,
}