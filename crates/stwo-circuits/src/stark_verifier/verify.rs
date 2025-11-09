use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::eq;
use crate::stark_verifier::channel::Channel;
use crate::stark_verifier::oods::extract_expected_composition_eval;
use crate::stark_verifier::proof::{Proof, ProofConfig};
use crate::stark_verifier::statement::{OodsSamples, Statement};

#[cfg(test)]
#[path = "verify_test.rs"]
pub mod test;

pub fn verify(
    context: &mut Context<impl IValue>,
    proof: &Proof<Var>,
    config: &ProofConfig,
    statement: &impl Statement,
) {
    let mut channel = Channel::new(context);

    // Mix the trace commitments into the channel.
    channel.mix_commitment(context, proof.preprocessed_root);
    channel.mix_commitment(context, proof.trace_root);

    // TODO(lior): Add proof of work before drawing the interaction elements.

    // Pick the interaction elements.
    let [interaction_z, interaction_alpha] = channel.draw_two_qm31s(context);

    channel.mix_commitment(context, proof.interaction_root);

    // Draw a random QM31 coefficient for the composition polynomial.
    let composition_polynomial_coef = channel.draw_qm31(context);

    channel.mix_commitment(context, proof.composition_polynomial_root);

    // Draw a random point for the OODS.
    let oods_point = channel.draw_point(context);

    // Compute the composition evaluation at the OODS point from `proof.*_at_oods` and compare
    // to `proof.composition_eval_at_oods`.
    let composition_eval = statement.evaluate(
        context,
        OodsSamples {
            preprocessed_columns: &proof.preprocessed_columns_at_oods,
            trace: &proof.trace_at_oods,
            interaction: &proof.interaction_at_oods,
        },
        oods_point,
        config.log_trace_size(),
        composition_polynomial_coef,
        [interaction_z, interaction_alpha],
    );
    let expected_composition_eval = extract_expected_composition_eval(
        context,
        &proof.composition_eval_at_oods,
        oods_point,
        config.log_evaluation_domain_size(),
    );
    eq(context, composition_eval, expected_composition_eval);

    // TODO(lior): Complete the verification.
}
