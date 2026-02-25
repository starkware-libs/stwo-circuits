use std::collections::HashMap;

use stwo::core::circle::CirclePoint;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::constraint_eval::CircuitEval;
use crate::proof::{Claim, InteractionAtOods};
use circuits::context::{Context, Var};
use circuits::ivalue::IValue;
use circuits::simd::Simd;

/// Values at the OODS point (and its previous point where applicable).
pub struct OodsSamples<'a> {
    pub preprocessed_columns: &'a [Var],
    pub trace: &'a [Var],
    pub interaction: &'a [InteractionAtOods<Var>],
}

pub struct EvaluateArgs<'a> {
    pub oods_samples: OodsSamples<'a>,
    pub pt: CirclePoint<Var>,
    pub log_domain_size: usize,
    pub composition_polynomial_coeff: Var,
    pub interaction_elements: [Var; 2],
    pub enable_bits: &'a [Var],
    pub claimed_sums: &'a [Var],
    pub component_sizes: &'a [Var],
    pub n_instances_bits: &'a [Simd],
}

/// Represents an AIR and its public inputs.
pub trait Statement<Value: IValue> {
    fn claims_to_mix(&self, context: &mut Context<Value>) -> Vec<Vec<Var>>;

    /// Returns the components of the statement.
    fn get_components(&self) -> &[Box<dyn CircuitEval<Value>>];

    fn get_preprocessed_column_ids(&self) -> Vec<PreProcessedColumnId>;

    /// Computes the part of the logup sum that is determined by the (public) statement rather than
    /// by the witness.
    fn public_logup_sum(
        &self,
        context: &mut Context<Value>,
        interaction_elements: [Var; 2],
        claim: &Claim<Var>,
    ) -> Var;

    fn public_params(&self, _context: &mut Context<Value>) -> HashMap<String, Var> {
        HashMap::new()
    }

    fn verify_claim(
        &self,
        _context: &mut Context<Value>,
        _enable_bits: &[Var],
        _component_sizes: &[Var],
        _shifted_relation_uses: &HashMap<&'static str, Var>,
    ) {
    }
}
