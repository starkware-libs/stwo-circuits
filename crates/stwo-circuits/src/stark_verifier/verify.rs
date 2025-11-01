use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;

use crate::stark_verifier::channel::Channel;
use crate::stark_verifier::proof::{Proof, ProofConfig};

#[cfg(test)]
#[path = "verify_test.rs"]
pub mod test;

pub fn verify(context: &mut Context<impl IValue>, proof: &Proof<Var>, _config: &ProofConfig) {
    let mut channel = Channel::new(context);

    // Mix the trace commitments into the channel.
    channel.mix_commitment(context, proof.preprocessed_root);
    channel.mix_commitment(context, proof.trace_root);

    // TODO(lior): Complete the verification.
}
