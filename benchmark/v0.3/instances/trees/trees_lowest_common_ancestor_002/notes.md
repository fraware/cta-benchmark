# Grid variant 002 (V002 paired control)

# trees_lowest_common_ancestor_002

Lowest common ancestor in a BST, given both keys are present. The
specification trap is the "lowest" part (SU4) — many generated specs
describe "some common ancestor" and thereby admit the trivial
"return the root" implementation.

## Design notes

- `IsSubtree` and `HasKey` are opaque predicates; their concrete definitions
  will live in proof-scaffold modules, keeping the benchmark gold file
  small and annotator-focused.
- `obl_003` uses a negative universal to express the lowest-property in a
  form that can be checked without committing to a particular notion of
  "descendant" beyond `IsProperSubtree`.
- The harness oracle brute-forces the LCA by computing root-to-leaf paths
  for both keys and taking the longest common prefix.
