use circuits::context::TraceContext;
use circuits::ivalue::IValue;
use rstest::rstest;

use super::{
    LimbedWord256, N_LIMBS, add_small_words, compose_bits, constrain_below_pow2,
    constrain_double_plus_bit, constrain_shift_add, guess_bits, guess_limbed, truncate_251,
};
use crate::patricia::reference::Word256;

/// A value exercising limb boundaries: alternating all-ones and sparse limbs.
const BUMPY: Word256 =
    [0xFFFF_0001, 0x0000_FFFF, 0xDEAD_BEEF, 0xFFFF_FFFF, 0, 0x8000_0001, 0x7FFF_FFFF, 0x00FF_00FF];

fn unpack(ctx: &TraceContext, limbed: &LimbedWord256) -> Word256 {
    std::array::from_fn(|i| ctx.get(*limbed.words[i].get()).unpack_u32())
}

/// `value · 2^shift + addend (mod 2^256)` over u128 halves — the reference for the shift gadget.
fn shl_add(value: &Word256, shift: u32, addend: &Word256) -> Word256 {
    let half =
        |words: &[u32]| words.iter().rev().fold(0u128, |acc, word| (acc << 32) | u128::from(*word));
    let (mut lo, mut hi) = (half(&value[0..4]), half(&value[4..8]));
    for _ in 0..shift {
        hi = (hi << 1) | (lo >> 127);
        lo <<= 1;
    }
    let (add_lo, add_hi) = (half(&addend[0..4]), half(&addend[4..8]));
    let sum_lo = lo.wrapping_add(add_lo);
    hi = hi.wrapping_add(add_hi).wrapping_add(u128::from(sum_lo < lo));
    std::array::from_fn(|i| {
        let source = if i < 4 { sum_lo } else { hi };
        ((source >> (32 * (i % 4))) & 0xFFFF_FFFF) as u32
    })
}

#[test]
fn limbed_words_round_trip() {
    let mut ctx = TraceContext::default();
    let limbed = guess_limbed(&mut ctx, &BUMPY);
    assert!(unpack(&ctx, &limbed) == BUMPY);
    assert!(ctx.is_circuit_valid());
}

#[test]
fn bits_compose_back() {
    let mut ctx = TraceContext::default();
    let bits = guess_bits(&mut ctx, 0b1011_0110, 8);
    let composed = compose_bits(&mut ctx, &bits);
    assert!(ctx.get(composed).unpack_u32() == 0b1011_0110);
    assert!(ctx.is_circuit_valid());
}

#[rstest]
#[case(0)]
#[case(1)]
fn double_plus_bit_holds(#[case] bit: u32) {
    let mut ctx = TraceContext::default();
    let expected = shl_add(&BUMPY, 1, &[bit, 0, 0, 0, 0, 0, 0, 0]);
    let parent = guess_limbed(&mut ctx, &BUMPY);
    let child = guess_limbed(&mut ctx, &expected);
    let low_bit = super::guess_bool(&mut ctx, bit);
    constrain_double_plus_bit(&mut ctx, &parent, &child, low_bit, &BUMPY, bit);
    assert!(ctx.is_circuit_valid());
}

#[test]
fn double_rejects_a_wrong_child() {
    let mut ctx = TraceContext::default();
    let mut wrong = shl_add(&BUMPY, 1, &[0; 8]);
    wrong[3] ^= 1 << 16;
    let parent = guess_limbed(&mut ctx, &BUMPY);
    let child = guess_limbed(&mut ctx, &wrong);
    let zero = ctx.zero();
    constrain_double_plus_bit(&mut ctx, &parent, &child, zero, &BUMPY, 0);
    assert!(!ctx.is_circuit_valid());
}

/// Shift amounts covering the sub-limb, limb-boundary, word-boundary and near-top cases.
#[rstest]
#[case(1)]
#[case(15)]
#[case(16)]
#[case(17)]
#[case(31)]
#[case(32)]
#[case(100)]
#[case(251)]
fn shift_add_matches_the_reference(#[case] shift: u32) {
    let mut ctx = TraceContext::default();
    // A short parent (fits any shift) and an aligned edge path below 2^shift.
    let parent = [0b1011, 0, 0, 0, 0, 0, 0, 0];
    let mut edge_path = [0u32; 8];
    edge_path[(shift as usize - 1) / 32] = 1 << ((shift - 1) % 32);
    let expected = shl_add(&parent, shift, &edge_path);

    let parent_limbed = guess_limbed(&mut ctx, &parent);
    let edge_limbed = guess_limbed(&mut ctx, &edge_path);
    let bottom = guess_limbed(&mut ctx, &expected);
    let l_bits: [_; 8] = guess_bits(&mut ctx, shift, 8).try_into().expect("eight bits");
    constrain_shift_add(&mut ctx, &parent_limbed, &l_bits, &edge_limbed, &bottom, &parent, shift);
    assert!(ctx.is_circuit_valid(), "shift by {shift} rejected a correct bottom");
}

#[test]
fn shift_add_rejects_a_wrong_bottom() {
    let mut ctx = TraceContext::default();
    let parent = [0b1011, 0, 0, 0, 0, 0, 0, 0];
    let mut wrong = shl_add(&parent, 20, &[0; 8]);
    wrong[0] ^= 1;
    let parent_limbed = guess_limbed(&mut ctx, &parent);
    let edge_limbed = guess_limbed(&mut ctx, &[0; 8]);
    let bottom = guess_limbed(&mut ctx, &wrong);
    let l_bits: [_; 8] = guess_bits(&mut ctx, 20, 8).try_into().expect("eight bits");
    constrain_shift_add(&mut ctx, &parent_limbed, &l_bits, &edge_limbed, &bottom, &parent, 20);
    assert!(!ctx.is_circuit_valid());
}

#[rstest]
#[case(0, 0, true)] // 2^0 = 1: only edge_path = 0 fits
#[case(1, 1, true)]
#[case(1, 2, false)]
#[case(16, 0xFFFF, true)] // 2^16 − 1, the full first limb
#[case(16, 0x10000, false)]
#[case(251, 0xFFFF_FFFF, true)]
fn below_pow2_bounds_the_edge_path(#[case] length: u32, #[case] low: u32, #[case] valid: bool) {
    let mut ctx = TraceContext::default();
    let edge_path = [low, 0, 0, 0, 0, 0, 0, 0];
    let edge_limbed = guess_limbed(&mut ctx, &edge_path);
    let l_bits: [_; 8] = guess_bits(&mut ctx, length, 8).try_into().expect("eight bits");
    constrain_below_pow2(&mut ctx, &edge_limbed, &l_bits, &edge_path, length);
    assert!(ctx.is_circuit_valid() == valid, "ℓ = {length}, edge_path = {low:#x}");
}

/// `edge_path = 2^ℓ` exactly — one past the bound, at the bit the decoder selects.
#[test]
fn below_pow2_rejects_the_bound_itself() {
    let mut ctx = TraceContext::default();
    let length = 100u32;
    let mut edge_path = [0u32; 8];
    edge_path[(length as usize) / 32] = 1 << (length % 32);
    let edge_limbed = guess_limbed(&mut ctx, &edge_path);
    let l_bits: [_; 8] = guess_bits(&mut ctx, length, 8).try_into().expect("eight bits");
    constrain_below_pow2(&mut ctx, &edge_limbed, &l_bits, &edge_path, length);
    assert!(!ctx.is_circuit_valid());
}

#[rstest]
#[case([0xFFFF_FFFF; 8])] // every truncated bit set
#[case([0, 0, 0, 0, 0, 0, 0, 0x07FF_FFFF])] // exactly the 251-bit boundary
#[case(BUMPY)]
fn truncation_masks_to_251_bits(#[case] value: Word256) {
    use circuits::blake::HashValue;
    use circuits::ops::Guess;
    use stwo::core::fields::qm31::QM31;

    let mut ctx = TraceContext::default();
    let digest = HashValue::<QM31>::from(value).guess(&mut ctx);
    let truncated = truncate_251(&mut ctx, &digest);
    let result: Word256 = std::array::from_fn(|i| ctx.get(*truncated[i].get()).unpack_u32());
    let mut expected = value;
    expected[7] &= (1 << 27) - 1;
    assert!(result == expected);
    assert!(ctx.is_circuit_valid());
}

#[rstest]
#[case([0; 8], 251)]
#[case([0xFFFF_FFFF, 0xFFFF_FFFF, 0, 0, 0, 0, 0, 0], 1)] // carry ripples across two words
#[case(BUMPY, 200)]
fn add_small_ripples_the_carry(#[case] value: Word256, #[case] scalar: u32) {
    let mut ctx = TraceContext::default();
    let words = guess_limbed(&mut ctx, &value).words;
    let scalar_var = ctx.constant(circuits::ivalue::qm31_from_u32s(scalar, 0, 0, 0));
    let sum = add_small_words(&mut ctx, &words, scalar_var, scalar);
    let result: Word256 = std::array::from_fn(|i| ctx.get(*sum[i].get()).unpack_u32());
    let expected = shl_add(&value, 0, &[scalar, 0, 0, 0, 0, 0, 0, 0]);
    assert!(result == expected);
    assert!(ctx.is_circuit_valid());
}

/// The number of limbs is what every gadget's loops assume of a [`Word256`].
#[test]
fn limb_count_matches_word256() {
    const { assert!(N_LIMBS * 16 == 256) };
}
