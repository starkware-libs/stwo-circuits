//! Starknet-style Patricia trie over blake2s.
//!
//! A Patricia trie is a binary Merkle trie with path compression: a fixed `height` `H` determines
//! the key space (`H`-bit keys), long single-child runs are compressed into *edge* nodes, and the
//! empty subtree is represented by the all-zero hash. See [`mod@reference`] for the exact node
//! encoding and hashing rules.
//!
//! - [`mod@reference`] — the out-of-circuit reference implementation: the canonical trie build,
//!   hashing, and batch updates. It is the witness generator and test twin for the in-circuit
//!   Patricia verifiers.
//! - [`mod@skeleton`] — extraction of the *touched skeleton* over a batch of keys: the witness the
//!   skeleton circuit consumes, plus the invariants it owes.
//! - [`mod@skeleton_circuit`] — `verify_patricia_skeleton`, the in-circuit skeleton verifier
//!   (design doc §5 step 1), and the slot core the update circuit reuses (P6).
//! - [`mod@update`] — the update witness: two skeletons over one shared sibling list, frontier
//!   reconciliation, and the update-level invariants.
//! - [`mod@update_circuit`] — `verify_patricia_update`, the in-circuit `prev_root → new_root`
//!   verifier (design doc §5 step 2).
//! - [`mod@word_gadgets`] — limb-level gadgets over 256-bit words: doubling, the variable shift,
//!   the 251-bit truncation and the additive edge length.

pub mod reference;
pub mod skeleton;
pub mod skeleton_circuit;
pub mod update;
pub mod update_circuit;
pub mod word_gadgets;

#[cfg(test)]
#[path = "slot_count_test.rs"]
mod slot_count_test;
