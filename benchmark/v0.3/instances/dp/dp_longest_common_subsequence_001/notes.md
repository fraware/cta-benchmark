# Grid variant 001 (V001 baseline)

# dp_longest_common_subsequence_001

Longest common subsequence length. A canonical DP benchmark where the
faithfulness trap is confusing "subsequence" with "substring". Models that
output code-consistent but text-inconsistent specs (i.e. longest common
substring) must be scored as failing `SU4`.

## Design notes

- `CommonSubseq` keeps the two index lists separate so that
  "same relative order" is represented as monotonicity of *both* lists, not
  monotonicity of their difference.
- `get!` is used in the scaffold to keep the predicate a `Prop` without
  lifting through `Option`; a small amount of partiality is acceptable here
  because the quantifiers condition on `m < ia.length` and the alignment
  conditions bound `ia.get! m < a.length`.
- The harness oracle is a second independent DP implementation, cross-run
  with permuted inputs for a symmetry check.
