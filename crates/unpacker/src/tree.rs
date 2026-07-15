//! Binary tree data structure for in-circuit Merkle commitments.
//!
//! [`BinaryTree`] describes an arbitrary (possibly unbalanced) full binary tree; its shape drives
//! the witness for the depth-independent Merkle commitment circuit (see [`crate::unpacker`]).
//!
//! Both variants carry data: a `Leaf` holds a value of type `L` (in the unpacker, a
//! [`Node`](crate::unpacker::Node) pairing a leaf's `circuit_hash` with its `subtree_hash`), and an
//! internal `Node` holds a value of type `N` (the `circuit_hash` supplied for that internal
//! position — its `subtree_hash` is *computed* from its children, so it is not carried here).

/// A full binary tree: each `Leaf` carries a value of type `L`, and each internal `Node` carries a
/// value of type `N` together with its two children. Arbitrary, possibly unbalanced shapes
/// (caterpillars, lone odd nodes carried up, etc.) are expressed directly rather than inferred from
/// a flat leaf list.
#[derive(Clone)]
pub enum BinaryTree<L, N> {
    Leaf(L),
    Node(N, Box<[BinaryTree<L, N>; 2]>),
}

impl<L, N> BinaryTree<L, N> {
    /// Collects references to every leaf value in left-to-right (depth-first) order. This order is
    /// independent of the tree's shape, so a given leaf multiset always yields the same sequence.
    pub fn leaves(&self) -> Vec<&L> {
        let mut out = Vec::new();
        self.collect_leaves(&mut out);
        out
    }

    fn collect_leaves<'a>(&'a self, out: &mut Vec<&'a L>) {
        match self {
            BinaryTree::Leaf(v) => out.push(v),
            BinaryTree::Node(_, children) => {
                children[0].collect_leaves(out);
                children[1].collect_leaves(out);
            }
        }
    }
}
