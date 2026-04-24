# Gold-obligation audit — `v0.1`

This document records the audit performed over every
`reference_obligations.json` in `benchmark/v0.1/instances/**` during the
`v0.1` freeze and the exact rewrites applied. It is the primary evidence
for the "gold obligations have been audited for vacuity and semantic
correctness" green-light criterion in `docs/evaluation_contract.md`.

## Rewrite rules

Two rules were applied uniformly across all 12 instances.

## `v0.2` carry-forward note (`2026-04-24`)

The same anti-vacuity principles were applied to curated `v0.2` review packets
for target families (`sorting_insertion_sort_{001,002}`,
`sorting_merge_sort_{001,002}`, `trees_bst_insert_{001,002}`):

- remove/replace vacuous benchmark-facing placeholders,
- replace trusted-interface artifacts (`axiom` packet statements) with
  theorem-backed statements against concrete family theory definitions.

Validation continues to run through strict refresh and packet regression tests.

### R1 — drop vacuous existential termination obligations

Statements of the shape `∀ x, ∃ y, f x = y` are vacuously true in total
Lean for any total function, so they provide no discriminative signal.
Termination as a *semantic* property cannot be captured by a
post-existential; it requires either a well-founded variant or a
measure argument. Since the `v0.1` scaffolds do not yet ship such
infrastructure (see `lean/CTA/Core/Checkers.lean`), obligations of this
shape are dropped entirely for the pilot.

The corresponding termination semantic units (`SU4` or analogue) remain
in `semantic_units.json` as *declared* aspects of the problem but are
intentionally left unlinked from any gold obligation. The authoring
heuristic linter (phase I) warns on an authored obligation matching
`∃ .*, f .* = .*`, so future contributions cannot reintroduce the
pattern silently.

### R2 — no standalone unconditional precondition obligations

Precondition obligations of the form `∀ x, P x` assert `P` as a
universal truth, which is either a tautology (and therefore vacuous) or
simply false (e.g. `∀ t, IsBst t`). Preconditions belong in the
antecedent of the postcondition they guard, not as free-standing
theorems. Each flagged precondition obligation was dropped, and its
hypothesis was threaded into every downstream postcondition whose
correctness genuinely depends on it. Semantic units that previously
linked only to the dropped precondition were re-linked to the first
downstream postcondition that now carries the precondition in its
antecedent, preserving critical-unit coverage.

## Per-instance changes

Each entry below lists the obligations dropped (with reason code) and
obligations rewritten (old shape → new shape). Obligation ids were
renumbered to keep `obl_NNN` contiguous after deletions; the semantic
identity of each surviving obligation is unchanged modulo the threaded
precondition.

### `arrays/arrays_binary_search_001`

- Dropped `obl_001` (`∀ arr t, SortedLE arr`) under R2. Sortedness is
  threaded into the surviving None-branch obligation; the
  Some-branch obligation does not depend on it.
- Dropped `obl_004` (`∀ arr t, ∃ r, binarySearch arr t = r`) under R1.
- SU1 (sortedness) is now covered by the None-branch postcondition.
- SU4 (termination) is unlinked by design.

### `arrays/arrays_max_subarray_001`

- Dropped `obl_001` (`∀ arr, arr ≠ []`) under R2. Non-emptiness was
  already threaded into both downstream postconditions as
  `arr ≠ [] →`; no rewrite of the downstream statements was needed.
- Dropped `obl_004` (`∀ arr, arr ≠ [] → ∃ v, maxSubarray arr = v`)
  under R1.
- SU1 (non-emptiness) is now covered by the existence postcondition.

### `dp/dp_knapsack_01_001`

- Dropped `obl_001` (`∀ ws vs, ws.length = vs.length`) under R2. Length
  alignment is threaded into both surviving postconditions as
  `ws.length = vs.length →`.
- SU1 (alignment) is now covered by the existence postcondition.

### `dp/dp_longest_common_subsequence_001`

- No rewrites applied. All obligations are postconditions with
  non-vacuous content.

### `graph/graph_bfs_shortest_path_001`

- No rewrites applied. All obligations are postconditions with
  non-vacuous content.

### `graph/graph_dijkstra_001`

- Rewrote `obl_003` (optimality) to thread the non-negativity hypothesis:
  add `NonNegativeWeights g →` in the antecedent and link `SU1` (input
  precondition: edge weights in range and non-negative) alongside `SU5`
  (optimality). See `docs/authoring_examples.md` (Example 3) for discussion
  of the same shape. Achievability (`obl_004`, `SU4`) and the other
  postconditions were unchanged.

### `greedy/greedy_coin_change_canonical_001`

- Dropped `obl_001` (`∀ denoms, Canonical denoms`) under R2.
  Canonicity is threaded into the optimality postcondition only;
  decomposition and alignment hold regardless of canonicity.
- SU1 (canonicity) is now covered by the optimality postcondition.

### `greedy/greedy_interval_scheduling_001`

- Dropped `obl_001` (`∀ ivs i ∈ ivs, i.start < i.stop`) under R2.
  Positive-length is threaded into the feasibility and optimality
  postconditions.
- SU1 (positive-length) is now covered by the feasibility
  postcondition.

### `sorting/sorting_insertion_sort_001`

- Dropped `obl_003` (`∀ xs, ∃ ys, insertionSort xs = ys`) under R1.
- SU3 (termination) is unlinked by design.

### `sorting/sorting_merge_sort_001`

- Dropped `obl_004` (`∀ arr, ∃ r, mergeSort arr = r`) under R1.

### `trees/trees_bst_insert_001`

- Dropped `obl_001` (`∀ t k, IsBst t`) under R2. `IsBst t →` was already
  threaded into every surviving postcondition; no downstream rewrite
  needed.
- SU1 (BST invariant) is now covered by the invariant-preservation
  postcondition.

### `trees/trees_lowest_common_ancestor_001`

- Dropped `obl_001` (`∀ t p q, IsBst t ∧ HasKey t p ∧ HasKey t q`)
  under R2. The three hypotheses are threaded into every surviving
  postcondition (`IsBst t → HasKey t p → HasKey t q →`). The
  self-query postcondition uses only two of the three.
- SU1 and SU2 (BST invariant, key presence) are now covered by the
  subtree-root postcondition.

## Consequences

- All four vacuous termination obligations are removed; no surviving
  obligation matches the `∃ .*, f .* = .*` shape.
- No surviving obligation is an unconditional precondition; every
  precondition is a hypothesis of at least one guarded postcondition.
- The total obligation count dropped from 49 to 35 across the 12
  instances; the benchmark manifest is regenerated accordingly.
- Every critical semantic unit remains linked to at least one
  obligation. Supporting termination SUs are intentionally unlinked.
