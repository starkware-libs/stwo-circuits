//! Binary tree data structure for in-circuit Merkle commitments.
//!
//! [`BinaryTree`] describes an arbitrary (possibly unbalanced) full binary tree of leaf hashes; its
//! shape drives the witness for the depth-independent Merkle commitment circuit (see
//! [`crate::unpacker`]).

/// A full binary tree: each `Leaf` carries a value of type `T`, and each `Node` carries its two
/// children (internal nodes hold no data of their own — the tree is pure shape). Arbitrary,
/// possibly unbalanced shapes (caterpillars, lone odd nodes carried up, etc.) are expressed
/// directly rather than inferred from a flat leaf list.
#[derive(Clone)]
pub enum BinaryTree<T> {
    Leaf(T),
    Node(Box<[BinaryTree<T>; 2]>),
}

impl<T> BinaryTree<T> {
    /// Maps every leaf through `f`, preserving the tree shape. `f` is `FnMut` so it may thread
    /// external state (e.g. guessing each leaf into a circuit context).
    pub fn map<U, F: FnMut(T) -> U>(self, f: &mut F) -> BinaryTree<U> {
        match self {
            BinaryTree::Leaf(v) => BinaryTree::Leaf(f(v)),
            BinaryTree::Node(children) => {
                let [left, right] = *children;
                BinaryTree::Node(Box::new([left.map(f), right.map(f)]))
            }
        }
    }

    /// Collects references to every leaf value in left-to-right (depth-first) order.
    pub fn leaves(&self) -> Vec<&T> {
        let mut out = Vec::new();
        self.collect_leaves(&mut out);
        out
    }

    fn collect_leaves<'a>(&'a self, out: &mut Vec<&'a T>) {
        match self {
            BinaryTree::Leaf(v) => out.push(v),
            BinaryTree::Node(children) => {
                children[0].collect_leaves(out);
                children[1].collect_leaves(out);
            }
        }
    }
}
