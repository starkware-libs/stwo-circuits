use crate::circuits::blake::HashValue;
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::{IValue, NoValue};
use crate::circuits::ops::Guess;
use crate::stark_verifier::merkle::{AuthPath, AuthPaths};

/// Represents the structure of a FRI proof.
pub struct FriConfig {
    /// Log2 of the trace size.
    pub log_trace_size: usize,
    /// Log2 of the blowup factor.
    pub log_blowup_factor: usize,
    /// Number of queries.
    pub n_queries: usize,
    /// Log2 of the number of coefficients in the last layer of FRI.
    pub log_n_last_layer_coefs: usize,
}

impl FriConfig {
    pub fn log_evaluation_domain_size(&self) -> usize {
        self.log_trace_size + self.log_blowup_factor
    }
}

/// Represents the information for the FRI commitment phase of the proof.
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
pub struct FriProof<T> {
    /// Information regarding the FRI commitment phase.
    pub commit: FriCommitProof<T>,
    /// Authentication paths for all the FRI trees.
    pub auth_paths: AuthPaths<T>,
    /// For each layer, for each query, the sibling value.
    pub fri_siblings: Vec<Vec<T>>,
}

impl<T> FriProof<T> {
    /// Validates that the size of the members of the struct are consistent with the config.
    pub fn validate_structure(&self, config: &FriConfig) {
        let FriProof { commit, auth_paths, fri_siblings } = self;
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

        assert_eq!(fri_siblings.len(), config.log_trace_size);
        for siblings in fri_siblings {
            assert_eq!(siblings.len(), config.n_queries);
        }
    }
}

impl<Value: IValue> Guess<Value> for FriProof<Value> {
    type Target = FriProof<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        Self::Target {
            commit: self.commit.guess(context),
            auth_paths: self.auth_paths.guess(context),
            fri_siblings: self.fri_siblings.guess(context),
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

    FriProof {
        commit: FriCommitProof {
            layer_commitments: vec![empty_hash; config.log_trace_size],
            last_layer_coefs: vec![NoValue; 1 << config.log_n_last_layer_coefs],
        },
        auth_paths,
        fri_siblings: vec![vec![NoValue; config.n_queries]; config.log_trace_size],
    }
}
