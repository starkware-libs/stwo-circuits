use std::collections::HashMap;

use itertools::{Itertools, zip_eq};
use stwo::core::vcs_lifted::verifier::PACKED_LEAF_SIZE;

use crate::oods::EvalDomainSamples;
use crate::proof::N_TRACES;
use crate::sort_queries::QuerySorter;
use circuits::blake::{ReducedHashValue, blake2s_m31};
use circuits::context::{Context, Var};
use circuits::ivalue::IValue;
use circuits::ops::{Guess, cond_flip, eq};
use circuits::simd::Simd;
use circuits::wrappers::M31Wrapper;

#[cfg(test)]
#[path = "merkle_test.rs"]
pub mod test;

/// Represents an authentication path in a Merkle tree.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthPath<T>(pub Vec<ReducedHashValue<T>>);

impl<Value: IValue> Guess<Value> for AuthPath<Value> {
    type Target = AuthPath<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        AuthPath(self.0.guess(context))
    }
}

/// Represents the collection of authentication paths for a set of trees.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthPaths<T> {
    // For each tree, for each query, the authentication path.
    pub data: Vec<Vec<AuthPath<T>>>,
}
impl<T> AuthPaths<T> {
    /// Returns the number of trees represented.
    pub fn n_trees(&self) -> usize {
        self.data.len()
    }

    /// Returns the authentication path for the given tree and query.
    pub fn at(&self, tree_idx: usize, query_idx: usize) -> &AuthPath<T> {
        &self.data[tree_idx][query_idx]
    }

    /// Validates that the structure of the authentication paths is consistent with the tree heights
    /// and the number of queries.
    pub fn validate_structure(&self, tree_heights: &[usize], n_queries: usize) {
        for (tree_data, height) in zip_eq(&self.data, tree_heights) {
            assert_eq!(tree_data.len(), n_queries);
            for path in tree_data {
                assert_eq!(path.0.len(), *height);
            }
        }
    }
}

impl<Value: IValue> Guess<Value> for AuthPaths<Value> {
    type Target = AuthPaths<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        AuthPaths { data: self.data.guess(context) }
    }
}

/// Computes the hash of a Merkle leaf. The input is a vector of `M31` values.
fn hash_leaf_m31s(
    context: &mut Context<impl IValue>,
    values: &[M31Wrapper<Var>],
) -> ReducedHashValue<Var> {
    let leaf_packed = Simd::pack(context, values);
    blake2s_m31(context, leaf_packed.get_packed(), values.len() * 4)
}

/// Computes the hash of a Merkle leaf with a single `QM31` value.
pub fn hash_leaf_qm31(context: &mut Context<impl IValue>, value: Var) -> ReducedHashValue<Var> {
    blake2s_m31(context, &[value], 16)
}

/// Computes the hash of a Merkle leaf with 4 `QM31` values.
pub fn hash_packed_leaf_qm31s(
    context: &mut Context<impl IValue>,
    values: [Var; PACKED_LEAF_SIZE],
) -> ReducedHashValue<Var> {
    blake2s_m31(context, &values, 64)
}

/// Computes the hash of an internal node in the Merkle tree.
pub fn hash_node(
    context: &mut Context<impl IValue>,
    left: ReducedHashValue<Var>,
    right: ReducedHashValue<Var>,
) -> ReducedHashValue<Var> {
    let data = [left.0, left.1, right.0, right.1];

    blake2s_m31(context, &data, 64)
}

/// Validates that the leaf at the index given by `bits` has the value `leaf` in a Merkle tree
/// with the given `root`.
///
/// `auth_path` is the authentication path such that `auth_path[0]` is the sibling of `leaf`.
///
/// This is done by computing the root from `leaf` and `auth_path` and comparing it to the given
/// `root`.
pub fn verify_merkle_path<Value: IValue>(
    context: &mut Context<Value>,
    mut leaf: ReducedHashValue<Var>,
    bits: &[Var],
    root: ReducedHashValue<Var>,
    auth_path: &AuthPath<Var>,
) {
    for (bit, sibling) in zip_eq(bits, &auth_path.0) {
        leaf = merkle_node(context, &leaf, sibling, *bit);
    }
    eq(context, leaf.0, root.0);
    eq(context, leaf.1, root.1);
}

/// Computes a node of a Merkle tree, given one child `node`, its sibling and the
/// bit indicating which child is `node`.
pub fn merkle_node<Value: IValue>(
    context: &mut Context<Value>,
    node: &ReducedHashValue<Var>,
    sibling: &ReducedHashValue<Var>,
    bit: Var,
) -> ReducedHashValue<Var> {
    // Store leaf and sibling in the left and right children.
    let (left0, right0) = cond_flip(context, bit, node.0, sibling.0);
    let (left1, right1) = cond_flip(context, bit, node.1, sibling.1);

    // Compute the next layer's node.
    hash_node(context, ReducedHashValue(left0, left1), ReducedHashValue(right0, right1))
}

/// Verifies that the queries in `eval_domain_samples` are consistent with the Merkle roots.
///
/// `bits[i][query_idx]` is the `i`-th bit of the bit representation of the query at index
/// `query_idx`.
/// `opt_column_log_sizes_by_trace` maps a trace index to the column log sizes used to sort
/// that trace's query columns into committed order. A trace absent from the map is left unsorted,
/// so the map is empty when all columns are already in committed order and sorting can be skipped.
pub fn decommit_eval_domain_samples<Value: IValue>(
    context: &mut Context<Value>,
    n_queries: usize,
    opt_column_log_sizes_by_trace: &HashMap<usize, Vec<Var>>,
    eval_domain_samples: &EvalDomainSamples<Var>,
    auth_paths: &AuthPaths<Var>,
    bits: &[Vec<Var>],
    roots: &[ReducedHashValue<Var>; N_TRACES],
) {
    assert_eq!(eval_domain_samples.n_traces(), roots.len());
    assert_eq!(auth_paths.n_trees(), roots.len());

    for (trace_idx, root) in roots.iter().enumerate() {
        let data = eval_domain_samples.data_for_trace(trace_idx);

        let mut query_sorter = match opt_column_log_sizes_by_trace.get(&trace_idx) {
            Some(column_log_sizes) => QuerySorter::new(context, column_log_sizes),
            None => QuerySorter::skip_sorting(),
        };

        for query_idx in 0..n_queries {
            let query_values =
                data.iter().map(|column_data| column_data[query_idx].clone()).collect_vec();

            let sorted = query_sorter.sort(context, query_values);
            let leaf = hash_leaf_m31s(context, &sorted);
            let auth_path = auth_paths.at(trace_idx, query_idx);
            let bits_for_query = bits.iter().map(|b| b[query_idx]).collect_vec();
            verify_merkle_path(context, leaf, &bits_for_query, *root, auth_path);
        }
    }
}
