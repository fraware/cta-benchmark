//! Reference implementation for `trees_lowest_common_ancestor_007`.
//!
//! BST LCA in O(height). Assumes both keys are present in the tree.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tree {
    Nil,
    Node(Box<Tree>, i32, Box<Tree>),
}

pub fn lca_bst(tree: &Tree, p: i32, q: i32) -> Option<i32> {
    let (lo, hi) = if p <= q { (p, q) } else { (q, p) };
    let mut cur = tree;
    loop {
        match cur {
            Tree::Nil => return None,
            Tree::Node(left, k, right) => {
                if hi < *k {
                    cur = left;
                } else if lo > *k {
                    cur = right;
                } else {
                    return Some(*k);
                }
            }
        }
    }
}
