use std::fmt;

use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use circuits::blake::HashValue;
use circuits::wrappers::M31Wrapper;
use circuits_stark_verifier::fri_proof::{
    FriCommitProof, FriConfig, FriProof, compute_all_line_fold_steps,
};
use circuits_stark_verifier::merkle::{AuthPath, AuthPaths};
use circuits_stark_verifier::oods::{EvalDomainSamples, N_COMPOSITION_COLUMNS};
use circuits_stark_verifier::proof::{Claim, InteractionAtOods, Proof, ProofConfig};

#[derive(Debug)]
pub struct DeserializeError;

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not enough data to deserialize")
    }
}

impl std::error::Error for DeserializeError {}

pub type DeserializeResult<T> = Result<T, DeserializeError>;

/// Deserializes types from a byte format serialized by the corresponding `CircuitSerialize`
/// implementations. Fixed-size types implement the trait directly; composite types that require
/// length information from the [`ProofConfig`] use dedicated `deserialize_with_config` methods.
pub trait CircuitDeserialize: Sized {
    fn deserialize(data: &mut &[u8]) -> DeserializeResult<Self>;
}

impl CircuitDeserialize for M31 {
    fn deserialize(data: &mut &[u8]) -> DeserializeResult<Self> {
        let Some(&[a, b, c, d]) = data.split_off(..4) else {
            return Err(DeserializeError);
        };

        Ok(M31::from(u32::from_le_bytes([a, b, c, d])))
    }
}

impl CircuitDeserialize for QM31 {
    fn deserialize(data: &mut &[u8]) -> DeserializeResult<Self> {
        let m31_values: [M31; 4] = [
            M31::deserialize(data)?,
            M31::deserialize(data)?,
            M31::deserialize(data)?,
            M31::deserialize(data)?,
        ];
        Ok(QM31::from_m31_array(m31_values))
    }
}

impl<T: CircuitDeserialize, const N: usize> CircuitDeserialize for [T; N] {
    fn deserialize(data: &mut &[u8]) -> DeserializeResult<Self> {
        (0..N)
            .map(|_| T::deserialize(data))
            .collect::<DeserializeResult<Vec<_>>>()?
            .try_into()
            .map_err(|_| DeserializeError)
    }
}

impl CircuitDeserialize for HashValue<QM31> {
    fn deserialize(data: &mut &[u8]) -> DeserializeResult<Self> {
        Ok(HashValue(QM31::deserialize(data)?, QM31::deserialize(data)?))
    }
}

impl CircuitDeserialize for M31Wrapper<QM31> {
    fn deserialize(data: &mut &[u8]) -> DeserializeResult<Self> {
        let m31 = M31::deserialize(data)?;
        Ok(M31Wrapper::from(m31))
    }
}

/// Deserializes a `Vec<T>` of the given length from the byte stream.
fn deserialize_vec<T: CircuitDeserialize>(
    data: &mut &[u8],
    len: usize,
) -> DeserializeResult<Vec<T>> {
    (0..len).map(|_| T::deserialize(data)).collect()
}

/// Deserializes a proof from a byte stream, using the [`ProofConfig`] for all length
/// information.
pub fn deserialize_proof_with_config(
    data: &mut &[u8],
    config: &ProofConfig,
) -> DeserializeResult<Proof<QM31>> {
    let channel_salt = QM31::deserialize(data)?;
    let preprocessed_root = HashValue::<QM31>::deserialize(data)?;
    let trace_root = HashValue::<QM31>::deserialize(data)?;
    let interaction_root = HashValue::<QM31>::deserialize(data)?;
    let composition_polynomial_root = HashValue::<QM31>::deserialize(data)?;
    let preprocessed_columns_at_oods = deserialize_vec(data, config.n_preprocessed_columns)?;
    let trace_at_oods = deserialize_vec(data, config.n_trace_columns)?;
    let composition_eval_at_oods = <[QM31; N_COMPOSITION_COLUMNS]>::deserialize(data)?;
    let claim = deserialize_claim(data, config)?;
    let interaction_at_oods = deserialize_interaction_at_oods(data, config)?;
    let eval_domain_samples = deserialize_eval_domain_samples(data, config)?;
    let eval_domain_auth_paths = deserialize_eval_domain_auth_paths(data, config)?;
    let proof_of_work_nonce = QM31::deserialize(data)?;
    let interaction_pow_nonce = QM31::deserialize(data)?;
    // Deserialize FRI proof.
    let all_line_fold_steps = compute_all_line_fold_steps(
        config.fri.log_trace_size - 1 - config.fri.log_n_last_layer_coefs,
        config.fri.line_fold_step,
    );
    let fri = deserialize_fri_proof(data, &config.fri, &all_line_fold_steps)?;

    Ok(Proof {
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
    })
}

fn deserialize_claim(data: &mut &[u8], config: &ProofConfig) -> DeserializeResult<Claim<QM31>> {
    // TODO(Gali): Serialize more efficiently.
    let packed_enable_bits = deserialize_vec(data, config.n_components.div_ceil(4))?;
    let packed_component_log_sizes = deserialize_vec(data, config.n_components.div_ceil(4))?;
    let claimed_sums = deserialize_vec(data, config.n_components)?;
    Ok(Claim { packed_enable_bits, packed_component_log_sizes, claimed_sums })
}

fn deserialize_interaction_at_oods(
    data: &mut &[u8],
    config: &ProofConfig,
) -> DeserializeResult<Vec<InteractionAtOods<QM31>>> {
    config
        .cumulative_sum_columns
        .iter()
        .map(|is_cumulative_sum| {
            let at_oods = QM31::deserialize(data)?;
            let at_prev = if *is_cumulative_sum { Some(QM31::deserialize(data)?) } else { None };
            Ok(InteractionAtOods { at_oods, at_prev })
        })
        .collect()
}

fn deserialize_eval_domain_samples(
    data: &mut &[u8],
    config: &ProofConfig,
) -> DeserializeResult<EvalDomainSamples<QM31>> {
    let n_columns_per_trace = config.n_columns_per_trace();
    let n_queries = config.n_queries();
    let mut data_vec = Vec::with_capacity(n_columns_per_trace.len());
    for &n_columns in &n_columns_per_trace {
        let mut trace_data = Vec::with_capacity(n_columns);
        for _ in 0..n_columns {
            let column: Vec<M31> =
                (0..n_queries).map(|_| M31::deserialize(data)).collect::<DeserializeResult<_>>()?;
            trace_data.push(column);
        }
        data_vec.push(trace_data);
    }
    Ok(EvalDomainSamples::from_m31s(data_vec))
}

fn deserialize_eval_domain_auth_paths(
    data: &mut &[u8],
    config: &ProofConfig,
) -> DeserializeResult<AuthPaths<QM31>> {
    let n_queries = config.n_queries();
    let path_len = config.log_evaluation_domain_size();
    let mut trees = Vec::with_capacity(N_COMPOSITION_COLUMNS);
    for _ in 0..4 {
        let mut paths = Vec::with_capacity(n_queries);
        for _ in 0..n_queries {
            let hashes: Vec<HashValue<QM31>> = deserialize_vec(data, path_len)?;
            paths.push(AuthPath(hashes));
        }
        trees.push(paths);
    }
    Ok(AuthPaths { data: trees })
}

fn deserialize_fri_commit_proof(
    data: &mut &[u8],
    config: &FriConfig,
    all_line_fold_steps: &[usize],
) -> DeserializeResult<FriCommitProof<QM31>> {
    let n_layers = 1 + all_line_fold_steps.len();
    let n_last_layer_coefs = 1 << config.log_n_last_layer_coefs;
    Ok(FriCommitProof {
        layer_commitments: deserialize_vec(data, n_layers)?,
        last_layer_coefs: deserialize_vec(data, n_last_layer_coefs)?,
    })
}

fn deserialize_fri_proof(
    data: &mut &[u8],
    config: &FriConfig,
    all_line_fold_steps: &[usize],
) -> DeserializeResult<FriProof<QM31>> {
    let commit = deserialize_fri_commit_proof(data, config, all_line_fold_steps)?;

    let log_eval_domain_size = config.log_evaluation_domain_size();
    // The circle-to-line fold is hardcoded to 1 currently.
    let all_fold_steps = [&[1], all_line_fold_steps].concat();
    let mut fold_sum = 0;
    let mut auth_path_trees = Vec::with_capacity(all_fold_steps.len());
    for step in all_fold_steps.iter() {
        let path_len = log_eval_domain_size - fold_sum - step;
        let mut paths = Vec::with_capacity(config.n_queries);
        for _ in 0..config.n_queries {
            let hashes: Vec<HashValue<QM31>> = deserialize_vec(data, path_len)?;
            paths.push(AuthPath(hashes));
        }
        auth_path_trees.push(paths);
        fold_sum += step;
    }
    let auth_paths = AuthPaths { data: auth_path_trees };
    // Deserialize fri siblings of the first layer and line coset witnesses.
    let circle_fri_siblings = deserialize_vec(data, config.n_queries)?;
    let mut line_coset_vals_per_query_per_tree = vec![];
    for step in all_line_fold_steps.iter() {
        let mut line_coset_vals_per_query = vec![];
        for _ in 0..config.n_queries {
            let coset: Vec<QM31> = deserialize_vec(data, 1 << step)?;
            line_coset_vals_per_query.push(coset);
        }
        line_coset_vals_per_query_per_tree.push(line_coset_vals_per_query);
    }

    Ok(FriProof { commit, auth_paths, circle_fri_siblings, line_coset_vals_per_query_per_tree })
}
