//! In-circuit Merkle "unpacker": proving a Merkle `root` commits to an entire ordered leaf set,
//! where each entry is a [`Node`](unpacker::Node) pairing a `circuit_hash` with a `subtree_hash`.
//!
//! - [`permutation`] — the multiset-permutation primitive over fixed-width word units
//!   ([`permute_units`](permutation::permute_units);
//!   [`permute_hash_values`](permutation::permute_hash_values) is the
//!   [`HashValue`](circuits::blake::HashValue) specialization).
//! - [`gadgets`] — shared in-circuit gadgets (hash selection, is-zero, witness guessing).
//! - [`tree`] — the [`BinaryTree`](tree::BinaryTree) shape.
//! - [`unpacker`] — the fixed, depth-independent commitment circuit
//!   ([`verify_merkle_commitment`](unpacker::verify_merkle_commitment)).
//! - [`patricia`] — the Starknet-style Patricia trie over blake2s (reference implementation; the
//!   in-circuit verifiers build on it).

pub mod gadgets;
pub mod patricia;
pub mod permutation;
pub mod tree;
pub mod unpacker;
