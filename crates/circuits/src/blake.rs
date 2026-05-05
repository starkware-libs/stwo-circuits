use blake2::{Blake2s256, Digest};
use itertools::Itertools;
use stwo::core::vcs::blake2_hash::Blake2sHash;
use stwo::core::{fields::qm31::QM31, vcs::blake2_hash::reduce_to_m31};

use crate::circuit::{Blake, BlakeGGate, M31ToU32, TripleXor};
use crate::context::{Context, Var};
use crate::ivalue::{IValue, qm31_from_u32s};
use crate::ops::Guess;

#[cfg(test)]
#[path = "blake_test.rs"]
pub mod test;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HashValue<T>(pub T, pub T);

impl<Value: IValue> Guess<Value> for HashValue<Value> {
    type Target = HashValue<Var>;
    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        HashValue(self.0.guess(context), self.1.guess(context))
    }
}

impl From<Blake2sHash> for HashValue<QM31> {
    fn from(value: Blake2sHash) -> Self {
        HashValue(
            qm31_from_bytes(&value.0[0..16].try_into().unwrap()),
            qm31_from_bytes(&value.0[16..32].try_into().unwrap()),
        )
    }
}

impl From<[u32; 8]> for HashValue<QM31> {
    fn from(value: [u32; 8]) -> Self {
        HashValue(
            qm31_from_u32s(value[0], value[1], value[2], value[3]),
            qm31_from_u32s(value[4], value[5], value[6], value[7]),
        )
    }
}

/// Convert QM31 to 16 bytes (4 u32s)
fn to_bytes(value: QM31) -> [u8; 16] {
    let mut bytes = [0u8; 16];
    bytes[0..4].copy_from_slice(&value.0.0.0.to_le_bytes());
    bytes[4..8].copy_from_slice(&value.0.1.0.to_le_bytes());
    bytes[8..12].copy_from_slice(&value.1.0.0.to_le_bytes());
    bytes[12..16].copy_from_slice(&value.1.1.0.to_le_bytes());
    bytes
}

/// Create QM31 from 16 bytes (4 u32s)
pub fn qm31_from_bytes(bytes: &[u8; 16]) -> QM31 {
    let a = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let b = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let c = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let d = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
    qm31_from_u32s(a, b, c, d)
}

/// Blake2s hash function implementation for QM31.
/// Takes [QM31] values as input and returns 2 [QM31] values as output.
pub fn blake_qm31(input: &[QM31], n_bytes: usize) -> HashValue<QM31> {
    // Sanity check: check the number of bytes is consistent with the number of [QM31] values.
    assert_eq!(input.len(), n_bytes.div_ceil(16));

    // Convert [QM31] inputs to bytes.
    let mut input_bytes: Vec<u8> = vec![];
    for x in input {
        input_bytes.extend_from_slice(&to_bytes(*x));
    }

    let mut hasher = Blake2s256::new();
    hasher.update(&input_bytes[0..n_bytes]);
    let hash: [u8; 32] = reduce_to_m31(hasher.finalize().into());

    let res0 = qm31_from_bytes(hash[0..16].try_into().unwrap());
    let res1 = qm31_from_bytes(hash[16..32].try_into().unwrap());

    HashValue(res0, res1)
}

/// Adds a blake hash gate to the circuit, and returns the two output variables as [HashValue].
///
/// NOTE: If the number of bytes is not a multiple of 16, the caller must make sure that the
/// remaining bytes are zero.
/// For example, if `n_bytes` is 4, only the first coordinate of the [QM31] may be non-zero.
/// If `n_bytes` is 1, that coordinate must be < 256.
pub fn blake<Value: IValue>(
    context: &mut Context<Value>,
    input: &[Var],
    n_bytes: usize,
) -> HashValue<Var> {
    // Sanity check: check the number of bytes is consistent with the number of [QM31] values.
    assert_eq!(input.len(), n_bytes.div_ceil(16));

    // Compute the hash.
    let out = Value::blake(&input.iter().map(|v| context.get(*v)).collect::<Vec<_>>(), n_bytes);

    // Pad input with zeros and split into chunks of 4 [QM31] values.
    let zero_idx = context.zero().idx;
    let chunks = input
        .iter()
        .chunks(4)
        .into_iter()
        .map(|chunk| {
            let mut res = [zero_idx; 4];
            for (i, v) in chunk.enumerate() {
                res[i] = v.idx;
            }
            res
        })
        .collect_vec();

    context.stats.blake_updates += chunks.len();
    let out_var0 = context.new_var(out.0);
    let out_var1 = context.new_var(out.1);

    context.circuit.blake.push(Blake {
        input: chunks,
        n_bytes,
        out0: out_var0.idx,
        out1: out_var1.idx,
    });

    HashValue(out_var0, out_var1)
}

/// Adds a TripleXor gate to the circuit: XOR three u32 values encoded as QM31 `(u16, u16, 0, 0)`
/// and return the result in the same encoding.
pub fn triple_xor<Value: IValue>(
    ctx: &mut Context<Value>,
    input_a: Var,
    input_b: Var,
    input_c: Var,
) -> Var {
    let a = ctx.get(input_a).unpack_u32();
    let b = ctx.get(input_b).unpack_u32();
    let c = ctx.get(input_c).unpack_u32();
    let out = ctx.new_var(Value::pack_u32(a ^ b ^ c));
    ctx.stats.triple_xor += 1;
    ctx.circuit.triple_xor.push(TripleXor {
        input_a: input_a.idx,
        input_b: input_b.idx,
        input_c: input_c.idx,
        out: out.idx,
    });
    out
}

/// Blake2s mixing function *G* on four state words `(a, b, c, d)` with message words `f0`, `f1`.
#[must_use]
pub fn blake2s_g(a: u32, b: u32, c: u32, d: u32, f0: u32, f1: u32) -> (u32, u32, u32, u32) {
    let a = a.wrapping_add(b).wrapping_add(f0);
    let d = (d ^ a).rotate_right(16);
    let c = c.wrapping_add(d);
    let b = (b ^ c).rotate_right(12);
    let a = a.wrapping_add(b).wrapping_add(f1);
    let d = (d ^ a).rotate_right(8);
    let c = c.wrapping_add(d);
    let b = (b ^ c).rotate_right(7);
    (a, b, c, d)
}

/// Adds an M31ToU32 gate to the circuit: convert an `M31` value into its `u32` representation, i.e
/// `(x, 0, 0, 0)` into `(x & 0xFFFF, x >> 16, 0, 0)`.
pub fn m31_to_u32<Value: IValue>(ctx: &mut Context<Value>, input: Var) -> Var {
    let out = ctx.new_var(ctx.get(input).m31_to_u32());
    ctx.stats.m31_to_u32 += 1;
    ctx.circuit.m31_to_u32.push(M31ToU32 { input: input.idx, out: out.idx });
    out
}

/// Adds a Blake2s G function gate to the circuit: G(a, b, c, d, f0, f1) = (a', b', c', d').
/// Inputs and outputs are all encoded as `(low_u16, high_u16, 0, 0)` in QM31.
pub fn blake_g_gate<Value: IValue>(
    ctx: &mut Context<Value>,
    input_a: Var,
    input_b: Var,
    input_c: Var,
    input_d: Var,
    input_f0: Var,
    input_f1: Var,
) -> (Var, Var, Var, Var) {
    let a = ctx.get(input_a).unpack_u32();
    let b = ctx.get(input_b).unpack_u32();
    let c = ctx.get(input_c).unpack_u32();
    let d = ctx.get(input_d).unpack_u32();
    let f0 = ctx.get(input_f0).unpack_u32();
    let f1 = ctx.get(input_f1).unpack_u32();

    let (a_out, b_out, c_out, d_out) = blake2s_g(a, b, c, d, f0, f1);

    let out_a = ctx.new_var(Value::pack_u32(a_out));
    let out_b = ctx.new_var(Value::pack_u32(b_out));
    let out_c = ctx.new_var(Value::pack_u32(c_out));
    let out_d = ctx.new_var(Value::pack_u32(d_out));

    ctx.circuit.blake_g_gate.push(BlakeGGate {
        input_a: input_a.idx,
        input_b: input_b.idx,
        input_c: input_c.idx,
        input_d: input_d.idx,
        input_f0: input_f0.idx,
        input_f1: input_f1.idx,
        out_a: out_a.idx,
        out_b: out_b.idx,
        out_c: out_c.idx,
        out_d: out_d.idx,
    });

    (out_a, out_b, out_c, out_d)
}
