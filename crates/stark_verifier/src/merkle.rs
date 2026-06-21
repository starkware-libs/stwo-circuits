use std::collections::HashMap;

use itertools::{Itertools, zip_eq};
use stwo::core::vcs_lifted::verifier::PACKED_LEAF_SIZE;

use crate::oods::EvalDomainSamples;
use crate::proof::N_TRACES;
use crate::sort_queries::QuerySorter;
use circuits::blake::{HashValue, ReducedHashValue, blake2s, blake2s_u32s, reduce_hash_value};
use circuits::context::{Context, Var};
use circuits::ivalue::IValue;
use circuits::ops::{Guess, cond_flip, eq};
use circuits::simd::Simd;
use circuits::wrappers::{M31Wrapper, U32Wrapper};

#[cfg(test)]
#[path = "merkle_test.rs"]
pub mod test;

/// Represents an authentication path in a Merkle tree.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthPath<T>(pub Vec<HashValue<T>>);

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
) -> HashValue<Var> {
    let leaf_packed = Simd::pack(context, values);
    blake2s(context, leaf_packed.get_packed(), values.len() * 4)
}

/// Computes the hash of a Merkle leaf with a single `QM31` value.
pub fn hash_leaf_qm31(context: &mut Context<impl IValue>, value: Var) -> HashValue<Var> {
    blake2s(context, &[value], 16)
}

/// Computes the hash of a Merkle leaf with 4 `QM31` values.
pub fn hash_packed_leaf_qm31s(
    context: &mut Context<impl IValue>,
    values: [Var; PACKED_LEAF_SIZE],
) -> HashValue<Var> {
    blake2s(context, &values, 64)
}

/// Computes the hash of an internal node in the Merkle tree by hashing `left || right` (64 bytes).
///
/// The children's eight words each are already in `blake2s_u32s` message-word form, so they are fed
/// directly as the 16 message words without any unpacking, and the eight output words are returned
/// losslessly as a [`HashValue`].
pub fn hash_node(
    context: &mut Context<impl IValue>,
    left: &HashValue<Var>,
    right: &HashValue<Var>,
) -> HashValue<Var> {
    let mut words = left.0.to_vec();
    words.extend_from_slice(&right.0);

    // The words are already in `blake2s_u32s` message-word form (the result of a previous
    // `blake2s_u32s`), so they are fed directly as the 16 message words.
    HashValue(blake2s_u32s(context, words, 64))
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
    mut leaf: HashValue<Var>,
    bits: &[Var],
    root: ReducedHashValue<Var>,
    auth_path: &AuthPath<Var>,
) {
    for (bit, sibling) in zip_eq(bits, &auth_path.0) {
        leaf = merkle_node(context, &leaf, sibling, *bit);
    }
    // The Merkle tree is committed losslessly, so the path is computed over lossless
    // `HashValue`s. Reduce the resulting root mod `M31::P` to compare against the
    // `M31`-reduced `root` bound by the Fiat-Shamir channel.
    let reduced = reduce_hash_value(context, leaf);
    eq(context, reduced.0, root.0);
    eq(context, reduced.1, root.1);
}

/// Computes a node of a Merkle tree, given one child `node`, its sibling and the
/// bit indicating which child is `node`.
pub fn merkle_node<Value: IValue>(
    context: &mut Context<Value>,
    node: &HashValue<Var>,
    sibling: &HashValue<Var>,
    bit: Var,
) -> HashValue<Var> {
    // Store node and sibling in the left and right children, word by word.
    let mut left = [U32Wrapper::new_unsafe(bit); 8];
    let mut right = [U32Wrapper::new_unsafe(bit); 8];
    for i in 0..8 {
        let (l, r) = cond_flip(context, bit, *node.0[i].get(), *sibling.0[i].get());
        left[i] = U32Wrapper::new_unsafe(l);
        right[i] = U32Wrapper::new_unsafe(r);
    }

    // Compute the next layer's node.
    hash_node(context, &HashValue(left), &HashValue(right))
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
