//! In-circuit Merkle "unpacker": proving a Merkle `root` commits to an entire ordered leaf set.
//!
//! - [`permutation`] — the multiset-permutation primitive over
//!   [`HashValue`](circuits::blake::HashValue)s.
//! - [`tree`] — the [`BinaryTree`](tree::BinaryTree) shape.
//! - [`unpacker`] — the fixed, depth-independent commitment circuit
//!   ([`verify_merkle_commitment`](unpacker::verify_merkle_commitment)).

pub mod permutation;
pub mod tree;
pub mod unpacker;
