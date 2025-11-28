use crate::circuits::blake::{HashValue, blake};
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::ops::Guess;
use crate::circuits::simd::Simd;

#[cfg(test)]
#[path = "merkle_test.rs"]
pub mod test;

const LEAF_HASH: u32 = 0x6661656c; // 'leaf'.
const NODE_HASH: u32 = 0x65646f6e; // 'node'.

/// Represents an authentication path in a Merkle tree.
#[derive(Clone, Debug)]
pub struct AuthPath<T>(pub Vec<HashValue<T>>);

impl<Value: IValue> Guess<Value> for AuthPath<Value> {
    type Target = AuthPath<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        AuthPath(self.0.guess(context))
    }
}

/// Represents the collection of authentication paths for all trees.
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
///
/// NOTE: The caller must check that all input values are in the base field `M31`.
// TODO(lior): Remove the `allow(dead_code)` below, once the function is used.
#[allow(dead_code)]
fn hash_leaf_m31s(context: &mut Context<impl IValue>, values: &[Var]) -> HashValue<Var> {
    let leaf_packed = Simd::pack(context, values);
    let mut data =
        vec![context.constant(LEAF_HASH.into()), context.zero(), context.zero(), context.zero()];
    data.extend_from_slice(leaf_packed.get_packed());

    blake(context, &data, 64 + values.len() * 4)
}

/// Computes the hash of an internal node in the Merkle tree.
#[allow(dead_code)]
fn hash_node(
    context: &mut Context<impl IValue>,
    left: HashValue<Var>,
    right: HashValue<Var>,
) -> HashValue<Var> {
    let data = [
        context.constant(NODE_HASH.into()),
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
