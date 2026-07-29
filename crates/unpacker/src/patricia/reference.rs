//! Reference (out-of-circuit) Starknet-style Patricia trie over blake2s.
//!
//! Maps `height`-bit keys (`height ≤ 256`) to non-zero 256-bit values. Keys, values, and hashes
//! are all [`Word256`]s — eight little-endian `u32` words, word 0 least significant. The key bit
//! at index `h - 1` selects the child (0 = left, 1 = right) at a node of height `h`; leaves sit at
//! height 0.
//!
//! # Node encoding and hashing
//!
//! - **Empty subtree**: hash = `0` (all-zero words). A key mapping to value `0` is *absent* —
//!   writing `0` deletes it.
//! - **Leaf** (height 0): hash = its value (non-zero for present keys).
//! - **Binary node** (height `h ≥ 1`): two non-empty children at height `h - 1`; `hash =
//!   hash2(left_hash, right_hash)`.
//! - **Edge node** (height `h`, length `1 ≤ ℓ ≤ h`): compresses `ℓ` single-child levels down to a
//!   non-empty `bottom` at height `h - ℓ`; `hash = hash2(bottom_hash, path) + ℓ` — the length is
//!   **added** to the hash, not hashed with it. `path` holds the `ℓ` compressed key bits,
//!   LSB-aligned: path bit `j` is key bit `h - ℓ + j`, and bits `≥ ℓ` are zero.
//!
//! The inner hash is [`hash2`]: blake2s over the two operands' little-endian bytes, **truncated
//! to 251 bits**. This is the production convention — `payment_thread_patricia::Blake2s251` and
//! `blake_as_hash.cairo` in the Cairo0 program — so roots computed here equal the roots the
//! sequencer commits and the Cairo0 program proves. Node hashes are therefore 251-bit values,
//! which is what every in-circuit `HashValue` over this trie must range-check.
//!
//! # Canonical form
//!
//! [`build_trie`] produces the *canonical* trie of a map, making the map → root mapping unique:
//!
//! - edges are maximally merged: an edge's `bottom` is never itself an edge, and every edge has
//!   length `≥ 1`;
//! - a binary node's children are both non-empty (a lone child is folded into an edge instead);
//! - the empty subtree is always the all-zero hash, never a hashed encoding of "empty".
//!
//! [`PatriciaTree::is_canonical`] checks these invariants (plus height consistency) for an
//! arbitrary tree. Uniqueness is what update-circuit soundness leans on: without it the same map
//! could hash to two different roots via shape-only rewrites.

use std::collections::BTreeMap;

use stwo::core::vcs::blake2_hash::{Blake2sHash, Blake2sHasher};

#[cfg(test)]
#[path = "reference_test.rs"]
mod test;

#[cfg(test)]
#[path = "golden_test.rs"]
mod golden_test;

/// Eight little-endian `u32` words (word 0 least significant) — the common shape of keys, values,
/// and blake2s hashes.
pub type Word256 = [u32; 8];

/// The hash of the empty subtree; also the "absent" leaf value.
pub const EMPTY_HASH: Word256 = [0; 8];

/// Returns bit `i` (0-based from the least-significant bit) of a [`Word256`].
pub fn word256_bit(word: &Word256, i: u32) -> u32 {
    assert!(i < 256, "bit index {i} out of range");
    (word[(i / 32) as usize] >> (i % 32)) & 1
}

/// A non-empty Patricia subtree. The empty subtree is not representable — it exists only as the
/// [`EMPTY_HASH`] root of an empty trie (see [`root_hash`]) — so canonical trees are non-empty by
/// construction.
///
/// Heights are contextual, not stored: interpreting a tree at height `h`, a `Binary`'s children
/// live at `h - 1` and an `Edge`'s bottom at `h - length`. [`PatriciaTree::height`] recomputes the
/// implied height.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatriciaTree {
    /// A key's value, at height 0. Non-zero for present keys.
    Leaf { value: Word256 },
    /// Two non-empty children one level down.
    Binary { left: Box<PatriciaTree>, right: Box<PatriciaTree> },
    /// `length` compressed single-child levels down to `bottom`. `path` holds the compressed key
    /// bits, LSB-aligned (see the module docs for the exact bit layout).
    Edge { length: u32, path: Word256, bottom: Box<PatriciaTree> },
}

/// Converts a [`Blake2sHash`] (32 bytes) into eight little-endian `u32` words.
fn hash_to_words(hash: Blake2sHash) -> Word256 {
    std::array::from_fn(|i| u32::from_le_bytes(hash.0[4 * i..4 * i + 4].try_into().unwrap()))
}

/// Mask for the top word of a 251-bit hash: `hash2` truncates to 251 bits, so the most
/// significant word keeps 27 of its 32 bits.
const TOP_WORD_MASK: u32 = (1 << 27) - 1;

/// `hash2(x, y) = LE_u32s(blake2s(LE32(x) ‖ LE32(y))) & (2^251 - 1)` — the production inner
/// hash (`payment_thread_patricia::Blake2s251`, mirrored by `blake_as_hash.cairo`).
pub fn hash2(x: &Word256, y: &Word256) -> Word256 {
    let mut hasher = Blake2sHasher::new();
    for word in x.iter().chain(y.iter()) {
        hasher.update(&word.to_le_bytes());
    }
    let mut words = hash_to_words(hasher.finalize());
    words[7] &= TOP_WORD_MASK;
    words
}

/// Binary-node hash: `hash2(left, right)`.
pub fn hash_binary(left: &Word256, right: &Word256) -> Word256 {
    hash2(left, right)
}

/// Edge-node hash: `hash2(bottom, path) + length` — the length is *added* to the truncated
/// hash, not hashed with it. Sum of a 251-bit value and `length ≤ 251`, so it cannot overflow
/// 256 bits and needs no reduction here.
pub fn hash_edge(bottom: &Word256, path: &Word256, length: u32) -> Word256 {
    add_small(hash2(bottom, path), length)
}

/// Adds a small scalar to a `Word256`, propagating carries.
fn add_small(mut words: Word256, addend: u32) -> Word256 {
    let mut carry = addend as u64;
    for word in words.iter_mut() {
        let sum = *word as u64 + carry;
        *word = sum as u32;
        carry = sum >> 32;
        if carry == 0 {
            break;
        }
    }
    debug_assert_eq!(carry, 0, "a 251-bit hash plus a length cannot overflow 256 bits");
    words
}

impl PatriciaTree {
    /// The subtree's hash, per the node-encoding rules in the module docs.
    pub fn hash(&self) -> Word256 {
        match self {
            PatriciaTree::Leaf { value } => *value,
            PatriciaTree::Binary { left, right } => hash_binary(&left.hash(), &right.hash()),
            PatriciaTree::Edge { length, path, bottom } => hash_edge(&bottom.hash(), path, *length),
        }
    }

    /// The height implied by the tree's structure (leaf = 0, binary = child + 1, edge = bottom +
    /// length). Panics if a binary node's children imply different heights.
    pub fn height(&self) -> u32 {
        match self {
            PatriciaTree::Leaf { .. } => 0,
            PatriciaTree::Binary { left, right } => {
                let height = left.height();
                assert!(height == right.height(), "binary children at different heights");
                height + 1
            }
            PatriciaTree::Edge { length, bottom, .. } => bottom.height() + length,
        }
    }

    /// Whether the tree is a canonical Patricia trie of the given `height`: leaves are non-zero
    /// and at height 0, edges have `1 ≤ length ≤ height` with zero path bits `≥ length` and a
    /// non-edge bottom (maximal merging), and children/bottoms are recursively canonical at their
    /// implied heights.
    pub fn is_canonical(&self, height: u32) -> bool {
        match self {
            PatriciaTree::Leaf { value } => height == 0 && *value != EMPTY_HASH,
            PatriciaTree::Binary { left, right } => {
                height >= 1 && left.is_canonical(height - 1) && right.is_canonical(height - 1)
            }
            PatriciaTree::Edge { length, path, bottom } => {
                (1..=height).contains(length)
                    && (*length..256).all(|i| word256_bit(path, i) == 0)
                    && !matches!(bottom.as_ref(), PatriciaTree::Edge { .. })
                    && bottom.is_canonical(height - length)
            }
        }
    }
}

/// Extends `child` (a subtree at height `h - 1`) one level up with direction `bit`, merging into
/// an existing edge to keep edges maximal: the result is an edge at height `h` whose new top path
/// bit is `bit`.
fn extend_edge(bit: u32, child: PatriciaTree) -> PatriciaTree {
    match child {
        PatriciaTree::Edge { length, mut path, bottom } => {
            path[(length / 32) as usize] |= bit << (length % 32);
            PatriciaTree::Edge { length: length + 1, path, bottom }
        }
        other => {
            let mut path = [0u32; 8];
            path[0] = bit;
            PatriciaTree::Edge { length: 1, path, bottom: Box::new(other) }
        }
    }
}

/// Builds the canonical subtree over non-empty `entries` at `height`. Keys are the low `height`
/// bits (higher bits already validated by [`build_trie`]).
fn build_subtree(entries: &[(Word256, Word256)], height: u32) -> PatriciaTree {
    assert!(!entries.is_empty());
    if height == 0 {
        assert!(entries.len() == 1, "duplicate key");
        return PatriciaTree::Leaf { value: entries[0].1 };
    }
    let (ones, zeros): (Vec<_>, Vec<_>) =
        entries.iter().cloned().partition(|(key, _)| word256_bit(key, height - 1) == 1);
    match (zeros.is_empty(), ones.is_empty()) {
        (false, false) => PatriciaTree::Binary {
            left: Box::new(build_subtree(&zeros, height - 1)),
            right: Box::new(build_subtree(&ones, height - 1)),
        },
        // All keys share the top bit: recurse one level down and fold the bit into an edge.
        (true, false) => extend_edge(1, build_subtree(&ones, height - 1)),
        (false, true) => extend_edge(0, build_subtree(&zeros, height - 1)),
        (true, true) => unreachable!("entries is non-empty"),
    }
}

/// Builds the canonical Patricia trie of `height` over `entries` (key → non-zero value). Returns
/// `None` for the empty map (root hash [`EMPTY_HASH`]).
///
/// Panics if `height > 256`, if a key has bits set at or above `height`, or if a value is zero
/// (an absent key must simply not appear).
pub fn build_trie(entries: &BTreeMap<Word256, Word256>, height: u32) -> Option<PatriciaTree> {
    assert!(height <= 256, "height {height} exceeds 256");
    for (key, value) in entries {
        assert!(
            (height..256).all(|i| word256_bit(key, i) == 0),
            "key {key:?} does not fit height {height}"
        );
        assert!(*value != EMPTY_HASH, "value for key {key:?} is zero; omit absent keys");
    }
    if entries.is_empty() {
        return None;
    }
    let entries: Vec<(Word256, Word256)> = entries.iter().map(|(k, v)| (*k, *v)).collect();
    Some(build_subtree(&entries, height))
}

/// The root hash of a (possibly empty) trie.
pub fn root_hash(tree: &Option<PatriciaTree>) -> Word256 {
    tree.as_ref().map_or(EMPTY_HASH, PatriciaTree::hash)
}

/// Applies `updates` in order to `entries`, returning the updated map: a zero value deletes the
/// key, any other value inserts or overwrites it.
pub fn apply_updates(
    entries: &BTreeMap<Word256, Word256>,
    updates: &[(Word256, Word256)],
) -> BTreeMap<Word256, Word256> {
    let mut result = entries.clone();
    for (key, value) in updates {
        if *value == EMPTY_HASH {
            result.remove(key);
        } else {
            result.insert(*key, *value);
        }
    }
    result
}
