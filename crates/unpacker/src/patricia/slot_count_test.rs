//! Slot-count cost model for the (unimplemented) Patricia skeleton circuit.
//!
//! The circuit's gate count is driven by how many binary / edge / sibling slots a batch of `K`
//! keys touches inside a trie of `N` entries. `DESIGN.md` claims `K - 1` binary slots,
//! independent of the trie's contents; that omits the binary nodes where a touched path meets an
//! *untouched* sibling subtree, which in a canonical trie is every such meeting. The real
//! scaling is `Θ(K · log2(N/K))`.
//!
//! [`report_slot_counts`] is `#[ignore]`d — it builds tries up to 1M entries (~30 s) and reports
//! rather than asserts. [`sibling_identity_holds`] pins the one exact relation the circuit's
//! multiset identity depends on. See `docs/payments-circuit-design.md` §4.1.

use std::collections::BTreeMap;

use super::reference::{PatriciaTree, Word256, build_trie, word256_bit};

const HEIGHT: u32 = 251;

/// Slots a touched skeleton needs: binary and edge nodes on the union of the root-to-leaf paths
/// of `keys`, plus the untouched sibling subtrees hanging off them.
#[derive(Debug, Default, PartialEq, Eq)]
struct SlotCounts {
    binary: usize,
    edge: usize,
    siblings: usize,
}

fn count_skeleton(tree: &PatriciaTree, height: u32, keys: &[Word256]) -> SlotCounts {
    if keys.is_empty() {
        return SlotCounts::default();
    }
    match tree {
        PatriciaTree::Leaf { .. } => SlotCounts::default(),
        PatriciaTree::Binary { left, right } => {
            let (zeros, ones): (Vec<_>, Vec<_>) =
                keys.iter().partition(|key| word256_bit(key, height - 1) == 0);
            let low = count_skeleton(left, height - 1, &zeros);
            let high = count_skeleton(right, height - 1, &ones);
            SlotCounts {
                binary: low.binary + high.binary + 1,
                edge: low.edge + high.edge,
                // An untouched child of a touched binary node is a sibling unit.
                siblings: low.siblings
                    + high.siblings
                    + usize::from(zeros.is_empty())
                    + usize::from(ones.is_empty()),
            }
        }
        PatriciaTree::Edge { length, bottom, .. } => {
            let below = count_skeleton(bottom, height - length, keys);
            SlotCounts { edge: below.edge + 1, ..below }
        }
    }
}

/// Deterministic pseudorandom 251-bit key from a counter (xorshift; no rand dependency, so the
/// reported numbers are reproducible).
fn key_of(i: u64) -> Word256 {
    let mut state = i.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0x1234_5678);
    let mut key = [0u32; 8];
    for word in key.iter_mut() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        *word = state as u32;
    }
    key[7] &= (1 << 27) - 1; // keep the key under 2^251
    key
}

fn trie_of(n_entries: u64) -> BTreeMap<Word256, Word256> {
    (0..n_entries).map(|i| (key_of(i), key_of(i + (1 << 40)))).collect()
}

/// The relation the skeleton circuit's multiset identity rests on: with `K` leaf slots, `B`
/// binary slots and `E` edge slots, the produced units (leaves + binaries + edges + siblings,
/// less the root) balance the consumed units (`2B` children + `E` bottoms) exactly when
/// `siblings = B - K + 1`. Sizing a capacity from a wrong sibling count makes the circuit
/// unsatisfiable, so this is pinned rather than reported.
#[test]
fn sibling_identity_holds() {
    let entries = trie_of(2_000);
    let tree = build_trie(&entries, HEIGHT).unwrap();
    for k in [1usize, 2, 7, 16, 64, 256] {
        let keys: Vec<Word256> = entries.keys().take(k).copied().collect();
        let counts = count_skeleton(&tree, HEIGHT, &keys);
        assert_eq!(
            counts.siblings,
            counts.binary + 1 - k,
            "sibling identity broken at K={k}: {counts:?}"
        );
    }
}

/// Reports the measured cost model behind `docs/payments-circuit-design.md` §4.1. Ignored: it
/// builds tries up to 1M entries (~30 s).
#[test]
#[ignore = "cost-model report over tries up to 1M entries; ~30s"]
fn report_slot_counts() {
    println!(
        "{:>10} {:>6} {:>8} {:>6} {:>9} {:>8}",
        "N", "K", "binary", "edge", "siblings", "bin/K"
    );
    for n_entries in [1_000u64, 10_000, 100_000, 1_000_000] {
        let entries = trie_of(n_entries);
        let tree = build_trie(&entries, HEIGHT).unwrap();
        for k in [16usize, 64, 256] {
            let keys: Vec<Word256> = entries.keys().take(k).copied().collect();
            let SlotCounts { binary, edge, siblings } = count_skeleton(&tree, HEIGHT, &keys);
            println!(
                "{n_entries:>10} {k:>6} {binary:>8} {edge:>6} {siblings:>9} {:>8.1}",
                binary as f64 / k as f64
            );
        }
    }
}
