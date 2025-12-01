use itertools::zip_eq;

use crate::circuits::blake::{HashValue, blake};
use crate::circuits::circuit::Var;
use crate::circuits::context::Context;
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::{Guess, cond_flip, eq};
use crate::circuits::simd::Simd;
use crate::circuits::wrappers::M31Wrapper;

#[cfg(test)]
#[path = "merkle_test.rs"]
pub mod test;

const LEAF_PREFIX: u32 = 0x6661656c; // 'leaf'.
const NODE_PREFIX: u32 = 0x65646f6e; // 'node'.

/// Represents an authentication path in a Merkle tree.
#[derive(Clone, Debug)]
pub struct AuthPath<T>(pub Vec<HashValue<T>>);

impl<Value: IValue> Guess<Value> for AuthPath<Value> {
    type Target = AuthPath<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        AuthPath(self.0.guess(context))
    }
}

/// Represents the collection of authentication paths for a set of trees.
pub struct AuthPaths<T> {
    // For each tree, for each query, the authentication path.
    pub data: Vec<Vec<AuthPath<T>>>,
}
impl<T> AuthPaths<T> {
    pub fn at(&self, tree_idx: usize, query_idx: usize) -> &AuthPath<T> {
        &self.data[tree_idx][query_idx]
    }
}

impl<Value: IValue> Guess<Value> for AuthPaths<Value> {
    type Target = AuthPaths<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        AuthPaths { data: self.data.guess(context) }
    }
}

/// Computes the hash of a Merkle leaf. The input is a vector of `M31` values.
// TODO(lior): Remove the `allow(dead_code)` below, once the function is used.
#[allow(dead_code)]
fn hash_leaf_m31s(
    context: &mut Context<impl IValue>,
    values: &[M31Wrapper<Var>],
) -> HashValue<Var> {
    let leaf_packed = Simd::pack(context, values);
    let mut data =
        vec![context.constant(LEAF_PREFIX.into()), context.zero(), context.zero(), context.zero()];
    data.extend_from_slice(leaf_packed.get_packed());

    blake(context, &data, 64 + values.len() * 4)
}

/// Computes the hash of an internal node in the Merkle tree.
fn hash_node(
    context: &mut Context<impl IValue>,
    left: HashValue<Var>,
    right: HashValue<Var>,
) -> HashValue<Var> {
    let data = [
        context.constant(NODE_PREFIX.into()),
        context.zero(),
        context.zero(),
        context.zero(),
        left.0,
        left.1,
        right.0,
        right.1,
    ];

    blake(context, &data, 128)
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
    root: HashValue<Var>,
    auth_path: &AuthPath<Var>,
) {
    for (bit, sibling) in zip_eq(bits, &auth_path.0) {
        // Store leaf and sibling in the left and right children.
        let (left0, right0) = cond_flip(context, *bit, leaf.0, sibling.0);
        let (left1, right1) = cond_flip(context, *bit, leaf.1, sibling.1);

        // Compute the next layer's node.
        leaf = hash_node(context, HashValue(left0, left1), HashValue(right0, right1));
    }
    eq(context, leaf.0, root.0);
    eq(context, leaf.1, root.1);
}
