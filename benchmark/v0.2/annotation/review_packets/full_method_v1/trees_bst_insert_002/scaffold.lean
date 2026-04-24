/-
Scaffold for instance `trees_bst_insert_002`.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Benchmark.Trees.BstInsertTheory

namespace CTA.Benchmark.Trees.BstInsert002

open CTA.Core
open CTA.Benchmark.Trees.BstInsertTheory

/-- Family-level tree model reused across BST-insert packets. -/
abbrev Tree := BstInsertTheory.Tree

/-- Shared in-order projection. -/
abbrev inorder := BstInsertTheory.inorder

/-- Shared BST invariant. -/
abbrev IsBst := BstInsertTheory.IsBst

/-- Shared key projection. -/
abbrev keys := BstInsertTheory.keys

/-- Shared insertion operator. -/
abbrev bstInsert := BstInsertTheory.bstInsert

end CTA.Benchmark.Trees.BstInsert002
