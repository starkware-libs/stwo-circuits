use stwo::core::air::Component;
use stwo::core::channel::{Blake2sM31Channel, Channel};
use stwo::core::pcs::{CommitmentSchemeVerifier, PcsConfig};
use stwo::core::vcs::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::core::verifier::verify;

use crate::examples::simple_air::create_proof;

#[test]
fn verify_simple_proof() {
    let config = PcsConfig::default();
    let (component, proof) = create_proof();

    // Verify.
    let verifier_channel = &mut Blake2sM31Channel::default();
    let commitment_scheme = &mut CommitmentSchemeVerifier::<Blake2sM31MerkleChannel>::new(config);

    // Retrieve the expected column sizes in each commitment interaction, from the AIR.
    let sizes = component.trace_log_degree_bounds();
    commitment_scheme.commit(proof.proof.commitments[0], &sizes[0], verifier_channel);
    commitment_scheme.commit(proof.proof.commitments[1], &sizes[1], verifier_channel);
    verifier_channel.mix_felts(&[component.claimed_sum()]);
    commitment_scheme.commit(proof.proof.commitments[2], &sizes[2], verifier_channel);
    verify(&[&component], verifier_channel, commitment_scheme, proof.proof).unwrap();
}
