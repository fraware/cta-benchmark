/-
Scaffold for instance `trees_lowest_common_ancestor_005`.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Benchmark.Trees.LowestCommonAncestorTheory

namespace CTA.Benchmark.Trees.LowestCommonAncestor005

open CTA.Core
open CTA.Benchmark.Trees.LowestCommonAncestorTheory

abbrev Tree := LowestCommonAncestorTheory.Tree
abbrev inorder := LowestCommonAncestorTheory.inorder
abbrev IsBst := LowestCommonAncestorTheory.IsBst
abbrev HasKey := LowestCommonAncestorTheory.HasKey
abbrev IsSubtree := LowestCommonAncestorTheory.IsSubtree
abbrev IsProperSubtree := LowestCommonAncestorTheory.IsProperSubtree
abbrev subtreeRootedAt := LowestCommonAncestorTheory.subtreeRootedAt
abbrev lcaBst := LowestCommonAncestorTheory.lcaBst

end CTA.Benchmark.Trees.LowestCommonAncestor005
