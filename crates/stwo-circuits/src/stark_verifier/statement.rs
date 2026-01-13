use stwo::core::circle::CirclePoint;

use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::simd::Simd;
use crate::stark_verifier::constraint_eval::{CircuitEval, ComponentData};
use crate::stark_verifier::proof::InteractionAtOods;
use crate::stark_verifier::verify::MAX_TRACE_SIZE_BITS;

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
    pub component_data: &'a [ComponentData],
    pub n_instances_bits: &'a [Simd; MAX_TRACE_SIZE_BITS],
}

/// Represents an AIR and its public inputs.
pub trait Statement<Value: IValue> {
    /// Returns the components of the statement.
    fn get_components(&self) -> &[Box<dyn CircuitEval<Value>>];

    /// Computes the part of the logup sum that is determined by the (public) statement rather than
    /// by the witness.
    fn public_logup_sum(&self, context: &mut Context<Value>, interaction_elements: [Var; 2])
    -> Var;
}
