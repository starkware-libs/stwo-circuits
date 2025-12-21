use stwo::core::air::Component;
use stwo::core::channel::{Blake2sM31Channel, Channel};
use stwo::core::pcs::{CommitmentSchemeVerifier, PcsConfig, TreeVec};
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleChannel;
use stwo::core::verifier::verify;

use crate::examples::simple_air::{PublicInput, create_proof};
use crate::stark_verifier::proof_from_stark_proof::pack_component_log_sizes;

#[test]
fn verify_simple_proof() {
    let config = PcsConfig::default();
    let (components, PublicInput { claimed_sums, component_log_sizes }, proof) = create_proof();

    // Verify.
    let verifier_channel = &mut Blake2sM31Channel::default();
    let commitment_scheme = &mut CommitmentSchemeVerifier::<Blake2sM31MerkleChannel>::new(config);

    // Retrieve the expected column sizes in each commitment interaction, from the AIR.
    let sizes = TreeVec::concat_cols(components.iter().map(|c| c.trace_log_degree_bounds()));

    verifier_channel.mix_felts(&pack_component_log_sizes(component_log_sizes));
    commitment_scheme.commit(proof.proof.commitments[0], &sizes[0].sorted(), verifier_channel);
    commitment_scheme.commit(proof.proof.commitments[1], &sizes[1], verifier_channel);
    verifier_channel.mix_felts(&claimed_sums);
    commitment_scheme.commit(proof.proof.commitments[2], &sizes[2], verifier_channel);
    verify(
        &components.iter().map(|c| c.as_ref()).collect::<Vec<&dyn Component>>(),
        verifier_channel,
        commitment_scheme,
        proof.proof,
    )
    .unwrap();
}
