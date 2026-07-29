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

pub mod reference;
pub mod skeleton;

#[cfg(test)]
#[path = "slot_count_test.rs"]
mod slot_count_test;
