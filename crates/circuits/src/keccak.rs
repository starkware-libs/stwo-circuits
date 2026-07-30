//! Keccak-f\[1600\] built entirely from existing gates — [`TripleXor`](crate::circuit::TripleXor),
//! field arithmetic and U16 range checks — with no dedicated AIR component.
//!
//! A 64-bit lane is held as two `u32` variables (see [`U64Lane`]). XORs go through TripleXor
//! gates, χ's AND is computed with the identity `x & y = (x + y - (x ^ y)) / 2` (valid per
//! 16-bit limb), and rotations split each 16-bit limb at the rotation offset into guessed,
//! range-checked pieces.

use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use crate::blake::triple_xor;
use crate::context::{Context, Var};
use crate::eval;
use crate::ivalue::{IValue, NoValue, qm31_from_u32s};
use crate::ops::{Guess, eq};
use crate::simd::Simd;
use crate::wrappers::{U16Wrapper, U32Wrapper};

#[cfg(test)]
#[path = "keccak_test.rs"]
pub mod test;

/// Number of 64-bit lanes in the Keccak-f\[1600\] state (5x5). Lane `(x, y)` is at index
/// `x + 5 * y`.
pub const KECCAK_STATE_N_LANES: usize = 25;

/// Number of rounds in Keccak-f\[1600\].
pub const KECCAK_N_ROUNDS: usize = 24;

/// Round constants (ι step).
pub const KECCAK_RC: [u64; KECCAK_N_ROUNDS] = [
    0x0000000000000001,
    0x0000000000008082,
    0x800000000000808A,
    0x8000000080008000,
    0x000000000000808B,
    0x0000000080000001,
    0x8000000080008081,
    0x8000000000008009,
    0x000000000000008A,
    0x0000000000000088,
    0x0000000080008009,
    0x000000008000000A,
    0x000000008000808B,
    0x800000000000008B,
    0x8000000000008089,
    0x8000000000008003,
    0x8000000000008002,
    0x8000000000000080,
    0x000000000000800A,
    0x800000008000000A,
    0x8000000080008081,
    0x8000000000008080,
    0x0000000080000001,
    0x8000000080008008,
];

/// Left-rotation offsets (ρ step), indexed `[x][y]`.
pub const KECCAK_RHO: [[u32; 5]; 5] = [
    [0, 36, 3, 41, 18],
    [1, 44, 10, 45, 2],
    [62, 6, 43, 15, 61],
    [28, 55, 25, 21, 56],
    [27, 20, 39, 8, 14],
];

/// A 64-bit Keccak lane held as two `u32` values `(low, high)`, each encoded as a QM31
/// `(low_u16, high_u16, 0, 0)`.
///
/// `T` is the per-element representation: `QM31` for concrete witness values, or [`Var`] for
/// variables inside a [`Context`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct U64Lane<T> {
    pub low: U32Wrapper<T>,
    pub high: U32Wrapper<T>,
}

impl From<u64> for U64Lane<QM31> {
    fn from(value: u64) -> Self {
        U64Lane {
            low: U32Wrapper::new_unsafe(QM31::pack_u32(value as u32)),
            high: U32Wrapper::new_unsafe(QM31::pack_u32((value >> 32) as u32)),
        }
    }
}

impl From<NoValue> for U64Lane<NoValue> {
    fn from(_: NoValue) -> Self {
        U64Lane { low: U32Wrapper::new_unsafe(NoValue), high: U32Wrapper::new_unsafe(NoValue) }
    }
}

impl U64Lane<Var> {
    /// Reads the lane's concrete `u64` value from the context.
    pub fn value<Value: IValue>(&self, ctx: &Context<Value>) -> u64 {
        (ctx.get(*self.low.get()).unpack_u32() as u64)
            | ((ctx.get(*self.high.get()).unpack_u32() as u64) << 32)
    }

    /// Packs the lane into a single wire holding its four 16-bit limbs, one per M31
    /// coordinate, least significant first: `(low_u16, low_hi16, high_u16, high_hi16)`.
    pub fn pack_to_qm31<Value: IValue>(&self, ctx: &mut Context<Value>) -> Var {
        let high_simd = Simd::from_packed(vec![*self.high.get()], 2);
        let h0 = Simd::unpack_idx(ctx, &high_simd, 0);
        let h1 = Simd::unpack_idx(ctx, &high_simd, 1);
        let u = ctx.constant(qm31_from_u32s(0, 0, 1, 0));
        let iu = ctx.constant(qm31_from_u32s(0, 0, 0, 1));
        let low = *self.low.get();
        eval!(ctx, ((low) + ((h0) * (u))) + ((h1) * (iu)))
    }

    /// Builds a lane from its four 16-bit limbs (little-endian), one single-coordinate wire
    /// each.
    ///
    /// The caller must supply wires already constrained to `[0, 2^16)` (e.g. constants or
    /// guessed [`U16Wrapper`]s); otherwise the resulting `u32` halves are not validly encoded
    /// and the gates consuming them will reject.
    pub fn from_limbs<Value: IValue>(ctx: &mut Context<Value>, limbs: [Var; 4]) -> Self {
        pack_lane(ctx, limbs)
    }

    /// Unpacks the lane into its four 16-bit limbs (little-endian), one single-coordinate
    /// wire each.
    pub fn to_limbs<Value: IValue>(&self, ctx: &mut Context<Value>) -> [Var; 4] {
        let low_simd = Simd::from_packed(vec![*self.low.get()], 2);
        let high_simd = Simd::from_packed(vec![*self.high.get()], 2);
        [
            Simd::unpack_idx(ctx, &low_simd, 0),
            Simd::unpack_idx(ctx, &low_simd, 1),
            Simd::unpack_idx(ctx, &high_simd, 0),
            Simd::unpack_idx(ctx, &high_simd, 1),
        ]
    }
}

impl<Value: IValue> Guess<Value> for U64Lane<Value> {
    type Target = U64Lane<Var>;

    /// Guesses the two `u32` halves via [`U32Wrapper`]'s guess, so each 16-bit limb is
    /// range-constrained to `[0, 2^16)`.
    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        U64Lane { low: self.low.guess(context), high: self.high.guess(context) }
    }
}

/// Adds gates computing the Keccak-f\[1600\] permutation on a 5x5 state of 64-bit lanes
/// (lane `(x, y)` at index `x + 5 * y`).
///
/// The input lanes must be valid `u32`-encoded pairs, e.g. produced by [`U64Lane`]'s guess or
/// by other gates; the output lanes are in the same encoding.
pub fn keccak_f1600<Value: IValue>(
    ctx: &mut Context<Value>,
    mut state: [U64Lane<Var>; KECCAK_STATE_N_LANES],
) -> [U64Lane<Var>; KECCAK_STATE_N_LANES] {
    for rc in KECCAK_RC {
        state = keccak_round(ctx, state, rc);
    }
    state
}

/// Rate of Keccak-256 in bytes: `200 - 2 * 32`.
pub const KECCAK256_RATE_BYTES: usize = 136;
/// Rate in 64-bit lanes.
const RATE_LANES: usize = KECCAK256_RATE_BYTES / 8;
/// Rate in 16-bit limbs.
const RATE_LIMBS: usize = KECCAK256_RATE_BYTES / 2;
/// Number of lanes in a Keccak-256 digest (256 bits / 64).
pub const KECCAK256_DIGEST_N_LANES: usize = 4;

/// Adds gates computing `keccak256(message)`, returning the digest's four 64-bit lanes
/// (little-endian byte order inside each lane, lane 0 covering digest bytes 0..8).
///
/// The message is given as 16-bit limbs in Keccak's little-endian byte order — limb `j` holds
/// message bytes `2j` (low) and `2j + 1` (high) — so `n_bytes` must be even and
/// `message_limbs.len() == n_bytes / 2`. Each limb wire must already be constrained to
/// `[0, 2^16)`; [`keccak256_bytes`] does that for a concrete message.
///
/// This is the standard Keccak-256 of the NIST-independent (pre-SHA3) domain, i.e. Ethereum's
/// `keccak256`, using the `0x01` domain suffix.
pub fn keccak256_limbs<Value: IValue>(
    ctx: &mut Context<Value>,
    message_limbs: &[Var],
    n_bytes: usize,
) -> [U64Lane<Var>; KECCAK256_DIGEST_N_LANES] {
    assert_eq!(n_bytes % 2, 0, "keccak256_limbs requires an even byte length");
    assert_eq!(message_limbs.len(), n_bytes / 2, "limb count must match the byte length");

    // Pad to a whole number of rate blocks: `0x01` right after the message, `0x80` in the last
    // byte of the block (they coincide in one byte only if a single pad byte remains, which an
    // even `n_bytes` never produces since the last block byte is at an odd offset).
    let n_blocks = (n_bytes + 1).div_ceil(KECCAK256_RATE_BYTES);
    let padded_bytes = n_blocks * KECCAK256_RATE_BYTES;
    let mut pad = vec![0u8; padded_bytes - n_bytes];
    pad[0] |= 0x01;
    *pad.last_mut().unwrap() |= 0x80;
    let pad_limbs: Vec<Var> = pad
        .chunks(2)
        .map(|pair| {
            ctx.constant(qm31_from_u32s(u16::from_le_bytes([pair[0], pair[1]]) as u32, 0, 0, 0))
        })
        .collect();
    let padded_limbs: Vec<Var> = message_limbs.iter().copied().chain(pad_limbs).collect();
    debug_assert_eq!(padded_limbs.len(), n_blocks * RATE_LIMBS);

    let zero_u32 = ctx.constant(QM31::pack_u32(0));
    let zero_lane =
        U64Lane { low: U32Wrapper::new_unsafe(zero_u32), high: U32Wrapper::new_unsafe(zero_u32) };
    let mut state = [zero_lane; KECCAK_STATE_N_LANES];

    for block in 0..n_blocks {
        let block_limbs = &padded_limbs[block * RATE_LIMBS..(block + 1) * RATE_LIMBS];
        for lane_idx in 0..RATE_LANES {
            let lane =
                U64Lane::from_limbs(ctx, std::array::from_fn(|k| block_limbs[4 * lane_idx + k]));
            // The first absorb XORs into the all-zero state, so the block lanes pass straight
            // through — no gates needed.
            state[lane_idx] = if block == 0 { lane } else { lane_xor2(ctx, state[lane_idx], lane) };
        }
        state = keccak_f1600(ctx, state);
    }

    std::array::from_fn(|i| state[i])
}

/// Adds gates computing `keccak256` of a concrete message, guessing the message limbs as
/// range-checked [`U16Wrapper`]s.
///
/// Only useful when the message is witness data with no other in-circuit provenance; when the
/// message is itself derived from circuit wires, build those limbs and call
/// [`keccak256_limbs`].
pub fn keccak256_bytes<Value: IValue>(
    ctx: &mut Context<Value>,
    message: &[u8],
) -> [U64Lane<Var>; KECCAK256_DIGEST_N_LANES] {
    assert_eq!(message.len() % 2, 0, "keccak256_bytes requires an even byte length");
    let limbs: Vec<Var> = message
        .chunks(2)
        .map(|pair| guess_u16(ctx, u16::from_le_bytes([pair[0], pair[1]]) as u32))
        .collect();
    keccak256_limbs(ctx, &limbs, message.len())
}

fn keccak_round<Value: IValue>(
    ctx: &mut Context<Value>,
    state: [U64Lane<Var>; KECCAK_STATE_N_LANES],
    rc: u64,
) -> [U64Lane<Var>; KECCAK_STATE_N_LANES] {
    // θ: XOR each lane with `d[x]`, derived from the column parities `c`.
    let c: [U64Lane<Var>; 5] =
        std::array::from_fn(|x| lane_xor5(ctx, std::array::from_fn(|y| state[x + 5 * y])));
    let d: [U64Lane<Var>; 5] = std::array::from_fn(|x| {
        let rotated = lane_rotl(ctx, c[(x + 1) % 5], 1);
        lane_xor2(ctx, c[(x + 4) % 5], rotated)
    });
    let a: [U64Lane<Var>; KECCAK_STATE_N_LANES] =
        std::array::from_fn(|i| lane_xor2(ctx, state[i], d[i % 5]));

    // ρ and π: rotate each lane by its fixed offset and permute the lanes.
    let mut b = [None; KECCAK_STATE_N_LANES];
    for x in 0..5 {
        for y in 0..5 {
            b[y + 5 * ((2 * x + 3 * y) % 5)] = Some(lane_rotl(ctx, a[x + 5 * y], KECCAK_RHO[x][y]));
        }
    }
    let b = b.map(|lane| lane.unwrap());

    // χ, with ι (the round constant) folded into lane (0, 0)'s final XOR.
    let zero = ctx.zero();
    let rc_low = ctx.constant(QM31::pack_u32(rc as u32));
    let rc_high = ctx.constant(QM31::pack_u32((rc >> 32) as u32));
    std::array::from_fn(|i| {
        let (x, y) = (i % 5, i / 5);
        let b0 = b[i];
        let b1 = b[(x + 1) % 5 + 5 * y];
        let b2 = b[(x + 2) % 5 + 5 * y];
        let (rc_l, rc_h) = if i == 0 { (rc_low, rc_high) } else { (zero, zero) };
        U64Lane {
            low: U32Wrapper::new_unsafe(chi_half(
                ctx,
                *b0.low.get(),
                *b1.low.get(),
                *b2.low.get(),
                rc_l,
            )),
            high: U32Wrapper::new_unsafe(chi_half(
                ctx,
                *b0.high.get(),
                *b1.high.get(),
                *b2.high.get(),
                rc_h,
            )),
        }
    })
}

/// XOR of two `u32`-encoded values via a TripleXor gate with a zero third input.
fn xor2<Value: IValue>(ctx: &mut Context<Value>, a: Var, b: Var) -> Var {
    let zero = ctx.zero();
    triple_xor(ctx, a, b, zero)
}

fn lane_xor2<Value: IValue>(
    ctx: &mut Context<Value>,
    a: U64Lane<Var>,
    b: U64Lane<Var>,
) -> U64Lane<Var> {
    U64Lane {
        low: U32Wrapper::new_unsafe(xor2(ctx, *a.low.get(), *b.low.get())),
        high: U32Wrapper::new_unsafe(xor2(ctx, *a.high.get(), *b.high.get())),
    }
}

fn lane_xor5<Value: IValue>(ctx: &mut Context<Value>, lanes: [U64Lane<Var>; 5]) -> U64Lane<Var> {
    let xor5 = |ctx: &mut Context<Value>, halves: [Var; 5]| {
        let t = triple_xor(ctx, halves[0], halves[1], halves[2]);
        triple_xor(ctx, t, halves[3], halves[4])
    };
    let low = xor5(ctx, lanes.map(|lane| *lane.low.get()));
    let high = xor5(ctx, lanes.map(|lane| *lane.high.get()));
    U64Lane { low: U32Wrapper::new_unsafe(low), high: U32Wrapper::new_unsafe(high) }
}

/// Computes `a ^ ((!b) & c) ^ rc` on `u32`-encoded values, using
/// `b & c = (b + c - (b ^ c)) / 2` and `(!b) & c = c - (b & c)`.
///
/// The identities hold per 16-bit limb without field wraparound because all limbs are < 2^16.
fn chi_half<Value: IValue>(ctx: &mut Context<Value>, a: Var, b: Var, c: Var, rc: Var) -> Var {
    let b_xor_c = xor2(ctx, b, c);
    let inv2 = ctx.constant(M31::from(2).inverse().into());
    let b_and_c = eval!(ctx, (((b) + (c)) - (b_xor_c)) * (inv2));
    let not_b_and_c = eval!(ctx, (c) - (b_and_c));
    triple_xor(ctx, a, not_b_and_c, rc)
}

/// Adds gates computing the 64-bit left-rotation of a lane by `r` bits.
///
/// The rotation is decomposed as `r = 16 * q + s`: each 16-bit limb is split at bit `16 - s`
/// into range-checked pieces, the pieces are recombined into rotated-by-`s` limbs, and the
/// remaining `16 * q` rotation is a pure limb permutation (free rewiring).
fn lane_rotl<Value: IValue>(ctx: &mut Context<Value>, lane: U64Lane<Var>, r: u32) -> U64Lane<Var> {
    assert!(r < 64);
    if r == 0 {
        return lane;
    }
    let (q, s) = ((r / 16) as usize, r % 16);

    // The four 16-bit limbs, little-endian.
    let low_simd = Simd::from_packed(vec![*lane.low.get()], 2);
    let high_simd = Simd::from_packed(vec![*lane.high.get()], 2);
    let limbs: [Var; 4] = [
        Simd::unpack_idx(ctx, &low_simd, 0),
        Simd::unpack_idx(ctx, &low_simd, 1),
        Simd::unpack_idx(ctx, &high_simd, 0),
        Simd::unpack_idx(ctx, &high_simd, 1),
    ];

    let rotated: [Var; 4] = if s == 0 {
        limbs
    } else {
        // Split each limb `l_i = a_i + b_i * 2^(16-s)`. Rotating left by `s` moves each limb's
        // high `s` bits into the next limb (limb 3's bits wrap around to limb 0), so rotated
        // limb `i` is `a_i * 2^s + b_{i-1}`.
        let splits = limbs.map(|limb| split_limb(ctx, limb, s));
        std::array::from_fn(|i| {
            let (a_shifted, _) = splits[i];
            let (_, b_prev) = splits[(i + 3) % 4];
            eval!(ctx, (a_shifted) + (b_prev))
        })
    };

    // Rotating by the remaining `16 * q` bits permutes the limbs.
    pack_lane(ctx, std::array::from_fn(|i| rotated[(i + 4 - q) % 4]))
}

/// Splits a 16-bit limb `l = a + b * 2^(16-s)` (`s` in `1..16`) into its low `16 - s` bits `a`
/// and high `s` bits `b`, and returns `(a * 2^s, b)`.
///
/// All four committed pieces are range-checked to `[0, 2^16)` as guessed [U16Wrapper]s;
/// `a * 2^s < 2^16` then bounds `a < 2^(16-s)`, and `b * 2^(16-s) < 2^16` bounds `b < 2^s`
/// (neither product can wrap around the field, both being < 2^31 - 1), so the recomposition
/// `l = a + b * 2^(16-s)` is a genuine bit split.
fn split_limb<Value: IValue>(ctx: &mut Context<Value>, limb: Var, s: u32) -> (Var, Var) {
    debug_assert!((1..16).contains(&s));
    let value = ctx.get(limb).unpack_u32();
    let a_value = value & ((1 << (16 - s)) - 1);
    let b_value = value >> (16 - s);

    let a = guess_u16(ctx, a_value);
    let a_shifted = guess_u16(ctx, a_value << s);
    let b = guess_u16(ctx, b_value);
    let b_shifted = guess_u16(ctx, b_value << (16 - s));

    let pow_s = ctx.constant(M31::from(1u32 << s).into());
    let pow_16_minus_s = ctx.constant(M31::from(1u32 << (16 - s)).into());
    let a_times_pow = eval!(ctx, (a) * (pow_s));
    eq(ctx, a_times_pow, a_shifted);
    let b_times_pow = eval!(ctx, (b) * (pow_16_minus_s));
    eq(ctx, b_times_pow, b_shifted);
    let recomposed = eval!(ctx, (a) + (b_shifted));
    eq(ctx, recomposed, limb);

    (a_shifted, b)
}

/// Guesses a single-coordinate value that is range-checked to `[0, 2^16)` at finalization.
fn guess_u16<Value: IValue>(ctx: &mut Context<Value>, value: u32) -> Var {
    debug_assert!(value <= 0xFFFF);
    *U16Wrapper::new_unsafe(Value::from_qm31(qm31_from_u32s(value, 0, 0, 0))).guess(ctx).get()
}

/// Packs four 16-bit limbs (little-endian) back into a lane's two `u32` halves:
/// `(l0 + l1 * i, l2 + l3 * i)`.
fn pack_lane<Value: IValue>(ctx: &mut Context<Value>, limbs: [Var; 4]) -> U64Lane<Var> {
    let i_c = ctx.constant(qm31_from_u32s(0, 1, 0, 0));
    let low = eval!(ctx, (limbs[0]) + ((limbs[1]) * (i_c)));
    let high = eval!(ctx, (limbs[2]) + ((limbs[3]) * (i_c)));
    U64Lane { low: U32Wrapper::new_unsafe(low), high: U32Wrapper::new_unsafe(high) }
}

/// Plain-Rust Keccak-256 (Ethereum's `keccak256`, `0x01` domain suffix), for cross-checking
/// the in-circuit gadget.
pub fn keccak256_u8(message: &[u8]) -> [u8; 32] {
    let n_blocks = (message.len() + 1).div_ceil(KECCAK256_RATE_BYTES);
    let mut padded = vec![0u8; n_blocks * KECCAK256_RATE_BYTES];
    padded[..message.len()].copy_from_slice(message);
    padded[message.len()] |= 0x01;
    padded[n_blocks * KECCAK256_RATE_BYTES - 1] |= 0x80;

    let mut state = [0u64; KECCAK_STATE_N_LANES];
    for block in padded.chunks(KECCAK256_RATE_BYTES) {
        for (lane, chunk) in state.iter_mut().zip(block.chunks(8)) {
            *lane ^= u64::from_le_bytes(chunk.try_into().unwrap());
        }
        keccak_f1600_u64(&mut state);
    }

    let mut digest = [0u8; 32];
    for (i, lane) in state[..KECCAK256_DIGEST_N_LANES].iter().enumerate() {
        digest[8 * i..8 * (i + 1)].copy_from_slice(&lane.to_le_bytes());
    }
    digest
}

/// Plain-Rust Keccak-f\[1600\] on a 5x5 lane state (lane `(x, y)` at index `x + 5 * y`).
pub fn keccak_f1600_u64(state: &mut [u64; KECCAK_STATE_N_LANES]) {
    for rc in KECCAK_RC {
        // θ
        let c: [u64; 5] = std::array::from_fn(|x| {
            state[x] ^ state[x + 5] ^ state[x + 10] ^ state[x + 15] ^ state[x + 20]
        });
        let d: [u64; 5] = std::array::from_fn(|x| c[(x + 4) % 5] ^ c[(x + 1) % 5].rotate_left(1));
        for y in 0..5 {
            for x in 0..5 {
                state[x + 5 * y] ^= d[x];
            }
        }
        // ρ and π
        let mut b = [0u64; KECCAK_STATE_N_LANES];
        for x in 0..5 {
            for y in 0..5 {
                b[y + 5 * ((2 * x + 3 * y) % 5)] = state[x + 5 * y].rotate_left(KECCAK_RHO[x][y]);
            }
        }
        // χ
        for y in 0..5 {
            for x in 0..5 {
                state[x + 5 * y] =
                    b[x + 5 * y] ^ (!b[(x + 1) % 5 + 5 * y] & b[(x + 2) % 5 + 5 * y]);
            }
        }
        // ι
        state[0] ^= rc;
    }
}
