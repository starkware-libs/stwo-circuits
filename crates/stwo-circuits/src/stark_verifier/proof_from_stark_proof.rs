use crate::stark_verifier::proof::Proof;
use crate::stark_verifier::proof::ProofConfig;
use stwo::core::fields::qm31::QM31;
use stwo::core::proof::ExtendedStarkProof;
use stwo::core::vcs::blake2_merkle::Blake2sM31MerkleHasher;

pub fn proof_from_stark_proof(
    proof: &ExtendedStarkProof<Blake2sM31MerkleHasher>,
    _config: &ProofConfig,
) -> Proof<QM31> {
    let commitments = &proof.proof.commitments;
    Proof {
        preprocessed_root: commitments[0].into(),
        trace_root: commitments[1].into(),
        interaction_root: commitments[2].into(),
        composition_polynomial_root: commitments[3].into(),
    }
}
