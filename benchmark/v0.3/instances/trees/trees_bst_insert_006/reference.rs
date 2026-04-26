//! Reference implementation for `trees_bst_insert_006`.
//!
//! Immutable recursive BST insertion. Keys are `i32`; no-op on duplicate.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tree {
    Nil,
    Node(Box<Tree>, i32, Box<Tree>),
}

pub fn bst_insert(tree: Tree, key: i32) -> Tree {
    match tree {
        Tree::Nil => Tree::Node(Box::new(Tree::Nil), key, Box::new(Tree::Nil)),
        Tree::Node(left, k, right) => {
            if key < k {
                Tree::Node(Box::new(bst_insert(*left, key)), k, right)
            } else if key > k {
                Tree::Node(left, k, Box::new(bst_insert(*right, key)))
            } else {
                Tree::Node(left, k, right)
            }
        }
    }
}
