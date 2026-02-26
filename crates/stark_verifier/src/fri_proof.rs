use crate::merkle::{AuthPath, AuthPaths};
use circuits::blake::HashValue;
use circuits::context::{Context, Var};
use circuits::ivalue::{IValue, NoValue};
use circuits::ops::Guess;
use itertools::zip_eq;

/// Represents the structure of a FRI proof.
#[derive(Debug, PartialEq)]
pub struct FriConfig {
    /// Log2 of the trace size.
    pub log_trace_size: usize,
    /// Log2 of the blowup factor.
    pub log_blowup_factor: usize,
    /// Number of queries.
    pub n_queries: usize,
    /// Log2 of the number of coefficients in the last layer of FRI.
    pub log_n_last_layer_coefs: usize,
    /// The step of the line folds in FRI's inner layers.
    pub line_fold_step: usize,
    /// The step of the first circle-to-line fold.
    pub circle_fold_step: usize,
}

impl FriConfig {
    pub fn log_evaluation_domain_size(&self) -> usize {
        self.log_trace_size + self.log_blowup_factor
    }
}

/// Represents the information for the FRI commitment phase of the proof.
#[derive(Debug, PartialEq)]
pub struct FriCommitProof<T> {
    pub layer_commitments: Vec<HashValue<T>>,
    pub last_layer_coefs: Vec<T>,
}

impl<T> FriCommitProof<T> {
    /// Validates that the size of the members of the struct are consistent with the config.
    pub fn validate_structure(&self, config: &FriConfig) {
        // Starting from the last layer, each layer increases the log2 of the polynomial degree by
        // one. The final degree should be the same as the trace size.
        assert_eq!(
            config.log_n_last_layer_coefs + self.layer_commitments.len(),
            config.log_trace_size
        );
        assert_eq!(self.last_layer_coefs.len(), 1 << config.log_n_last_layer_coefs);
    }
}

impl<Value: IValue> Guess<Value> for FriCommitProof<Value> {
    type Target = FriCommitProof<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        Self::Target {
            layer_commitments: self.layer_commitments.guess(context),
            last_layer_coefs: self.last_layer_coefs.guess(context),
        }
    }
}

/// Represents the information required to verify a FRI proof.
#[derive(Debug, PartialEq)]
pub struct FriProof<T> {
    /// Information regarding the FRI commitment phase.
    pub commit: FriCommitProof<T>,
    /// Authentication paths for all the FRI trees.
    pub auth_paths: AuthPaths<T>,
    /// Deprecated witness field for first-layer siblings.
    /// New proofs keep this empty and provide full per-layer coset witnesses instead.
    pub circle_fri_siblings: Vec<T>,
    /// For each FRI layer, for each query, the values of the layer polynomial on the FRI coset
    /// containing that query.
    pub line_coset_vals_per_query_per_tree: Vec<Vec<Vec<T>>>,
}

impl<T> FriProof<T> {
    /// Validates that the size of the members of the struct are consistent with the config.
    pub fn validate_structure(&self, config: &FriConfig) {
        let FriProof {
            commit,
            auth_paths,
            line_coset_vals_per_query_per_tree,
            circle_fri_siblings,
        } = self;
        commit.validate_structure(config);
        let all_line_fold_steps = compute_all_line_fold_steps(
            config.log_trace_size - config.circle_fold_step - config.log_n_last_layer_coefs,
            config.line_fold_step,
        );

        assert_eq!(auth_paths.data.len(), all_line_fold_steps.len() + 1);
        let first_layer_log_size = config.log_evaluation_domain_size();
        let mut all_fold_steps = vec![config.circle_fold_step];
        all_fold_steps.extend_from_slice(&all_line_fold_steps);
        let mut fold_sum = 0;
        for (tree_data, fold_step) in zip_eq(&auth_paths.data, &all_fold_steps) {
            assert_eq!(tree_data.len(), config.n_queries);
            for query_data in tree_data {
                assert_eq!(query_data.0.len(), first_layer_log_size - fold_sum - *fold_step);
            }
            fold_sum += *fold_step
        }

        assert!(circle_fri_siblings.is_empty());
        assert_eq!(line_coset_vals_per_query_per_tree.len(), all_fold_steps.len());
        for (fri_coset_per_query, step) in zip_eq(line_coset_vals_per_query_per_tree, &all_fold_steps) {
            assert_eq!(fri_coset_per_query.len(), config.n_queries);
            fri_coset_per_query.iter().all(|coset| coset.len() == 1 << step);
        }
    }
}

impl<Value: IValue> Guess<Value> for FriProof<Value> {
    type Target = FriProof<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        Self::Target {
            commit: self.commit.guess(context),
            auth_paths: self.auth_paths.guess(context),
            circle_fri_siblings: self.circle_fri_siblings.guess(context),
            line_coset_vals_per_query_per_tree: self
                .line_coset_vals_per_query_per_tree
                .guess(context),
        }
    }
}

pub fn empty_fri_proof(config: &FriConfig) -> FriProof<NoValue> {
    let empty_hash = HashValue(NoValue, NoValue);
    let auth_paths = AuthPaths {
        data: (0..config.log_trace_size)
            .map(|tree_idx| {
                vec![
                    AuthPath(vec![empty_hash; config.log_evaluation_domain_size() - tree_idx - 1]);
                    config.n_queries
                ]
            })
            .collect(),
    };

    let all_line_fold_steps = compute_all_line_fold_steps(
        config.log_trace_size - config.circle_fold_step - config.log_n_last_layer_coefs,
        config.line_fold_step,
    );
    let all_fold_steps = [&[config.circle_fold_step], all_line_fold_steps.as_slice()].concat();
    let line_coset_vals_per_query_per_tree = all_fold_steps
        .iter()
        .map(|step| vec![vec![NoValue; 1 << step]; config.n_queries])
        .collect();
    FriProof {
        commit: FriCommitProof {
            layer_commitments: vec![empty_hash; config.log_trace_size],
            last_layer_coefs: vec![NoValue; 1 << config.log_n_last_layer_coefs],
        },
        auth_paths,
        circle_fri_siblings: vec![],
        line_coset_vals_per_query_per_tree,
    }
}

/// Computes all the line-to-line folding steps.
///
/// # Arguments
///
/// - `line_degree_log_ratio`: (log degree of FRI's second layer poly) - (log degree of FRI's last
///   layer).
/// - `line_fold_step`: the folding step of all the line-to-line folds except possibly the last.
pub fn compute_all_line_fold_steps(
    line_degree_log_ratio: usize,
    line_fold_step: usize,
) -> Vec<usize> {
    let n_folds = line_degree_log_ratio.div_ceil(line_fold_step);
    let rem = line_degree_log_ratio % line_fold_step;
    let mut line_fold_steps = vec![line_fold_step; n_folds];
    line_fold_steps[n_folds - 1] = if rem == 0 { line_fold_step } else { rem };
    line_fold_steps
}
