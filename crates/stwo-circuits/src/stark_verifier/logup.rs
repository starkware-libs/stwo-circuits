use crate::circuits::context::Context;
use crate::circuits::context::Var;
use crate::circuits::ivalue::IValue;
use crate::eval;
use stwo::core::Fraction;

pub type LogupTerm = Fraction<Var, Var>;

/// Computes the logup term for a single element.
pub fn logup_term(
    context: &mut Context<impl IValue>,
    interaction_elements: [Var; 2],
    numerator: Var,
    element: &[Var],
) -> LogupTerm {
    let denominator = combine_term(context, element, interaction_elements);
    LogupTerm::new(numerator, denominator)
}

/// Computes the denominator of a logup term.
pub fn combine_term(
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
pub fn single_logup_constraint(
    context: &mut Context<impl IValue>,
    element: &[Var],
    shifted_diff: Var,
    interaction_elements: [Var; 2],
) -> Var {
    let denominator = combine_term(context, element, interaction_elements);
    eval!(context, ((shifted_diff) * (denominator)) - (1))
}
