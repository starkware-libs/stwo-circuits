use std::collections::{BTreeMap, HashSet};

use rstest::rstest;

use super::{
    EMPTY_HASH, PatriciaTree, Word256, apply_updates, build_trie, hash_binary, hash_edge,
    root_hash, word256_bit,
};

/// A [`Word256`] with the given low word.
fn w(x: u32) -> Word256 {
    [x, 0, 0, 0, 0, 0, 0, 0]
}

/// A distinct non-zero value for key index `i`.
fn value(i: u32) -> Word256 {
    std::array::from_fn(|k| 0x1000_0000 + i * 8 + k as u32)
}

fn map(entries: &[(Word256, Word256)]) -> BTreeMap<Word256, Word256> {
    entries.iter().cloned().collect()
}

/// Independent twin of `build_trie` + `hash`: computes the canonical root directly from the key
/// set by scanning for the longest shared bit prefix (rather than merging edges bottom-up), so a
/// bug in edge merging cannot hide in both implementations.
fn naive_root(entries: &BTreeMap<Word256, Word256>, height: u32) -> Word256 {
    if entries.is_empty() {
        return EMPTY_HASH;
    }
    if height == 0 {
        assert!(entries.len() == 1);
        return *entries.values().next().unwrap();
    }
    // The longest `shared` such that all keys agree on bits [height - shared, height).
    let keys: Vec<&Word256> = entries.keys().collect();
    let mut shared = 0;
    while shared < height
        && keys.iter().all(|k| {
            word256_bit(k, height - 1 - shared) == word256_bit(keys[0], height - 1 - shared)
        })
    {
        shared += 1;
    }
    if shared == height {
        // A single key runs all the way down to its leaf.
        assert!(entries.len() == 1);
        let (key, value) = entries.iter().next().unwrap();
        let mut path = [0u32; 8];
        for i in 0..height {
            path[(i / 32) as usize] |= word256_bit(key, i) << (i % 32);
        }
        return hash_edge(value, &path, height);
    }
    // Split at the divergence bit, one level below the shared prefix.
    let split = height - shared - 1;
    let (zeros, ones): (BTreeMap<_, _>, BTreeMap<_, _>) =
        entries.iter().map(|(k, v)| (*k, *v)).partition(|(k, _)| word256_bit(k, split) == 0);
    let inner = hash_binary(&naive_root(&zeros, split), &naive_root(&ones, split));
    if shared == 0 {
        return inner;
    }
    // The shared prefix is the edge path: bit `j` of the path is key bit `split + 1 + j`.
    let mut path = [0u32; 8];
    for j in 0..shared {
        path[(j / 32) as usize] |= word256_bit(keys[0], split + 1 + j) << (j % 32);
    }
    hash_edge(&inner, &path, shared)
}

#[test]
fn word256_bit_is_lsb_first() {
    assert!(word256_bit(&w(0b101), 0) == 1);
    assert!(word256_bit(&w(0b101), 1) == 0);
    assert!(word256_bit(&w(0b101), 2) == 1);
    let mut high = [0u32; 8];
    high[1] = 1;
    assert!(word256_bit(&high, 32) == 1);
    assert!(word256_bit(&high, 0) == 0);
}

#[test]
fn empty_trie_has_zero_root() {
    let tree = build_trie(&BTreeMap::new(), 251);
    assert!(tree.is_none());
    assert!(root_hash(&tree) == EMPTY_HASH);
}

#[test]
fn single_entry_is_a_full_height_edge_to_the_leaf() {
    let key = w(0b1011);
    let tree = build_trie(&map(&[(key, value(0))]), 8);
    let expected = PatriciaTree::Edge {
        length: 8,
        path: key,
        bottom: Box::new(PatriciaTree::Leaf { value: value(0) }),
    };
    assert!(*tree.as_ref().unwrap() == expected);
    assert!(root_hash(&tree) == hash_edge(&value(0), &key, 8));
}

/// Keys `000` and `100` diverge at the top bit: a binary root over two length-2 edges.
#[test]
fn diverging_keys_form_a_binary_node() {
    let tree = build_trie(&map(&[(w(0b000), value(0)), (w(0b100), value(1))]), 3);
    let expected = hash_binary(&hash_edge(&value(0), &w(0), 2), &hash_edge(&value(1), &w(0), 2));
    assert!(root_hash(&tree) == expected);
}

/// Keys `110` and `111` share bits 2..1: one length-2 edge (path `0b11`) over a binary of leaves.
#[test]
fn shared_prefix_is_compressed_into_one_edge() {
    let tree = build_trie(&map(&[(w(0b110), value(0)), (w(0b111), value(1))]), 3);
    let expected = hash_edge(&hash_binary(&value(0), &value(1)), &w(0b11), 2);
    assert!(root_hash(&tree) == expected);
}

/// Deterministic pseudo-random key set of `n` keys at the given `height`.
fn pseudo_random_map(n: u32, height: u32) -> BTreeMap<Word256, Word256> {
    let mask = if height.is_multiple_of(32) { u32::MAX } else { (1u32 << (height % 32)) - 1 };
    (0..n)
        .map(|i| {
            let mut key = [0u32; 8];
            let mut state = i.wrapping_mul(0x9E37_79B9).wrapping_add(1);
            for word in key.iter_mut().take((height as usize).div_ceil(32)) {
                state = state.wrapping_mul(0x0019_660D).wrapping_add(0x3C6E_F35F);
                *word = state;
            }
            let top = (height as usize).div_ceil(32) - 1;
            key[top] &= mask;
            (key, value(i))
        })
        .collect()
}

#[rstest]
#[case(1, 10)]
#[case(2, 10)]
#[case(17, 10)]
#[case(50, 64)]
#[case(50, 251)]
fn built_trie_is_canonical_at_its_height(#[case] n: u32, #[case] height: u32) {
    let entries = pseudo_random_map(n, height);
    let tree = build_trie(&entries, height).unwrap();
    assert!(tree.is_canonical(height));
    assert!(tree.height() == height);
}

#[rstest]
#[case(1, 10)]
#[case(2, 10)]
#[case(17, 10)]
#[case(50, 64)]
#[case(50, 251)]
fn built_trie_root_matches_the_naive_twin(#[case] n: u32, #[case] height: u32) {
    let entries = pseudo_random_map(n, height);
    assert!(root_hash(&build_trie(&entries, height)) == naive_root(&entries, height));
}

/// Every one of the 2^8 subsets of an 8-key space yields a distinct root — the map → root mapping
/// is injective on a small exhaustive domain.
#[test]
fn roots_are_distinct_across_all_key_subsets() {
    let mut roots: HashSet<Word256> = HashSet::new();
    for subset in 0u32..256 {
        let entries = map(&(0..8)
            .filter(|i| subset & (1 << i) != 0)
            .map(|i| (w(i), value(i)))
            .collect::<Vec<_>>());
        assert!(
            roots.insert(root_hash(&build_trie(&entries, 3))),
            "duplicate root for {subset:#b}"
        );
    }
    assert!(roots.len() == 256);
}

/// Insert, overwrite, and delete via `apply_updates`, then undo — the root round-trips.
#[test]
fn apply_updates_inserts_overwrites_deletes_and_round_trips() {
    let height = 10;
    let original = pseudo_random_map(9, height);
    let (existing_key, existing_value) = original.iter().next().map(|(k, v)| (*k, *v)).unwrap();

    let inserted_key = w(0b11_0000_0001);
    assert!(!original.contains_key(&inserted_key));
    let updates = [
        (inserted_key, value(100)),                     // insert
        (existing_key, value(101)),                     // overwrite
        (*original.keys().last().unwrap(), EMPTY_HASH), // delete
    ];
    let updated = apply_updates(&original, &updates);
    assert!(root_hash(&build_trie(&updated, height)) != root_hash(&build_trie(&original, height)));

    let undo = [
        (inserted_key, EMPTY_HASH),
        (existing_key, existing_value),
        (*original.keys().last().unwrap(), original.values().last().copied().unwrap()),
    ];
    let restored = apply_updates(&updated, &undo);
    assert!(restored == original);
}

#[test]
fn deleting_every_key_restores_the_empty_root() {
    let entries = pseudo_random_map(5, 16);
    let deletions: Vec<_> = entries.keys().map(|k| (*k, EMPTY_HASH)).collect();
    let emptied = apply_updates(&entries, &deletions);
    assert!(root_hash(&build_trie(&emptied, 16)) == EMPTY_HASH);
}

#[test]
fn non_canonical_trees_are_rejected() {
    let leaf = || Box::new(PatriciaTree::Leaf { value: value(0) });
    // Stacked edges (must be merged into one).
    let stacked = PatriciaTree::Edge {
        length: 1,
        path: w(0),
        bottom: Box::new(PatriciaTree::Edge { length: 1, path: w(0), bottom: leaf() }),
    };
    assert!(!stacked.is_canonical(2));
    // A zero-length edge.
    assert!(!PatriciaTree::Edge { length: 0, path: w(0), bottom: leaf() }.is_canonical(1));
    // Path bits at or above `length` must be zero.
    assert!(!PatriciaTree::Edge { length: 1, path: w(0b10), bottom: leaf() }.is_canonical(1));
    // A leaf must sit exactly at height 0.
    assert!(!PatriciaTree::Leaf { value: value(0) }.is_canonical(1));
    assert!(!PatriciaTree::Edge { length: 1, path: w(0), bottom: leaf() }.is_canonical(2));
    // A present leaf must be non-zero.
    assert!(!PatriciaTree::Leaf { value: EMPTY_HASH }.is_canonical(0));
    // The valid counterparts, for contrast.
    assert!(PatriciaTree::Edge { length: 1, path: w(1), bottom: leaf() }.is_canonical(1));
    assert!(PatriciaTree::Leaf { value: value(0) }.is_canonical(0));
}

#[test]
#[should_panic(expected = "fit height")]
fn key_exceeding_the_height_panics() {
    build_trie(&map(&[(w(0b1000), value(0))]), 3);
}

#[test]
#[should_panic(expected = "zero")]
fn zero_value_panics() {
    build_trie(&map(&[(w(1), EMPTY_HASH)]), 3);
}

#[test]
#[should_panic(expected = "exceeds 256")]
fn height_above_256_panics() {
    build_trie(&BTreeMap::new(), 257);
}
