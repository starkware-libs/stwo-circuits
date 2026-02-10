use std::array;

use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use crate::circuits::blake::HashValue;
use crate::circuits::wrappers::M31Wrapper;
use crate::stark_verifier::fri_proof::{FriCommitProof, FriConfig, FriProof};
use crate::stark_verifier::merkle::{AuthPath, AuthPaths};
use crate::stark_verifier::oods::{EvalDomainSamples, N_COMPOSITION_COLUMNS};
use crate::stark_verifier::proof::{Claim, InteractionAtOods, Proof, ProofConfig};

/// Deserializes types from a format serialized by corresponding `CircuitSerialize` implementations.
pub trait CircuitDeserialize: Sized {
    fn deserialize(data: &mut &[u8]) -> Self;
}

impl CircuitDeserialize for M31 {
    fn deserialize(data: &mut &[u8]) -> Self {
        let bytes: [u8; 4] = data[..4].try_into().unwrap();
        *data = &data[4..];
        M31::from(u32::from_le_bytes(bytes))
    }
}

impl CircuitDeserialize for QM31 {
    fn deserialize(data: &mut &[u8]) -> Self {
        let m31_values: [M31; 4] = array::from_fn(|_| M31::deserialize(data));
        QM31::from_m31_array(m31_values)
    }
}

impl<T: CircuitDeserialize, const N: usize> CircuitDeserialize for [T; N] {
    fn deserialize(data: &mut &[u8]) -> Self {
        array::from_fn(|_| T::deserialize(data))
    }
}

impl CircuitDeserialize for HashValue<QM31> {
    fn deserialize(data: &mut &[u8]) -> Self {
        HashValue(QM31::deserialize(data), QM31::deserialize(data))
    }
}

impl CircuitDeserialize for M31Wrapper<QM31> {
    fn deserialize(data: &mut &[u8]) -> Self {
        let m31 = M31::deserialize(data);
        M31Wrapper::from(m31)
    }
}

/// Deserializes a `Vec<T>` of the given length from the byte stream.
fn deserialize_vec<T: CircuitDeserialize>(data: &mut &[u8], len: usize) -> Vec<T> {
    (0..len).map(|_| T::deserialize(data)).collect()
}

impl Proof<QM31> {
    /// Deserializes a proof from a byte stream, using the [`ProofConfig`] for all length
    /// information.
    pub fn deserialize_with_config(data: &mut &[u8], config: &ProofConfig) -> Self {
        let channel_salt = QM31::deserialize(data);
        let preprocessed_root = HashValue::<QM31>::deserialize(data);
        let trace_root = HashValue::<QM31>::deserialize(data);
        let interaction_root = HashValue::<QM31>::deserialize(data);
        let composition_polynomial_root = HashValue::<QM31>::deserialize(data);
        let preprocessed_columns_at_oods = deserialize_vec(data, config.n_preprocessed_columns);
        let trace_at_oods = deserialize_vec(data, config.n_trace_columns);
        let composition_eval_at_oods = <[QM31; N_COMPOSITION_COLUMNS]>::deserialize(data);
        let claim = deserialize_claim(data, config);
        let interaction_at_oods = deserialize_interaction_at_oods(data, config);
        let eval_domain_samples = deserialize_eval_domain_samples(data, config);
        let eval_domain_auth_paths = deserialize_eval_domain_auth_paths(data, config);
        let proof_of_work_nonce = QM31::deserialize(data);
        let interaction_pow_nonce = QM31::deserialize(data);
        let fri = deserialize_fri_proof(data, &config.fri);

        Proof {
            channel_salt,
            preprocessed_root,
            trace_root,
            interaction_root,
            composition_polynomial_root,
            preprocessed_columns_at_oods,
            trace_at_oods,
            composition_eval_at_oods,
            claim,
            interaction_at_oods,
            eval_domain_samples,
            eval_domain_auth_paths,
            proof_of_work_nonce,
            interaction_pow_nonce,
            fri,
        }
    }
}

fn deserialize_claim(data: &mut &[u8], config: &ProofConfig) -> Claim<QM31> {
    let packed_enable_bits = deserialize_vec(data, config.n_components.div_ceil(4));
    let packed_component_log_sizes = deserialize_vec(data, config.n_components.div_ceil(4));
    let claimed_sums = deserialize_vec(data, config.n_components);
    Claim { packed_enable_bits, packed_component_log_sizes, claimed_sums }
}

fn deserialize_interaction_at_oods(
    data: &mut &[u8],
    config: &ProofConfig,
) -> Vec<InteractionAtOods<QM31>> {
    config
        .cumulative_sum_columns
        .iter()
        .map(|is_cumulative_sum| {
            let at_oods = QM31::deserialize(data);
            let at_prev = if *is_cumulative_sum { Some(QM31::deserialize(data)) } else { None };
            InteractionAtOods { at_oods, at_prev }
        })
        .collect()
}

fn deserialize_eval_domain_samples(
    data: &mut &[u8],
    config: &ProofConfig,
) -> EvalDomainSamples<QM31> {
    let n_columns_per_trace = config.n_columns_per_trace();
    let n_queries = config.n_queries();
    let mut data_vec = Vec::with_capacity(n_columns_per_trace.len());
    for &n_columns in &n_columns_per_trace {
        let mut trace_data = Vec::with_capacity(n_columns);
        for _ in 0..n_columns {
            let column: Vec<M31> = (0..n_queries).map(|_| M31::deserialize(data)).collect();
            trace_data.push(column);
        }
        data_vec.push(trace_data);
    }
    EvalDomainSamples::from_m31s(data_vec)
}

fn deserialize_eval_domain_auth_paths(data: &mut &[u8], config: &ProofConfig) -> AuthPaths<QM31> {
    let n_queries = config.n_queries();
    let path_len = config.log_evaluation_domain_size();
    let mut trees = Vec::with_capacity(N_COMPOSITION_COLUMNS);
    for _ in 0..4 {
        let mut paths = Vec::with_capacity(n_queries);
        for _ in 0..n_queries {
            let hashes: Vec<HashValue<QM31>> = deserialize_vec(data, path_len);
            paths.push(AuthPath(hashes));
        }
        trees.push(paths);
    }
    AuthPaths { data: trees }
}

fn deserialize_fri_commit_proof(data: &mut &[u8], config: &FriConfig) -> FriCommitProof<QM31> {
    let n_layers = config.log_trace_size - config.log_n_last_layer_coefs;
    let n_last_layer_coefs = 1 << config.log_n_last_layer_coefs;
    FriCommitProof {
        layer_commitments: deserialize_vec(data, n_layers),
        last_layer_coefs: deserialize_vec(data, n_last_layer_coefs),
    }
}

fn deserialize_fri_proof(data: &mut &[u8], config: &FriConfig) -> FriProof<QM31> {
    let commit = deserialize_fri_commit_proof(data, config);

    let log_eval_domain_size = config.log_evaluation_domain_size();
    let mut auth_path_trees = Vec::with_capacity(config.log_trace_size);
    for tree_idx in 0..config.log_trace_size {
        let path_len = log_eval_domain_size - tree_idx - 1;
        let mut paths = Vec::with_capacity(config.n_queries);
        for _ in 0..config.n_queries {
            let hashes: Vec<HashValue<QM31>> = deserialize_vec(data, path_len);
            paths.push(AuthPath(hashes));
        }
        auth_path_trees.push(paths);
    }
    let auth_paths = AuthPaths { data: auth_path_trees };

    let mut fri_siblings = Vec::with_capacity(config.log_trace_size);
    for _ in 0..config.log_trace_size {
        fri_siblings.push(deserialize_vec(data, config.n_queries));
    }

    FriProof { commit, auth_paths, fri_siblings }
}
