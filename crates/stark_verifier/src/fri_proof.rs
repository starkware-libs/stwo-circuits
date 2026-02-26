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
    /// Whether FRI decommitments use packed leaves (4 QM31 values per leaf).
    pub pack_leaves: bool,
    /// Reduntant field.
    pub line_fold_steps_aux: Vec<usize>,
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
        assert_eq!(self.layer_commitments.len(), config.line_fold_steps_aux.len() + 1);
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
        commit.validate_structure(config);

        assert_eq!(auth_paths.data.len(), config.line_fold_steps_aux.len() + 1);
        // for (tree_idx, tree_data) in auth_paths.data.iter().enumerate() {
        //     assert_eq!(tree_data.len(), config.n_queries);
        //     for query_data in tree_data {
        //         // Reduce size by 1 because we take the sibling of the leaf from `fri_siblings`
        //         // rather than `auth_paths`.
        //         assert_eq!(query_data.0.len(), log_evaluation_domain_size - tree_idx - 1);
        //     }
        // }

        let packed_circle_width = if config.pack_leaves { 4 } else { 1 };
        assert_eq!(circle_fri_siblings.len(), config.n_queries * packed_circle_width);
        assert_eq!(line_coset_vals_per_query_per_tree.len(), config.line_fold_steps_aux.len());
        let mut layer_log_size = config.log_evaluation_domain_size() - 1;
        for (fri_coset_per_query, step) in zip_eq(
            line_coset_vals_per_query_per_tree,
            &config.line_fold_steps_aux,
        )
        {
            assert_eq!(fri_coset_per_query.len(), config.n_queries);
            let expected_len = if config.pack_leaves && layer_log_size >= 2 && *step == 1 {
                4
            } else {
                1 << step
            };
            assert!(fri_coset_per_query.iter().all(|coset| coset.len() == expected_len));
            layer_log_size -= *step;
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
    let n = config.log_evaluation_domain_size();
    let mut layer_log_sizes = vec![n];
    let mut inner_log_size = n - 1;
    for step in &config.line_fold_steps_aux {
        layer_log_sizes.push(inner_log_size);
        inner_log_size -= *step;
    }
    let mut extended_steps = vec![1];
    extended_steps.extend_from_slice(&config.line_fold_steps_aux);
    let auth_paths = AuthPaths {
        data: zip_eq(layer_log_sizes, extended_steps)
            .map(|(log_size, step)| {
                let pack_layer = config.pack_leaves && log_size >= 2;
                let pack_shift = if pack_layer { 2 } else { 0 };
                let start_level_original = if pack_layer { step.max(pack_shift) } else { step };
                let start_level = start_level_original - pack_shift;
                let merkle_log_size = log_size - pack_shift;
                vec![
                    AuthPath(vec![empty_hash; merkle_log_size - start_level]);
                    config.n_queries
                ]
            })
            .collect(),
    };
    let mut layer_log_size = config.log_evaluation_domain_size() - 1;
    let line_coset_vals_per_query_per_tree = config
        .line_fold_steps_aux
        .iter()
        .map(|step| {
            let width = if config.pack_leaves && layer_log_size >= 2 && *step == 1 {
                4
            } else {
                1 << step
            };
            layer_log_size -= *step;
            vec![vec![NoValue; width]; config.n_queries]
        })
        .collect();
    FriProof {
        commit: FriCommitProof {
            layer_commitments: vec![empty_hash; config.log_trace_size],
            last_layer_coefs: vec![NoValue; 1 << config.log_n_last_layer_coefs],
        },
        auth_paths,
        circle_fri_siblings: vec![NoValue; config.n_queries * if config.pack_leaves { 4 } else { 1 }],
        line_coset_vals_per_query_per_tree,
    }
}
