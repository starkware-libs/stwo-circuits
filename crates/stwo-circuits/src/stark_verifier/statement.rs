use stwo::core::circle::CirclePoint;

use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::stark_verifier::proof::InteractionAtOods;

/// Values at the OODS point (and its previous point where applicable).
pub struct OodsSamples<'a> {
    pub preprocessed_columns: &'a [Var],
    pub trace: &'a [Var],
    pub interaction: &'a InteractionAtOods<Var>,
}

pub struct EvaluateArgs<'a> {
    pub oods_samples: OodsSamples<'a>,
    pub pt: CirclePoint<Var>,
    pub log_domain_size: usize,
    pub composition_polynomial_coef: Var,
    pub interaction_elements: [Var; 2],
}

/// Represents an AIR and its public inputs.
pub trait Statement {
    /// Evaluates the composition polynomial at the OODS point (after dividing by the domain
    /// polynomial).
    fn evaluate(&self, context: &mut Context<impl IValue>, args: EvaluateArgs<'_>) -> Var;
}
