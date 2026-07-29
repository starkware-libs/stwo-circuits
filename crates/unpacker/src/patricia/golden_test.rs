//! Golden roots pinning this trie to the **production** blake2s-251 convention.
//!
//! Every other test in this module compares the reference implementation against itself (its
//! naive twin, slot counts, the sibling identity), so all of them stay green if the hash
//! convention changes — which is exactly how a reference implementation drifts from the system
//! it is meant to model, taking the circuits built on it along.
//!
//! The vectors below were computed by a third, independent implementation (a standalone
//! blake2s + trie build, neither this crate nor the Rust production code), from the documented
//! rules: `hash2(x, y) = LE_u32s(blake2s(LE32(x) ‖ LE32(y))) & (2^251 - 1)`, binary =
//! `hash2(l, r)`, edge = `hash2(bottom, path) + ℓ`, leaf = value, empty = 0. Production is
//! `payment_thread_patricia::{Blake2s251, BlakeTreeHashFunction}`; the Cairo0 program mirrors it
//! in `blake_as_hash.cairo` / `patricia_with_blake.cairo`.
//!
//! Changing a vector means changing what this trie *is*. That is a spec change, and it must be
//! made in a commit that says so and that updates the other two implementations too.
//!
//! Still open (the last link in the chain): confirming that production's canonical *construction*
//! — in particular the `Felt::from(path)` bit layout of an `EdgePath` — matches the LSB-aligned
//! `path` used here. That needs a test on the production side; these vectors pin the spec, not
//! yet the agreement.

use std::collections::BTreeMap;

use rstest::rstest;

use super::{Word256, build_trie, hash2};

/// A trie fixture: `height`, `(key, value)` entries, and the expected root.
struct Golden {
    height: u32,
    entries: &'static [(Word256, Word256)],
    root: Word256,
}

const fn w(low: u32) -> Word256 {
    [low, 0, 0, 0, 0, 0, 0, 0]
}

/// `key` with bit 250 set — a height-251 key in the top half of the trie.
const fn w_top(low: u32) -> Word256 {
    [low, 0, 0, 0, 0, 0, 0, 1 << 26]
}

const SINGLE_KEY_FULL_HEIGHT_EDGE: Golden = Golden {
    height: 8,
    entries: &[(w(0b1011_0101), w(0xAAAA))],
    root: [3910960737, 252377291, 2287723055, 2635285127, 17668072, 324565021, 17570955, 6653737],
};

const TWO_KEYS_DIVERGE_AT_TOP: Golden = Golden {
    height: 8,
    entries: &[(w(0b0000_0001), w(0x11)), (w(0b1000_0001), w(0x22))],
    root: [
        4136634345, 2160790607, 3195665831, 797018471, 3793532246, 919670614, 4198829721, 73703186,
    ],
};

const TWO_KEYS_DIFFER_LAST_BIT: Golden = Golden {
    height: 8,
    entries: &[(w(0b1011_0100), w(0x33)), (w(0b1011_0101), w(0x44))],
    root: [
        4283538750, 3729676026, 2857743097, 23568369, 2253465753, 3823934659, 1819047636, 21196289,
    ],
};

const THREE_KEYS: Golden = Golden {
    height: 8,
    entries: &[(w(0), w(1)), (w(0b0111_1111), w(2)), (w(0b1111_1111), w(3))],
    root: [
        1176121524, 983620962, 1388239265, 2139299531, 3050943003, 123917973, 2569403848, 7763844,
    ],
};

/// The nonce trie's shape: one leaf whose value is `1`.
const NONCE_STYLE_VALUE_ONE: Golden = Golden {
    height: 8,
    entries: &[(w(0b0000_1111), w(1))],
    root: [
        1920732672, 3021102057, 3380700722, 3048931007, 1029208712, 1790707113, 1217141181,
        108931833,
    ],
};

const HEIGHT_251_SINGLE_KEY: Golden = Golden {
    height: 251,
    entries: &[(w_top(12345), w(0x99))],
    root: [
        3536686549, 2668338429, 231793187, 3849744909, 2315987292, 303385793, 2722250063, 18150611,
    ],
};

const HEIGHT_251_TWO_KEYS: Golden = Golden {
    height: 251,
    entries: &[(w(1), w(5)), (w_top(1), w(6))],
    root: [
        1698004502, 3694472958, 1272126549, 2239706199, 3628672642, 3894951010, 4211884399,
        57183629,
    ],
};

#[rstest]
#[case(SINGLE_KEY_FULL_HEIGHT_EDGE)]
#[case(TWO_KEYS_DIVERGE_AT_TOP)]
#[case(TWO_KEYS_DIFFER_LAST_BIT)]
#[case(THREE_KEYS)]
#[case(NONCE_STYLE_VALUE_ONE)]
#[case(HEIGHT_251_SINGLE_KEY)]
#[case(HEIGHT_251_TWO_KEYS)]
fn root_matches_the_production_convention(#[case] golden: Golden) {
    let entries: BTreeMap<Word256, Word256> = golden.entries.iter().copied().collect();
    let tree = build_trie(&entries, golden.height).expect("non-empty trie");
    assert_eq!(tree.hash(), golden.root, "root diverged from the pinned production convention");
}

/// The inner hash truncates to 251 bits, so the top word never exceeds 27 bits. A single
/// unmasked hash would break every root above; this pins the primitive on its own.
#[test]
fn hash2_is_truncated_to_251_bits() {
    for x in 0..64u32 {
        let hash = hash2(&w(x), &w(x.wrapping_mul(7).wrapping_add(1)));
        assert!(hash[7] < (1 << 27), "hash2 top word {} exceeds 27 bits", hash[7]);
    }
}
