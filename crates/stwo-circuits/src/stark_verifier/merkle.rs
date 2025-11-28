use crate::circuits::blake::{HashValue, blake};
use crate::circuits::context::{Context, Var};
use crate::circuits::ivalue::IValue;
use crate::circuits::simd::Simd;

#[cfg(test)]
#[path = "merkle_test.rs"]
pub mod test;

const LEAF_HASH: u32 = 0x6661656c; // 'leaf'.
const NODE_HASH: u32 = 0x65646f6e; // 'node'.

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
