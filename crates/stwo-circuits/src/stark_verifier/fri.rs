use crate::circuits::circuit::Var;
use crate::circuits::context::Context;
use crate::circuits::ivalue::IValue;
use crate::stark_verifier::channel::Channel;
use crate::stark_verifier::fri_proof::FriCommitProof;

#[cfg(test)]
#[path = "fri_test.rs"]
pub mod test;

// Commits to the FRI layers and returns the random alphas.
pub fn fri_commit(
    context: &mut Context<impl IValue>,
    channel: &mut Channel,
    proof: &FriCommitProof<Var>,
) -> Vec<Var> {
    let mut alphas = Vec::new();
    for root in &proof.layer_commitments {
        channel.mix_commitment(context, *root);
        alphas.push(channel.draw_qm31(context));
    }
    channel.mix_qm31s(context, proof.last_layer_coefs.iter().cloned());

    alphas
}
