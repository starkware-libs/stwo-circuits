use std::array;

use hashbrown::HashMap;
use itertools::Itertools;
use itertools::chain;
use num_traits::Zero;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::proof::ExtendedStarkProof;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sM31MerkleHasher;

use crate::circuits::ivalue::qm31_from_u32s;
use crate::stark_verifier::fri_proof::{FriCommitProof, FriProof};
use crate::stark_verifier::merkle::{AuthPath, AuthPaths};
use crate::stark_verifier::oods::EvalDomainSamples;
use crate::stark_verifier::proof::{Claim, InteractionAtOods, N_TRACES, Proof, ProofConfig};

/// Constructs [Proof] with the values from the given proof ([ExtendedStarkProof]).
pub fn proof_from_stark_proof(
    proof: &ExtendedStarkProof<Blake2sM31MerkleHasher>,
    config: &ProofConfig,
    claim: Claim<QM31>,
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
        interaction_at_oods: sampled_values[2]
            .iter()
            .map(|x| match x[..] {
                [at_prev, at_oods] => InteractionAtOods { at_oods, at_prev: Some(at_prev) },
                [at_oods] => InteractionAtOods { at_oods, at_prev: None },
                _ => panic!("Unexpected interaction at OODS values"),
            })
            .collect_vec(),
        claim,
        composition_eval_at_oods: as_single_row(&sampled_values[3]).try_into().unwrap(),
        eval_domain_samples: construct_eval_domain_samples(proof, config),
        eval_domain_auth_paths: construct_eval_domain_auth_paths(proof, config),
        fri: FriProof {
            commit: FriCommitProof {
                layer_commitments: chain!(
                    [fri_proof.first_layer.commitment.into()],
                    fri_proof.inner_layers.iter().map(|layer| layer.commitment.into()),
                )
                .collect(),
                last_layer_coefs: (*fri_proof.last_layer_poly).to_vec(),
            },
            auth_paths: construct_fri_auth_paths(proof, config),
            fri_siblings: construct_fri_siblings(proof, config),
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

    // Map from query position to its one or more indices in
    // `extra_proof_data.unsorted_query_locations`.
    // For example, if `unsorted_query_locations = [10, 5, 10]` then
    //   `query_to_indices = { 10: [0, 2], 5: [1] }`.
    let mut query_to_indices = HashMap::<usize, Vec<usize>>::new();
    assert_eq!(unsorted_query_locations.len(), n_queries);
    for (i, query) in unsorted_query_locations.iter().enumerate() {
        query_to_indices.entry(*query).or_insert(Vec::new()).push(i);
    }

    let sorted_query_indices = query_to_indices.keys().sorted().collect_vec();

    // For each trace and column, add the sampled values from all the queries in unsorted order.
    let mut data: Vec<Vec<Vec<M31>>> = vec![vec![]; N_TRACES];
    for (trace_idx, n_columns_in_trace) in config.n_columns_per_trace().iter().enumerate() {
        for column_idx in 0..*n_columns_in_trace {
            data[trace_idx].push(vec![M31::zero(); n_queries]);
            for (query_idx, query) in sorted_query_indices.iter().enumerate() {
                for idx in &query_to_indices[*query] {
                    data[trace_idx][column_idx][*idx] =
                        queried_values[trace_idx][column_idx][query_idx];
                }
            }
        }
    }

    EvalDomainSamples::from_m31s(data)
}

/// Constructs [AuthPaths] for the evaluation domain queries (the in-domain queries) with the values
/// from the given proof ([ExtendedStarkProof]).
fn construct_eval_domain_auth_paths(
    proof: &ExtendedStarkProof<Blake2sM31MerkleHasher>,
    config: &ProofConfig,
) -> AuthPaths<QM31> {
    let unsorted_query_locations = &proof.aux.unsorted_query_locations;
    let res = proof
        .aux
        .trace_decommitment
        .iter()
        .map(|merkle_decommitment_aux| {
            unsorted_query_locations
                .iter()
                .map(|query_idx| {
                    let mut auth_path: AuthPath<QM31> = AuthPath(vec![]);
                    let mut pos = *query_idx;
                    for j in 0..config.log_evaluation_domain_size() {
                        let hash = merkle_decommitment_aux.all_node_values[j][&(pos ^ 1)];
                        auth_path.0.push(hash.into());
                        pos >>= 1;
                    }
                    auth_path
                })
                .collect()
        })
        .collect();

    AuthPaths { data: res }
}

/// Constructs [AuthPaths] for the FRI trees with the values from the given proof
/// ([ExtendedStarkProof]).
fn construct_fri_auth_paths(
    proof: &ExtendedStarkProof<Blake2sM31MerkleHasher>,
    config: &ProofConfig,
) -> AuthPaths<QM31> {
    let unsorted_query_locations = &proof.aux.unsorted_query_locations;
    let layers = chain!([&proof.aux.fri.first_layer], &proof.aux.fri.inner_layers);
    let res = layers
        .enumerate()
        .map(|(tree_idx, aux)| {
            unsorted_query_locations
                .iter()
                .map(|query| {
                    let mut pos = *query;
                    pos >>= tree_idx;
                    let mut auth_path: AuthPath<QM31> = AuthPath(vec![]);
                    for j in 0..config.log_evaluation_domain_size() - tree_idx {
                        let hash = aux.decommitment.all_node_values[j][&(pos ^ 1)];
                        if j > 0 {
                            // Don't add the first hash because it's computed from the fri sibling.
                            auth_path.0.push(hash.into());
                        }
                        pos >>= 1;
                    }
                    auth_path
                })
                .collect()
        })
        .collect();

    AuthPaths { data: res }
}

/// Constructs the vector of siblings for the FRI trees with the values from the given proof
/// ([ExtendedStarkProof]).
///
/// For each tree, for each query, the sibling of the relevant node in FRI.
fn construct_fri_siblings(
    proof: &ExtendedStarkProof<Blake2sM31MerkleHasher>,
    config: &ProofConfig,
) -> Vec<Vec<QM31>> {
    let mut res = vec![vec![]; config.log_trace_size()];
    let layers = chain!([&proof.aux.fri.first_layer], &proof.aux.fri.inner_layers).collect_vec();
    for query in &proof.aux.unsorted_query_locations {
        let mut pos = *query;
        for j in 0..config.log_trace_size() {
            let sibling = layers[j].all_values[0][&(pos ^ 1)];
            pos >>= 1;
            res[j].push(sibling);
        }
    }
    res
}

/// Packs the enable bits into QM31s.
pub fn pack_enable_bits(enable_bits: &[bool]) -> Vec<QM31> {
    pack_into_qm31s(enable_bits.iter().map(|b| if *b { 1 } else { 0 }))
}

/// Packs the component log sizes into QM31s.
/// Each QM31 holds up to 4 log sizes and the last one is padded with zeros.
pub fn pack_component_log_sizes(component_log_sizes: &[u32]) -> Vec<QM31> {
    pack_into_qm31s(component_log_sizes.iter().cloned())
}

pub fn pack_into_qm31s<T: Into<M31>>(values: impl Iterator<Item = T>) -> Vec<QM31> {
    values
        .chunks(4)
        .into_iter()
        .map(|mut chunk| {
            QM31::from_m31_array(array::from_fn(|_| {
                chunk.next().map(|v| v.into()).unwrap_or(M31::zero())
            }))
        })
        .collect_vec()
}
