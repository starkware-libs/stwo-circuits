use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::stark_verifier::channel::Channel;
use crate::stark_verifier::oods::extract_expected_composition_eval;
use crate::stark_verifier::proof::{Proof, ProofConfig};

#[cfg(test)]
#[path = "verify_test.rs"]
pub mod test;

pub fn verify(context: &mut Context<impl IValue>, proof: &Proof<Var>, config: &ProofConfig) {
    let mut channel = Channel::new(context);

    // Mix the trace commitments into the channel.
    channel.mix_commitment(context, proof.preprocessed_root);
    channel.mix_commitment(context, proof.trace_root);

    // TODO(lior): Add proof of work before drawing the interaction elements.

    // Pick the interaction elements.
    let [_interaction_z, _interaction_alpha] = channel.draw_two_qm31s(context);

    channel.mix_commitment(context, proof.interaction_root);

    // Draw a random QM31 coefficient for the composition polynomial.
    let _composition_polynomial_coef = channel.draw_qm31(context);

    channel.mix_commitment(context, proof.composition_polynomial_root);

    // Draw a random point for the OODS.
    let oods_point = channel.draw_point(context);

    let _expected_composition_eval = extract_expected_composition_eval(
        context,
        &proof.composition_eval_at_oods,
        oods_point,
        config.log_evaluation_domain_size(),
    );

    // TODO(lior): Complete the verification.
}
