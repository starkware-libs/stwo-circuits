use crate::circuits::blake::HashValue;
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::{IValue, NoValue};
use crate::circuits::ops::Guess;
use crate::stark_verifier::merkle::{AuthPath, AuthPaths};

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
    // Jumps
    pub steps: Vec<usize>,
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
    pub line_coset_vals_per_query_per_tree: Vec<Vec<Vec<T>>>,
    /// For each query, the sibling value in the first layer of FRI (circle to line).
    pub circle_fri_siblings: Vec<T>,
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
        let log_evaluation_domain_size = config.log_evaluation_domain_size();

        commit.validate_structure(config);

        // As layer skipping is not currently supported and the last layer is a constant polynomial,
        // the number of commitment trees should be the same as the trace log size.
        assert_eq!(auth_paths.data.len(), config.log_trace_size);
        for (tree_idx, tree_data) in auth_paths.data.iter().enumerate() {
            assert_eq!(tree_data.len(), config.n_queries);
            for query_data in tree_data {
                // Reduce size by 1 because we take the sibling of the leaf from `fri_siblings`
                // rather than `auth_paths`.
                assert_eq!(query_data.0.len(), log_evaluation_domain_size - tree_idx - 1);
            }
        }

        assert_eq!(circle_fri_siblings.len(), config.n_queries);
        assert_eq!(line_coset_vals_per_query_per_tree.len(), config.log_trace_size - 1);
        for fri_coset_per_query in line_coset_vals_per_query_per_tree {
            assert_eq!(fri_coset_per_query.len(), config.n_queries);
            // TODO: fri_coset_per_query.iter().all(item.len() == 1 << step).
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
                    // Reduce size by 1 because we take the sibling of the leaf from `fri_siblings`
                    // rather than `auth_paths`.
                    AuthPath(vec![empty_hash; config.log_evaluation_domain_size() - tree_idx - 1]);
                    config.n_queries
                ]
            })
            .collect(),
    };

    let line_coset_vals_per_query_per_tree =
        config.steps.iter().map(|step| vec![vec![NoValue; 1 << step]; config.n_queries]).collect();
    FriProof {
        commit: FriCommitProof {
            layer_commitments: vec![empty_hash; config.log_trace_size],
            last_layer_coefs: vec![NoValue; 1 << config.log_n_last_layer_coefs],
        },
        auth_paths,
        circle_fri_siblings: vec![NoValue; config.n_queries],
        line_coset_vals_per_query_per_tree,
    }
}
