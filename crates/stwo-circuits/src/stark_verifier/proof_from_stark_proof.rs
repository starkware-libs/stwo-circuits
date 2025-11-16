use crate::circuits::ivalue::qm31_from_u32s;
use crate::stark_verifier::fri_proof::FriCommitProof;
use crate::stark_verifier::proof::InteractionAtOods;
use crate::stark_verifier::proof::Proof;
use crate::stark_verifier::proof::ProofConfig;
use itertools::Itertools;
use itertools::chain;
use stwo::core::fields::qm31::QM31;
use stwo::core::proof::ExtendedStarkProof;
use stwo::core::vcs::blake2_merkle::Blake2sM31MerkleHasher;

pub fn proof_from_stark_proof(
    proof: &ExtendedStarkProof<Blake2sM31MerkleHasher>,
    _config: &ProofConfig,
) -> Proof<QM31> {
    let commitments = &proof.proof.commitments;
    let sampled_values = &proof.proof.sampled_values;
    let fri_proof = &proof.proof.fri_proof;

    let pow_high = (proof.proof.proof_of_work >> 32) as u32;
    let pow_low = (proof.proof.proof_of_work & 0xFFFFFFFF) as u32;

    Proof {
        preprocessed_root: commitments[0].into(),
        trace_root: commitments[1].into(),
        interaction_root: commitments[2].into(),
        composition_polynomial_root: commitments[3].into(),
        preprocessed_columns_at_oods: as_single_row(&sampled_values[0]),
        trace_at_oods: as_single_row(&sampled_values[1]),
        interaction_at_oods: InteractionAtOods {
            value: sampled_values[2].iter().map(|x| (x[1], x[0])).collect_vec(),
        },
        composition_eval_at_oods: as_single_row(&sampled_values[3]).try_into().unwrap(),
        fri: FriCommitProof {
            layer_commitments: chain!(
                [fri_proof.first_layer.commitment.into()],
                fri_proof.inner_layers.iter().map(|layer| layer.commitment.into()),
            )
            .collect(),
            last_layer_coefs: (*fri_proof.last_layer_poly).to_vec(),
        },
        proof_of_work_nonce: qm31_from_u32s(pow_low, pow_high, 0, 0),
    }
}

/// Converts a 2D vector of singletons to a 1D vector.
fn as_single_row(values: &[Vec<QM31>]) -> Vec<QM31> {
    values
        .iter()
        .map(|x| {
            let [x] = x[..].try_into().unwrap();
            x
        })
        .collect_vec()
}
