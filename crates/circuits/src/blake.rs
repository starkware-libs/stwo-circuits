use blake2::{Blake2s256, Digest};
use itertools::Itertools;
use stwo::core::vcs::blake2_hash::Blake2sHash;
use stwo::core::{fields::qm31::QM31, vcs::blake2_hash::reduce_to_m31};

use crate::circuit::{Blake, BlakeGGate, M31ToU32Gate, TripleXorGate};
use crate::context::{Context, TraceContext, Var};
use crate::eval;
use crate::ivalue::{IValue, qm31_from_u32s};
use crate::ops::{Guess, from_partial_evals};
use crate::simd::Simd;
use crate::wrappers::U32Wrapper;

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

// --- Constants for Blake2s ---

pub const BLAKE2S_IV: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];

pub const BLAKE_SIGMA: [[u8; 16]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
];

/// The 8 (a,b,c,d) state index tuples per round: 4 columns then 4 diagonals.
pub const G_STATE_INDICES: [[usize; 4]; 8] = [
    [0, 4, 8, 12],
    [1, 5, 9, 13],
    [2, 6, 10, 14],
    [3, 7, 11, 15],
    [0, 5, 10, 15],
    [1, 6, 11, 12],
    [2, 7, 8, 13],
    [3, 4, 9, 14],
];

// --- Helpers ---

/// Packs a u32 as a QM31 with limbs in the first two coordinates: `(low_u16, high_u16, 0, 0)`.
pub fn pack_u32(v: u32) -> QM31 {
    qm31_from_u32s(v & 0xFFFF, v >> 16, 0, 0)
}

/// Reconstructs a u32 from a packed-limbs QM31: `coord0 + coord1 * 65536`.
pub fn unpack_u32(v: QM31) -> u32 {
    v.0.0.0 + v.0.1.0 * 65536
}

/// Wraps a single M31 value as a QM31: `(value, 0, 0, 0)`.
pub fn u32_to_qm31(v: u32) -> QM31 {
    qm31_from_u32s(v, 0, 0, 0)
}

// --- Builder functions ---

type U32Var = U32Wrapper<Var>;

/// Adds a BlakeG gate. All inputs and outputs are [`U32Wrapper`] packed-limbs wires.
pub fn blake_g_gate(
    ctx: &mut TraceContext,
    a: U32Var,
    b: U32Var,
    c: U32Var,
    d: U32Var,
    m0: U32Var,
    m1: U32Var,
) -> (U32Var, U32Var, U32Var, U32Var) {
    let mut a_val = unpack_u32(ctx.get(*a.get()));
    let mut b_val = unpack_u32(ctx.get(*b.get()));
    let mut c_val = unpack_u32(ctx.get(*c.get()));
    let mut d_val = unpack_u32(ctx.get(*d.get()));
    let m0_val = unpack_u32(ctx.get(*m0.get()));
    let m1_val = unpack_u32(ctx.get(*m1.get()));

    a_val = a_val.wrapping_add(b_val).wrapping_add(m0_val);
    d_val = (d_val ^ a_val).rotate_right(16);
    c_val = c_val.wrapping_add(d_val);
    b_val = (b_val ^ c_val).rotate_right(12);
    a_val = a_val.wrapping_add(b_val).wrapping_add(m1_val);
    d_val = (d_val ^ a_val).rotate_right(8);
    c_val = c_val.wrapping_add(d_val);
    b_val = (b_val ^ c_val).rotate_right(7);

    let out_a = ctx.new_var(pack_u32(a_val));
    let out_b = ctx.new_var(pack_u32(b_val));
    let out_c = ctx.new_var(pack_u32(c_val));
    let out_d = ctx.new_var(pack_u32(d_val));

    ctx.stats.blake_g += 1;
    ctx.circuit.blake_g.push(BlakeGGate {
        a: a.get().idx,
        b: b.get().idx,
        c: c.get().idx,
        d: d.get().idx,
        m0: m0.get().idx,
        m1: m1.get().idx,
        out_a: out_a.idx,
        out_b: out_b.idx,
        out_c: out_c.idx,
        out_d: out_d.idx,
    });

    (
        U32Var::new_unsafe(out_a),
        U32Var::new_unsafe(out_b),
        U32Var::new_unsafe(out_c),
        U32Var::new_unsafe(out_d),
    )
}

/// Adds a TripleXor gate. All operands are [`U32Wrapper`] packed-limbs wires.
pub fn triple_xor_gate(ctx: &mut TraceContext, a: U32Var, b: U32Var, c: U32Var) -> U32Var {
    let result = unpack_u32(ctx.get(*a.get()))
        ^ unpack_u32(ctx.get(*b.get()))
        ^ unpack_u32(ctx.get(*c.get()));
    let out = ctx.new_var(pack_u32(result));

    ctx.stats.triple_xor += 1;
    ctx.circuit.triple_xor.push(TripleXorGate {
        a: a.get().idx,
        b: b.get().idx,
        c: c.get().idx,
        out: out.idx,
    });

    U32Var::new_unsafe(out)
}

/// Converts an M31 wire `(x, 0, 0, 0)` to a [`U32Wrapper`] packed-limbs wire
/// `(low_u16, high_u15, 0, 0)`.
pub fn m31_to_u32_gate(ctx: &mut TraceContext, input: Var) -> U32Var {
    let x = ctx.get(input).0.0.0;
    let out = ctx.new_var(pack_u32(x));

    ctx.stats.m31_to_u32 += 1;
    ctx.circuit.m31_to_u32.push(M31ToU32Gate { input: input.idx, out: out.idx });

    U32Var::new_unsafe(out)
}

/// Computes Blake2s hash using decomposed gates (BlakeG, TripleXor, M31ToU32) instead of the
/// monolithic Blake gate. Produces the same output as [`blake`].
pub fn blake_from_gates(ctx: &mut TraceContext, input: &[Var], n_bytes: usize) -> HashValue<Var> {
    assert_eq!(input.len(), n_bytes.div_ceil(16));

    // 1. Extract individual M31 message words from input QM31 vars, then convert each to U32Wrapper
    //    packed limbs for blake_g_gate.
    let mut message_u32s: Vec<U32Var> = Vec::new();
    for &var in input {
        let simd = Simd::from_packed(vec![var], 4);
        for coord in 0..4 {
            let comp = Simd::unpack_idx(ctx, &simd, coord);
            message_u32s.push(m31_to_u32_gate(ctx, comp));
        }
    }

    // 2. Pad message to complete 64-byte (16 u32-word) blocks.
    let n_blocks = std::cmp::max(1, n_bytes.div_ceil(64));
    let total_words = n_blocks * 16;
    let zero_u32 = U32Var::new_unsafe(ctx.zero());
    while message_u32s.len() < total_words {
        message_u32s.push(zero_u32);
    }

    // 3. Initialize chaining value: IV with parameter block XOR, as packed-limbs constants. All
    //    limb values are < 2^16 < P, so ctx.constant() is safe.
    let mut h: [U32Var; 8] = std::array::from_fn(|i| {
        let iv_val = if i == 0 { BLAKE2S_IV[0] ^ 0x01010020 } else { BLAKE2S_IV[i] };
        U32Var::new_unsafe(ctx.constant(pack_u32(iv_val)))
    });

    // 4. Compress each block.
    for block_idx in 0..n_blocks {
        let block: [U32Var; 16] = std::array::from_fn(|i| message_u32s[block_idx * 16 + i]);
        let t = std::cmp::min(n_bytes, (block_idx + 1) * 64) as u64;
        let last = block_idx == n_blocks - 1;

        let old_h = h;

        // Set up working vector: v[0..8] = h, v[8..16] = IV with counter/flag pre-XORed.
        let t_low = (t & 0xFFFFFFFF) as u32;
        let mut v: [U32Var; 16] = std::array::from_fn(|i| {
            if i < 8 {
                h[i]
            } else {
                let mut iv = BLAKE2S_IV[i - 8];
                if i == 12 {
                    iv ^= t_low;
                }
                if i == 14 && last {
                    iv ^= 0xFFFFFFFF;
                }
                U32Var::new_unsafe(ctx.constant(pack_u32(iv)))
            }
        });

        // 10 rounds of mixing.
        for s in &BLAKE_SIGMA {
            for g_idx in 0..8 {
                let [ai, bi, ci, di] = G_STATE_INDICES[g_idx];
                let (na, nb, nc, nd) = blake_g_gate(
                    ctx,
                    v[ai],
                    v[bi],
                    v[ci],
                    v[di],
                    block[s[g_idx * 2] as usize],
                    block[s[g_idx * 2 + 1] as usize],
                );
                v[ai] = na;
                v[bi] = nb;
                v[ci] = nc;
                v[di] = nd;
            }
        }

        // Finalize current compress: h[i] = old_h[i] ^ v[i] ^ v[i+8].
        for i in 0..8 {
            h[i] = triple_xor_gate(ctx, old_h[i], v[i], v[i + 8]);
        }
    }

    // 5. Apply reduce_to_m31: extract low/high from packed limbs via Simd::unpack_idx, then low +
    //    high * 65536 in M31 arithmetic naturally reduces mod P.
    let c65536 = ctx.constant(u32_to_qm31(65536));
    let reduced: [Var; 8] = std::array::from_fn(|i| {
        let h_simd = Simd::from_packed(vec![*h[i].get()], 2);
        let low = Simd::unpack_idx(ctx, &h_simd, 0);
        let high = Simd::unpack_idx(ctx, &h_simd, 1);
        eval!(ctx, (low) + ((high) * (c65536)))
    });

    // 6. Pack into 2 QM31 outputs.
    let out0 = from_partial_evals(ctx, [reduced[0], reduced[1], reduced[2], reduced[3]]);
    let out1 = from_partial_evals(ctx, [reduced[4], reduced[5], reduced[6], reduced[7]]);

    HashValue(out0, out1)
}
