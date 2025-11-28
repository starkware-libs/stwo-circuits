use hashbrown::HashMap;
use itertools::Itertools;
use itertools::chain;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::proof::ExtendedStarkProof;
use stwo::core::vcs::blake2_merkle::Blake2sM31MerkleHasher;

use crate::circuits::ivalue::qm31_from_u32s;
use crate::stark_verifier::fri_proof::FriCommitProof;
use crate::stark_verifier::oods::EvalDomainSamples;
use crate::stark_verifier::proof::InteractionAtOods;
use crate::stark_verifier::proof::Proof;
use crate::stark_verifier::proof::ProofConfig;

/// Constructs [Proof] with the values from the given proof ([ExtendedStarkProof]).
pub fn proof_from_stark_proof(
    proof: &ExtendedStarkProof<Blake2sM31MerkleHasher>,
    config: &ProofConfig,
) -> Proof<QM31> {
    let commitments = &proof.proof.commitments;
    let sampled_values = &proof.proof.sampled_values;
    let fri_proof = &proof.proof.fri_proof;

    let pow: u64 = proof.proof.proof_of_work;
    let pow_high = (pow >> 32) as u32;
    let pow_low = (pow & 0xFFFFFFFF) as u32;

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
        eval_domain_samples: construct_eval_domain_samples(proof, config),
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

/// Constructs [EvalDomainSamples] with the values from the given proof ([ExtendedStarkProof]).
fn construct_eval_domain_samples(
    proof: &ExtendedStarkProof<Blake2sM31MerkleHasher>,
    config: &ProofConfig,
) -> EvalDomainSamples<QM31> {
    let unsorted_query_locations = &proof.aux.unsorted_query_locations;
    let queried_values = &proof.proof.queried_values;

    let n_queries = config.n_queries();
    let mut data: Vec<Vec<Vec<M31>>> = vec![vec![vec![]; n_queries]; 4];

    // Map from query position to its one or more indices in
    // `extra_proof_data.unsorted_query_locations`.
    // For example, if `unsorted_query_locations = [10, 5, 10]` then
    //   `query_to_indices = { 10: [0, 2], 5: [1] }`.
    let mut query_to_indices = HashMap::<usize, Vec<usize>>::new();
    assert_eq!(unsorted_query_locations.len(), n_queries);
    for (i, query) in unsorted_query_locations.iter().enumerate() {
        query_to_indices.entry(*query).or_insert(Vec::new()).push(i);
    }

    // For each trace and query, add the sampled values from all the columns to `data`.
    for (trace_idx, n_columns_in_trace) in config.n_columns_per_trace().iter().enumerate() {
        for (j, query) in query_to_indices.keys().sorted().enumerate() {
            for idx in &query_to_indices[query] {
                let slice = j * n_columns_in_trace..(j + 1) * n_columns_in_trace;
                data[trace_idx][*idx].extend(queried_values[trace_idx][slice].iter());
            }
        }
    }

    EvalDomainSamples::from_m31s(data)
}
