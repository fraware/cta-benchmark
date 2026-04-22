# trees_bst_insert_002

Immutable BST insertion with a duplicate no-op semantics. The faithfulness
nuance is that BST insertion "adds or leaves unchanged" — stating a plain
"key is inserted" postcondition is either incorrect (breaks uniqueness) or
vacuous depending on the model of the BST.

## Design notes

- `inorder` and `keys` are opaque so annotators reason about
  multiset-level properties via `List.Perm` without depending on a concrete
  traversal implementation.
- The duplicate-no-op obligation (`obl_004`) is marked `supporting` because
  it is a corollary of the more general set-change obligation (`obl_003`);
  annotators may accept either.
- The random-BST harness uses in-order + `bst_insert` to build candidates
  whose BST invariant is preserved by construction.
