use crate::circuits::context::Context;
use crate::circuits::context::Var;
use crate::circuits::ivalue::IValue;
use crate::eval;

#[derive(Clone)]
pub struct Frac {
    pub numerator: Var,
    pub denominator: Var,
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
pub fn single_logup_term(context: &mut Context<impl IValue>, frac: Frac, shifted_diff: Var) -> Var {
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
    let numerator = eval!(
        context,
        ((frac1.numerator) * (frac0.denominator)) + ((frac0.numerator) * (frac1.denominator))
    );

    eval!(context, ((shifted_diff) * (denominator)) - (numerator))
}
