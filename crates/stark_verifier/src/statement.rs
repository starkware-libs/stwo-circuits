use std::collections::HashMap;

use circuits::blake::HashValue;
use stwo::core::circle::CirclePoint;
use stwo_constraint_framework::preprocessed_columns::PreProcessedColumnId;

use crate::constraint_eval::CircuitEval;
use crate::proof::InteractionAtOods;
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
    pub claimed_sums: &'a [Var],
    pub component_sizes: &'a [Var],
    pub n_instances_bits: &'a [Simd],
}

/// Represents an AIR and its public inputs.
pub trait Statement<Value: IValue> {
    /// Returns the statement's public inputs to mix into the channel (one
    /// `mix_qm31s` call per inner `Vec`). Only statement-specific data belongs here,
    /// e.g. program bytecode, public memory, circuit outputs. Per-component metadata
    /// (enable bits, log sizes, claimed sums) is already mixed via `proof.claim` before
    /// this is called and should not be included.
    fn claims_to_mix(&self, context: &mut Context<Value>) -> Vec<Vec<Var>>;

    /// Returns the AIR components that define the constraint system.
    fn get_components(&self) -> &[Box<dyn CircuitEval<Value>>];

    /// Returns the IDs of the preprocessed columns used by this statement's components.
    fn get_preprocessed_column_ids(&self) -> Vec<PreProcessedColumnId>;

    /// Returns the expected preprocessed trace root as circuit variables.
    fn get_preprocessed_root(&self, context: &mut Context<Value>) -> HashValue<Var>;

    /// Returns the part of the logup sum determined by the public statement.
    fn public_logup_sum(&self, context: &mut Context<Value>, interaction_elements: [Var; 2])
    -> Var;

    /// Returns statement-specific named parameters passed to component constraint evaluators.
    fn public_params(&self, _context: &mut Context<Value>) -> HashMap<String, Var> {
        HashMap::new()
    }

    /// Performs statement-level consistency checks on the claim.
    fn verify_claim(
        &self,
        _context: &mut Context<Value>,
        _component_sizes: &[Var],
        _shifted_relation_uses: &HashMap<&'static str, Var>,
        _enable_bits: &[bool],
    ) {
    }
}
