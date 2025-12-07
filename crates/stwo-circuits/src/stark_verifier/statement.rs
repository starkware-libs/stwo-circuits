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

/// Represents an AIR and its public inputs.
pub trait Statement {
    /// Evaluates the composition polynomial at the OODS point (after dividing by the domain
    /// polynomial).
    // TODO(ilya): Pack the arguments in a struct.
    #[allow(clippy::too_many_arguments)]
    fn evaluate(
        &self,
        context: &mut Context<impl IValue>,
        oods_samples: OodsSamples<'_>,
        pt: CirclePoint<Var>,
        log_domain_size: usize,
        composition_polynomial_coef: Var,
        interaction_elements: [Var; 2],
        claimed_sums: &[Var],
    ) -> Var;

    /// Computes the logup sum that is determined by the statement rather than by the witness.
    fn public_logup_sum(
        &self,
        context: &mut Context<impl IValue>,
        interaction_elements: [Var; 2],
    ) -> Var;
}
