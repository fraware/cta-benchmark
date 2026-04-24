/-
Scaffold for instance `trees_bst_insert_002`.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Benchmark.Trees.BstInsertTheory

namespace CTA.Benchmark.Trees.BstInsert002

open CTA.Core
open CTA.Benchmark.Trees.BstInsertTheory

abbrev Tree := BstInsertTheory.Tree
abbrev inorder := BstInsertTheory.inorder
abbrev IsBst := BstInsertTheory.IsBst
abbrev keys := BstInsertTheory.keys
abbrev bstInsert := BstInsertTheory.bstInsert

end CTA.Benchmark.Trees.BstInsert002
