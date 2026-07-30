use std::array;

use itertools::zip_eq;
use rstest::rstest;
use stwo::core::fields::qm31::QM31;

use crate::context::{Context, TraceContext};
use crate::ivalue::{NoValue, qm31_from_u32s};
use crate::keccak::{
    KECCAK_STATE_N_LANES, U64Lane, keccak_f1600, keccak_f1600_u64, keccak256_bytes, keccak256_u8,
};
use crate::ops::{Guess, eq};
use crate::stats::Stats;

/// Keccak-f[1600] applied to the all-zero state, from XKCP's
/// `KeccakF-1600-IntermediateValues.txt`.
const PERMUTED_ZERO_STATE: [u64; KECCAK_STATE_N_LANES] = [
    0xF1258F7940E1DDE7,
    0x84D5CCF933C0478A,
    0xD598261EA65AA9EE,
    0xBD1547306F80494D,
    0x8B284E056253D057,
    0xFF97A42D7F8E6FD4,
    0x90FEE5A0A44647C4,
    0x8C5BDA0CD6192E76,
    0xAD30A6F71B19059C,
    0x30935AB7D08FFC64,
    0xEB5AA93F2317D635,
    0xA9A6E6260D712103,
    0x81A57C16DBCF555F,
    0x43B831CD0347C826,
    0x01F22F1A11A5569F,
    0x05E5635A21D9AE61,
    0x64BEFEF28CC970F2,
    0x613670957BC46611,
    0xB87C5A554FD00ECB,
    0x8C3EE88A1CCF32C8,
    0x940C7922AE3A2614,
    0x1841F924A2C509E4,
    0x16F53526E70465C2,
    0x75F644E97F30A13B,
    0xEAF1FF7B5CECA249,
];

#[test]
fn test_keccak_f1600_u64() {
    let mut state = [0u64; KECCAK_STATE_N_LANES];
    keccak_f1600_u64(&mut state);
    assert_eq!(state, PERMUTED_ZERO_STATE);
}

/// SHA3-256 of the empty message, computed with the reference permutation — an independent
/// cross-check of the round constants and step functions.
#[test]
fn test_keccak_f1600_u64_sha3_256_empty() {
    let mut state = [0u64; KECCAK_STATE_N_LANES];
    // Absorb the padded empty message: domain-separation suffix 0x06 and final padding bit 0x80
    // within the 136-byte rate.
    state[0] ^= 0x06;
    state[16] ^= 0x8000000000000000;
    keccak_f1600_u64(&mut state);
    let digest: Vec<u8> = state[..4].iter().flat_map(|lane| lane.to_le_bytes()).collect();
    assert_eq!(
        digest,
        [
            0xa7, 0xff, 0xc6, 0xf8, 0xbf, 0x1e, 0xd7, 0x66, 0x51, 0xc1, 0x47, 0x56, 0xa0, 0x61,
            0xd6, 0x62, 0xf5, 0x80, 0xff, 0x4d, 0xe4, 0x3b, 0x49, 0xfa, 0x82, 0xd8, 0x0a, 0x4b,
            0x80, 0xf8, 0x43, 0x4a,
        ]
    );
}

#[rstest]
#[case::success(false)]
#[case::wrong_output(true)]
fn test_keccak_f1600(#[case] wrong_output: bool) {
    let mut context = TraceContext::default();

    // Deterministic pseudo-random state exercising all limbs of every lane.
    let input_state: [u64; KECCAK_STATE_N_LANES] =
        array::from_fn(|i| (i as u64 + 1).wrapping_mul(0x9E3779B97F4A7C15));
    let mut expected = input_state;
    keccak_f1600_u64(&mut expected);
    if wrong_output {
        expected[0] ^= 1;
    }

    let input = input_state.map(U64Lane::from).guess(&mut context);
    let output = keccak_f1600(&mut context, input);

    for (out, exp) in zip_eq(output, expected) {
        if !wrong_output {
            assert_eq!(out.value(&context), exp);
        }
        let exp_lane = U64Lane::from(exp).guess(&mut context);
        eq(&mut context, *out.low.get(), *exp_lane.low.get());
        eq(&mut context, *out.high.get(), *exp_lane.high.get());
    }

    // Gate-count snapshot documenting the cost of a DSL-only permutation (plus this test's
    // input/expected-output guesses). Destructured with `..` so the assertion is insensitive
    // to unrelated gate counters.
    let Stats { equals, add, sub, mul, pointwise_mul, guess, triple_xor, .. } = context.stats;
    assert_eq!(
        (equals, add, sub, mul, pointwise_mul, guess, triple_xor),
        (8402, 8260, 2400, 9652, 2784, 11336, 4320)
    );

    let context = context.finalize(false);
    context.circuit().check_yields();
    assert_eq!(context.is_circuit_valid(), !wrong_output);
}

#[test]
fn test_u64_lane_pack_to_qm31() {
    let mut context = TraceContext::default();

    let word: u64 = 0x0123_4567_89AB_CDEF;
    let lane = U64Lane::from(word).guess(&mut context);
    let packed = lane.pack_to_qm31(&mut context);

    assert_eq!(
        context.get(packed),
        qm31_from_u32s(
            (word & 0xFFFF) as u32,
            ((word >> 16) & 0xFFFF) as u32,
            ((word >> 32) & 0xFFFF) as u32,
            (word >> 48) as u32,
        )
    );

    let context = context.finalize(false);
    context.circuit().check_yields();
    assert!(context.is_circuit_valid());
}

/// Known-answer vectors for Ethereum's `keccak256`, produced by an independent implementation
/// (`alloy_primitives::keccak256`). Each message is `test_message(n)`; the lengths bracket the
/// 136-byte rate to cover single-, two- and three-block sponges. The empty-input digest is the
/// canonical published one.
const KECCAK256_VECTORS: [(usize, &str); 7] = [
    (0, "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"),
    (2, "11142b44fdf76f1d4ee1c17b439c667139ada3ab64ad505873555fcb29bc1e69"),
    (4, "974f13a40476284e056acb216c7b0c4d8619c2d4ffa95d554ba04ac6323bef93"),
    (134, "8b2af3b215fa1425ced8e97e7a45902093b060e08317780d936040bce17997d0"),
    (136, "eaccfc5aa7bf6bf1941809ef7cc9ee6a2fa306a7dd1de3f2e8504849b0a5e3c4"),
    (138, "2cead55fdc33ca13ce7609a106e84659fb76b0f57a2472f7d7adcd2f070de486"),
    (300, "f10b5f4756f414ea659c96a9104b919d2441dca5c621babd0d26fefa15a64208"),
];

/// The message used by every keccak256 test: `n` bytes of a fixed pseudo-random pattern.
fn test_message(n: usize) -> Vec<u8> {
    (0..n).map(|i| (i as u8).wrapping_mul(31).wrapping_add(7)).collect()
}

fn decode_hex(hex: &str) -> Vec<u8> {
    (0..hex.len()).step_by(2).map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap()).collect()
}

#[test]
fn test_keccak256_u8_vectors() {
    for (n_bytes, expected_hex) in KECCAK256_VECTORS {
        assert_eq!(
            keccak256_u8(&test_message(n_bytes)).to_vec(),
            decode_hex(expected_hex),
            "keccak256 mismatch for a {n_bytes}-byte message"
        );
    }
}

/// The in-circuit sponge must reproduce the known-answer digests, for single-block,
/// rate-exact and multi-block messages alike.
#[rstest]
#[case::empty(0)]
#[case::short(4)]
#[case::one_limb_before_rate(134)]
#[case::exactly_rate(136)]
#[case::two_blocks(138)]
#[case::three_blocks(300)]
fn test_keccak256_in_circuit(#[case] n_bytes: usize) {
    let message = test_message(n_bytes);
    let (_, expected_hex) =
        KECCAK256_VECTORS.iter().find(|(n, _)| *n == n_bytes).expect("length must have a vector");
    let expected = decode_hex(expected_hex);

    let mut context = TraceContext::default();
    let digest = keccak256_bytes(&mut context, &message);

    for (t, lane) in digest.iter().enumerate() {
        let expected_lane = u64::from_le_bytes(expected[8 * t..8 * (t + 1)].try_into().unwrap());
        assert_eq!(lane.value(&context), expected_lane, "lane {t} mismatch");
    }

    let context = context.finalize(false);
    context.circuit().check_yields();
    assert!(context.is_circuit_valid());
}

/// A tampered message limb must change the digest — the sponge really depends on its input
/// wires rather than on guessed output.
#[test]
fn test_keccak256_in_circuit_rejects_tampered_digest() {
    let message = [0xAB; 8];
    let mut context = TraceContext::default();
    let digest = keccak256_bytes(&mut context, &message);

    // Constrain the digest's first lane to the digest of a *different* message.
    let wrong = keccak256_u8(&[0xAC; 8]);
    let wrong_lane = u64::from_le_bytes(wrong[0..8].try_into().unwrap());
    let wrong_lane = U64Lane::from(wrong_lane).guess(&mut context);
    eq(&mut context, *digest[0].low.get(), *wrong_lane.low.get());
    eq(&mut context, *digest[0].high.get(), *wrong_lane.high.get());

    let context = context.finalize(false);
    assert!(!context.is_circuit_valid());
}

/// The circuit topology must not depend on the concrete values: building with `QM31` witness
/// values and with `NoValue` must produce identical circuits.
#[test]
fn test_keccak_f1600_topology_independent_of_values() {
    let with_values = {
        let mut context = TraceContext::default();
        let input: [U64Lane<QM31>; KECCAK_STATE_N_LANES] =
            array::from_fn(|i| U64Lane::from((i as u64).wrapping_mul(0x0123456789ABCDEF)));
        let input = input.guess(&mut context);
        keccak_f1600(&mut context, input);
        context.finalize(false)
    };
    let without_values = {
        let mut context = Context::<NoValue>::default();
        let input: [U64Lane<NoValue>; KECCAK_STATE_N_LANES] =
            array::from_fn(|_| U64Lane::from(NoValue));
        let input = input.guess(&mut context);
        keccak_f1600(&mut context, input);
        context.finalize(false)
    };
    assert!(with_values.circuit() == without_values.circuit());
}
