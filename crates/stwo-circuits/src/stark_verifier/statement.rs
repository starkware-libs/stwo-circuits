use stwo::core::circle::CirclePoint;

use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::stark_verifier::proof::InteractionAtOods;

/// Represents an AIR and its public inputs.
pub trait Statement {
    /// Evaluates the composition polynomial at the OODS point (after dividing by the domain
    /// polynomial).
    fn evaluate(
        &self,
        context: &mut Context<impl IValue>,
        preprocessed_columns: &Vec<Var>,
        trace_at_oods: &Vec<Var>,
        interaction_at_oods: &InteractionAtOods<Var>,
        pt: CirclePoint<Var>,
        log_domain_size: usize,
        composition_polynomial_coef: Var,
        interaction_elements: [Var; 2],
    ) -> Var;
}
