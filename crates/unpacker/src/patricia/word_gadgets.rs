//! Limb-level gadgets over 256-bit words for the Patricia circuits.
//!
//! A [`Word256`] enters the circuit as sixteen guessed 16-bit limbs (each range-constrained at
//! finalization), recombined pairwise into the eight packed `u32` words that flow through
//! multisets and Blake gates. Limbs are what position arithmetic works on: field arithmetic on a
//! packed word never crosses the limb boundary, so doubling (`child = 2·parent + bit`), the
//! variable shift `bottom = parent·2^ℓ + edge_path` (design doc §4.2), the 251-bit hash
//! truncation and the additive edge length (§4.4) all guess their carries limb-wise, keeping
//! every field coordinate far below `M31::P`.
//!
//! Every gadget takes the concrete witness values it needs alongside the vars and computes its
//! guesses with wrapping integer arithmetic, so building a circuit over a *rejected* witness never
//! panics — the constraints simply fail. Gate emission depends only on structure, never on the
//! values, keeping the topology witness-independent.

use circuits::blake::HashValue;
use circuits::context::{Context, Var};
use circuits::eval;
use circuits::ivalue::{IValue, qm31_from_u32s};
use circuits::ops::{Guess, add, eq, guess, mul};
use circuits::wrappers::{U16Wrapper, U32Wrapper};

use super::reference::Word256;

#[cfg(test)]
#[path = "word_gadgets_test.rs"]
mod test;

/// 16-bit limbs in a 256-bit value; limb `i` is bits `16i..16(i+1)`.
pub const N_LIMBS: usize = 16;

/// A 256-bit value as sixteen range-constrained 16-bit limbs (limb 0 least significant) plus the
/// eight packed `u32` words derived from them (`word i = limb 2i + limb 2i+1 · 2^16`).
pub struct LimbedWord256 {
    pub limbs: [Var; N_LIMBS],
    pub words: [U32Wrapper<Var>; 8],
}

/// Guesses `value` as sixteen ranged limbs and derives the packed words.
pub fn guess_limbed<Value: IValue>(ctx: &mut Context<Value>, value: &Word256) -> LimbedWord256 {
    let limbs: [Var; N_LIMBS] = std::array::from_fn(|i| guess_u16(ctx, limb_of(value, i)));
    let words = pack_limbs(ctx, &limbs);
    LimbedWord256 { limbs, words }
}

/// Re-expresses already-derived packed `words` (e.g. a Blake digest) as ranged limbs, constraining
/// the recombination to equal the given words. Word values are read back from the context, so
/// `words` must be valid `u32` packings for any witness — Blake and truncation outputs are.
pub fn limbed_from_words<Value: IValue>(
    ctx: &mut Context<Value>,
    words: &[U32Wrapper<Var>; 8],
) -> LimbedWord256 {
    let value: Word256 = std::array::from_fn(|i| ctx.get(*words[i].get()).unpack_u32());
    let limbs: [Var; N_LIMBS] = std::array::from_fn(|i| guess_u16(ctx, limb_of(&value, i)));
    let packed = pack_limbs(ctx, &limbs);
    for (mine, given) in packed.iter().zip(words) {
        eq(ctx, *mine.get(), *given.get());
    }
    LimbedWord256 { limbs, words: *words }
}

/// Guesses a variable constrained to `{0, 1}` (`b · (b − 1) = 0`) holding `bit`.
pub fn guess_bool<Value: IValue>(ctx: &mut Context<Value>, bit: u32) -> Var {
    let b = guess(ctx, Value::from_qm31(qm31_from_u32s(bit, 0, 0, 0)));
    let one = ctx.one();
    let zero = ctx.zero();
    let prod = eval!(ctx, ((b) - (one)) * (b));
    eq(ctx, prod, zero);
    b
}

/// Guesses the low `n_bits` of `value` as boolean-constrained bits, LSB first.
pub fn guess_bits<Value: IValue>(ctx: &mut Context<Value>, value: u32, n_bits: usize) -> Vec<Var> {
    (0..n_bits).map(|i| guess_bool(ctx, (value >> i) & 1)).collect()
}

/// `Σ bits[i] · 2^i` — the scalar the bits decompose.
pub fn compose_bits<Value: IValue>(ctx: &mut Context<Value>, bits: &[Var]) -> Var {
    let mut acc = ctx.zero();
    for (i, bit) in bits.iter().enumerate() {
        let coefficient = ctx.constant(qm31_from_u32s(1 << i, 0, 0, 0));
        let term = mul(ctx, *bit, coefficient);
        acc = add(ctx, acc, term);
    }
    acc
}

/// Constrains `child = 2·parent + low_bit (mod 2^256)` limb-wise, guessing the carry chain from
/// `parent_value`/`bit_value`. The top carry is boolean but otherwise free — the relation is a
/// deterministic function of the inputs either way.
pub fn constrain_double_plus_bit<Value: IValue>(
    ctx: &mut Context<Value>,
    parent: &LimbedWord256,
    child: &LimbedWord256,
    low_bit: Var,
    parent_value: &Word256,
    bit_value: u32,
) {
    let two = ctx.constant(qm31_from_u32s(2, 0, 0, 0));
    let limb_base = ctx.constant(qm31_from_u32s(1 << 16, 0, 0, 0));
    let mut carry = low_bit;
    let mut carry_value = bit_value;
    for i in 0..N_LIMBS {
        let t_value = 2 * limb_of(parent_value, i) + carry_value;
        let t = eval!(ctx, ((two) * (parent.limbs[i])) + (carry));
        let carry_out = guess_bool(ctx, t_value >> 16);
        let rhs = eval!(ctx, (child.limbs[i]) + ((carry_out) * (limb_base)));
        eq(ctx, t, rhs);
        carry = carry_out;
        carry_value = t_value >> 16;
    }
}

/// Constrains `bottom = parent·2^ℓ + edge_path (mod 2^256)`, `ℓ = Σ l_bits[i]·2^i` with boolean
/// `l_bits` (8 bits, so `ℓ ≤ 255`): a limb-rotate by `ℓ / 16` via four select stages (zero-fill),
/// a scale by `2^(ℓ mod 16)` with per-limb lo/hi splits, then the edge-path add.
///
/// The final add has no carry chain: aligned inputs (`edge_path < 2^ℓ`, which
/// [`constrain_below_pow2`] enforces) are bit-disjoint per limb, and a dishonest overlap makes the
/// sum exceed `bottom`'s ranged limb, failing the equality.
pub fn constrain_shift_add<Value: IValue>(
    ctx: &mut Context<Value>,
    parent: &LimbedWord256,
    l_bits: &[Var; 8],
    edge_path: &LimbedWord256,
    bottom: &LimbedWord256,
    parent_value: &Word256,
    l_value: u32,
) {
    let zero = ctx.zero();
    let limb_base = ctx.constant(qm31_from_u32s(1 << 16, 0, 0, 0));

    // Rotate by ℓ / 16 limbs, one doubling stage per bit of the limb count.
    let mut limbs: Vec<Var> = parent.limbs.to_vec();
    let mut values: Vec<u32> = (0..N_LIMBS).map(|i| limb_of(parent_value, i)).collect();
    for k in 0..4 {
        let stride = 1 << k;
        let bit = l_bits[4 + k];
        let bit_value = (l_value >> (4 + k)) & 1;
        limbs = (0..N_LIMBS)
            .map(|i| {
                let shifted = if i >= stride { limbs[i - stride] } else { zero };
                select(ctx, bit, shifted, limbs[i])
            })
            .collect();
        values = (0..N_LIMBS)
            .map(|i| match (bit_value, i >= stride) {
                (1, true) => values[i - stride],
                (1, false) => 0,
                _ => values[i],
            })
            .collect();
    }

    // Scale by 2^(ℓ mod 16): per limb, split `limb · 2^r` (< 2^31) into lo/hi ranged halves and
    // recombine one limb up. The top limb's hi half falls off 2^256.
    let (pow2r, pow2r_value) = pow2_of_bits(ctx, &l_bits[0..4], l_value & 15);
    let mut carry_hi = zero;
    for i in 0..N_LIMBS {
        let t_value = values[i] * pow2r_value;
        let lo = guess_u16(ctx, t_value & 0xFFFF);
        let hi = guess_u16(ctx, t_value >> 16);
        let t = mul(ctx, limbs[i], pow2r);
        let split = eval!(ctx, (lo) + ((hi) * (limb_base)));
        eq(ctx, t, split);
        // bottom limb = shifted limb + edge-path limb, carry-free (see above).
        let shifted = add(ctx, lo, carry_hi);
        let sum = add(ctx, shifted, edge_path.limbs[i]);
        eq(ctx, sum, bottom.limbs[i]);
        carry_hi = hi;
    }
    let _ = carry_hi;
}

/// Constrains `edge_path < 2^ℓ` — the canonical alignment of an edge's compressed bits — by
/// guessing `g` with `edge_path + 1 + g = 2^ℓ` limb-wise. `2^ℓ`'s limbs come from a one-hot
/// 4-bit decoder over `ℓ / 16` times `2^(ℓ mod 16)`; the top carry is forced to zero so the
/// relation cannot be satisfied modulo `2^256`.
pub fn constrain_below_pow2<Value: IValue>(
    ctx: &mut Context<Value>,
    edge_path: &LimbedWord256,
    l_bits: &[Var; 8],
    edge_path_value: &Word256,
    l_value: u32,
) {
    let one = ctx.one();
    let limb_base = ctx.constant(qm31_from_u32s(1 << 16, 0, 0, 0));
    let (pow2r, _) = pow2_of_bits(ctx, &l_bits[0..4], l_value & 15);
    let hot = decoder16(ctx, &l_bits[4..8]);
    let g_values = below_pow2_witness(edge_path_value, l_value);

    let mut carry = one;
    let mut carry_value = 1u32;
    for i in 0..N_LIMBS {
        let pow_limb = mul(ctx, hot[i], pow2r);
        let g = guess_u16(ctx, g_values[i]);
        let t = eval!(ctx, ((edge_path.limbs[i]) + (g)) + (carry));
        let t_value = limb_of(edge_path_value, i) + g_values[i] + carry_value;
        if i + 1 < N_LIMBS {
            let carry_out = guess_bool(ctx, t_value >> 16);
            let rhs = eval!(ctx, (pow_limb) + ((carry_out) * (limb_base)));
            eq(ctx, t, rhs);
            carry = carry_out;
            carry_value = t_value >> 16;
        } else {
            // No top carry: the sum must hit 2^ℓ exactly, not 2^ℓ + 2^256.
            eq(ctx, t, pow_limb);
        }
    }
}

/// Truncates a Blake digest to 251 bits (design doc §4.4): splits the top word's high limb as
/// `mid11 + top5·2^11` and drops `top5`. Returns the truncated words; words 0–6 pass through.
///
/// Witness values are read back from the digest vars, which are valid `u32` packings for any
/// witness (they are Blake outputs).
pub fn truncate_251<Value: IValue>(
    ctx: &mut Context<Value>,
    digest: &HashValue<Var>,
) -> [U32Wrapper<Var>; 8] {
    let word7 = *digest[7].get();
    let value = ctx.get(word7).unpack_u32();
    let i_unit = ctx.constant(qm31_from_u32s(0, 1, 0, 0));
    let mid_base = ctx.constant(qm31_from_u32s(1 << 11, 0, 0, 0));

    let lo = guess_u16(ctx, value & 0xFFFF);
    let mid_bits = guess_bits(ctx, (value >> 16) & 0x7FF, 11);
    let mid = compose_bits(ctx, &mid_bits);
    let top_bits = guess_bits(ctx, value >> 27, 5);
    let top = compose_bits(ctx, &top_bits);

    let recombined = eval!(ctx, (lo) + ((((top) * (mid_base)) + (mid)) * (i_unit)));
    eq(ctx, word7, recombined);
    let truncated = eval!(ctx, (lo) + ((mid) * (i_unit)));

    std::array::from_fn(|i| if i < 7 { digest[i] } else { U32Wrapper::new_unsafe(truncated) })
}

/// `words + scalar (mod 2^256)` with a limb-wise carry chain; `scalar`'s value must be under
/// `2^16` (an edge length is at most 251). Word values are read back, so `words` must be valid
/// packings for any witness — Blake and truncation outputs are.
pub fn add_small_words<Value: IValue>(
    ctx: &mut Context<Value>,
    words: &[U32Wrapper<Var>; 8],
    scalar: Var,
    scalar_value: u32,
) -> [U32Wrapper<Var>; 8] {
    let limbed = limbed_from_words(ctx, words);
    let value: Word256 = std::array::from_fn(|i| ctx.get(*words[i].get()).unpack_u32());
    let limb_base = ctx.constant(qm31_from_u32s(1 << 16, 0, 0, 0));

    let mut carry = scalar;
    let mut carry_value = scalar_value;
    let out: [Var; N_LIMBS] = std::array::from_fn(|i| {
        let t_value = limb_of(&value, i) + carry_value;
        let t = add(ctx, limbed.limbs[i], carry);
        let out_limb = guess_u16(ctx, t_value & 0xFFFF);
        let carry_out = guess_bool(ctx, t_value >> 16);
        let rhs = eval!(ctx, (out_limb) + ((carry_out) * (limb_base)));
        eq(ctx, t, rhs);
        carry = carry_out;
        carry_value = t_value >> 16;
        out_limb
    });
    let _ = carry;
    pack_limbs(ctx, &out)
}

/// Guesses a range-constrained 16-bit limb holding `value`.
pub fn guess_u16<Value: IValue>(ctx: &mut Context<Value>, value: u32) -> Var {
    debug_assert!(value < 1 << 16, "limb witness {value} overflows 16 bits");
    let wrapper = U16Wrapper::new_unsafe(Value::from_qm31(qm31_from_u32s(value, 0, 0, 0)));
    *wrapper.guess(ctx).get()
}

/// Packs limb pairs into `u32` words: `word i = limbs[2i] + limbs[2i+1] · 2^16`.
fn pack_limbs<Value: IValue>(
    ctx: &mut Context<Value>,
    limbs: &[Var; N_LIMBS],
) -> [U32Wrapper<Var>; 8] {
    let i_unit = ctx.constant(qm31_from_u32s(0, 1, 0, 0));
    std::array::from_fn(|i| {
        let word = eval!(ctx, (limbs[2 * i]) + ((limbs[2 * i + 1]) * (i_unit)));
        U32Wrapper::new_unsafe(word)
    })
}

/// Limb `i` of a concrete [`Word256`].
fn limb_of(value: &Word256, i: usize) -> u32 {
    (value[i / 2] >> (16 * (i % 2))) & 0xFFFF
}

/// `flag ? when_one : when_zero`, assuming `flag ∈ {0, 1}`.
fn select<Value: IValue>(
    ctx: &mut Context<Value>,
    flag: Var,
    when_one: Var,
    when_zero: Var,
) -> Var {
    eval!(ctx, (when_zero) + ((flag) * ((when_one) - (when_zero))))
}

/// `2^(Σ bits[i]·2^i)` for up to four bits, as a product of per-bit factors
/// `1 + bit·(2^(2^k) − 1)`; also returns the concrete value (`r ≤ 15`, so it stays a limb).
fn pow2_of_bits<Value: IValue>(ctx: &mut Context<Value>, bits: &[Var], r_value: u32) -> (Var, u32) {
    assert!(bits.len() <= 4, "2^r would overflow a limb");
    let one = ctx.one();
    let mut acc = one;
    for (k, bit) in bits.iter().enumerate() {
        let step = ctx.constant(qm31_from_u32s((1u32 << (1 << k)) - 1, 0, 0, 0));
        let factor = eval!(ctx, ((*bit) * (step)) + (one));
        acc = mul(ctx, acc, factor);
    }
    (acc, 1 << r_value)
}

/// One-hot decode of a 4-bit index (LSB first): `hot[j] = 1` iff `Σ bits[k]·2^k = j`. A product
/// tensor, ~30 multiplications.
fn decoder16<Value: IValue>(ctx: &mut Context<Value>, bits: &[Var]) -> Vec<Var> {
    assert!(bits.len() == 4);
    let one = ctx.one();
    let mut hot = vec![one];
    for bit in bits {
        let not_bit = eval!(ctx, (one) - (*bit));
        let mut grown = Vec::with_capacity(2 * hot.len());
        for h in &hot {
            grown.push(mul(ctx, *h, not_bit));
        }
        for h in &hot {
            grown.push(mul(ctx, *h, *bit));
        }
        hot = grown;
    }
    hot
}

/// `2^ℓ − 1 − edge_path` limb-wise, wrapping mod `2^256` (garbage when `edge_path ≥ 2^ℓ`, by
/// design — the constraints then fail).
fn below_pow2_witness(edge_path_value: &Word256, l_value: u32) -> [u32; N_LIMBS] {
    let (ep_lo, ep_hi) = u128_pair(edge_path_value);
    let mut bound = [0u32; 8];
    bound[(l_value as usize / 32) % 8] = 1u32 << (l_value % 32);
    let (bound_lo, bound_hi) = u128_pair(&bound);

    // bound − (edge_path + 1), wrapping mod 2^256 over two u128 halves.
    let rhs_lo = ep_lo.wrapping_add(1);
    let rhs_hi = ep_hi.wrapping_add(u128::from(rhs_lo == 0));
    let out_lo = bound_lo.wrapping_sub(rhs_lo);
    let borrow = u128::from(bound_lo < rhs_lo);
    let out_hi = bound_hi.wrapping_sub(rhs_hi).wrapping_sub(borrow);

    std::array::from_fn(|i| {
        let half = if i < 8 { out_lo } else { out_hi };
        ((half >> (16 * (i % 8))) & 0xFFFF) as u32
    })
}

/// A [`Word256`] as its low and high 128-bit halves.
fn u128_pair(value: &Word256) -> (u128, u128) {
    let half =
        |words: &[u32]| words.iter().rev().fold(0u128, |acc, word| (acc << 32) | u128::from(*word));
    (half(&value[0..4]), half(&value[4..8]))
}
