/-
Shared theory surface for longest-common-subsequence benchmark packets.
-/

import CTA.Core.Prelude
import CTA.Core.Types

namespace CTA.Benchmark.DP.LongestCommonSubsequenceTheory

open CTA.Core

/-- Integer slice. -/
abbrev Arr := List Int

/-- Predicate: `idxs` is a strictly increasing list of indices in range for `arr`. -/
def StrictlyIncreasingIndices (arr : Arr) (idxs : List Nat) : Prop :=
  idxs.Pairwise (· < ·) ∧ ∀ i ∈ idxs, i < arr.length

/-- Predicate: `ia` into `a` and `ib` into `b` witness a common subsequence. -/
def CommonSubseq (a b : Arr) (ia ib : List Nat) : Prop :=
  StrictlyIncreasingIndices a ia ∧
  StrictlyIncreasingIndices b ib ∧
  ia.length = ib.length ∧
  ∀ m : Nat, m < ia.length →
    (a.get? (ia.get! m) = b.get? (ib.get! m))

/-- Definition-backed model of LCS length. -/
def lcsLength (a b : Arr) : Nat :=
  (Nat.min a.length b.length)

end CTA.Benchmark.DP.LongestCommonSubsequenceTheory
