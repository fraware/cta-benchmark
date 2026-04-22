## Authoring examples — gold obligations

This appendix collects worked examples of acceptable gold obligations
alongside common authoring failures. It is intended as the reference
guide for anyone adding a new instance to a future benchmark version
(for example during the v1.0 scale-up from 12 to 48 instances). Every
example is grounded in a real `benchmark/v0.1/instances/<domain>/<id>/`
directory so that the reader can open the corresponding
`reference_obligations.json` and `semantic_units.json` side by side.

All examples are paired with the specific `AUTHORING_*` lint code that
would catch the bad variant today. The authoring-heuristic linter is
invoked by:

```
cargo run -p cta_cli --quiet -- benchmark lint --version v0.1 --strict-authoring
```

CI runs this in `benchmark-lint.yml` on every push and PR that touches
`benchmark/**`.

### Shared vocabulary

- A *semantic unit* (SU) is one claim an informal statement makes about
  the algorithm. SUs are classified `critical`, `supporting`, or
  `optional` in `semantic_units.json`.
- A *gold obligation* is a Lean statement in `reference_obligations.json`
  that encodes one specific SU (or a small combination of SUs) as a
  formal property of the reference implementation.
- Every critical SU must be linked to at least one gold obligation
  (`AUTHORING_CRITICAL_SU_UNCOVERED`). Every obligation must link to at
  least one SU (`AUTHORING_OBLIGATION_NO_SEMANTIC_UNITS`).

The eight examples below cover the authoring failures we actually
observed during the `v0.1` audit plus a few adjacent traps.

---

### Example 1 — `arrays_binary_search_001`: success postcondition

**Instance**: `benchmark/v0.1/instances/arrays/arrays_binary_search_001/`
**Semantic unit**: `SU2` — "if the return is `Some(i)`, then `i` is a
valid index and `arr[i] == target`".

**Good** (current gold, `obl_001`):

```lean
∀ (arr : Arr) (t : Int) (i : Nat),
  binarySearch arr t = some i →
    i < arr.length ∧ arr.get? i = some t
```

Why acceptable:
- Both conjuncts are load-bearing: omitting `i < arr.length` lets a
  buggy implementation return an out-of-bounds index; omitting
  `arr.get? i = some t` lets it return any in-bounds index.
- The statement is a postcondition guarded by the return shape
  (`= some i`), not a universal claim over arbitrary `i`.

**Bad variant A** (bounds only):

```lean
∀ (arr : Arr) (t : Int) (i : Nat), binarySearch arr t = some i → i < arr.length
```

This is weaker than the SU: a function that returned
`Some 0` for every input would satisfy this statement on any non-empty
array. `AUTHORING_OBLIGATION_NO_SEMANTIC_UNITS` would not fire — the
obligation is still linked to `SU2` — but the audit checklist in
`docs/obligation_audit_v0.1.md` explicitly rejects such weakenings.

**Bad variant B** (equality only):

```lean
∀ (arr : Arr) (t : Int) (i : Nat), binarySearch arr t = some i → arr.get? i = some t
```

`arr.get? i = some t` silently encodes the bounds check because
`get?` returns `none` for out-of-range indices, which is why this
variant is sometimes defended. Prefer the conjunction: it is
independently checkable and matches the SU phrasing exactly.

---

### Example 2 — `arrays_binary_search_001`: failure postcondition with threaded precondition

**Instance**: `benchmark/v0.1/instances/arrays/arrays_binary_search_001/`
**Semantic units**: `SU1` (sortedness precondition) and `SU3`
(failure postcondition).

**Good** (current gold, `obl_002`):

```lean
∀ (arr : Arr) (t : Int),
  SortedLE arr → binarySearch arr t = none →
    ∀ i, i < arr.length → arr.get? i ≠ some t
```

Why acceptable:
- The precondition `SortedLE arr` appears exactly where it is used —
  in the antecedent of the postcondition that depends on it. Dropping
  the precondition would make the statement false for any
  implementation that correctly handles sorted input only.
- Linked to both `SU1` and `SU3`, which keeps `SU1` covered without a
  standalone precondition obligation.

**Bad variant — standalone precondition** (dropped under audit rule
R2, flagged by `AUTHORING_UNCONDITIONAL_PRECONDITION`):

```lean
∀ (arr : Arr) (t : Int), SortedLE arr
```

This is simply false as a universal statement. The original author
meant it as "assume sortedness", but a Lean obligation is a theorem,
not an assumption. Thread the precondition into the postcondition
instead.

---

### Example 3 — `graph_dijkstra_001`: conditional optimality

**Instance**: `benchmark/v0.1/instances/graph/graph_dijkstra_001/`
**Semantic units**: `SU1` (non-negative weights) and `SU5`
(optimality).

**Good** (current gold, `obl_003`):

```lean
∀ (n source v : Nat) (g : Graph) (d : Nat),
  NonNegativeWeights g →
  (dijkstra n source g).get? v = some (some d) →
  ∀ d', PathWeight g source v d' → d ≤ d'
```

Why acceptable:
- Dijkstra's optimality is genuinely conditional on non-negativity;
  without the hypothesis the statement is false. Threading it into the
  antecedent makes the obligation honest.
- One obligation covers two critical SUs (`SU1`, `SU5`). This
  consolidation is desirable when the SUs are semantically coupled,
  because it avoids a vacuous `∀ g, NonNegativeWeights g` precondition
  obligation.

**Bad variant — unconditional optimality** (would be false):

```lean
∀ (n source v : Nat) (g : Graph) (d : Nat),
  (dijkstra n source g).get? v = some (some d) →
  ∀ d', PathWeight g source v d' → d ≤ d'
```

A Dijkstra implementation run on a graph with negative edges does not
satisfy this statement. Omitting the precondition would turn the gold
theorem into an overstatement of the algorithm's contract.

---

### Example 4 — `sorting_insertion_sort_001`: sortedness postcondition

**Instance**: `benchmark/v0.1/instances/sorting/sorting_insertion_sort_001/`
**Semantic unit**: `SU1` (the output is sorted).

**Good**:

```lean
∀ xs : List Int, SortedLE (insertionSort xs)
```

Why acceptable:
- The statement is a universally quantified postcondition, not a
  precondition: there is no antecedent because the algorithm requires
  no preconditions. `∀ xs, SortedLE (insertionSort xs)` is not flagged
  by `AUTHORING_UNCONDITIONAL_PRECONDITION` because its `kind` is
  `postcondition`, not `precondition`.
- `SortedLE` is the weaker non-strict ordering. Authors sometimes
  reach for `Sorted` (strict) and thereby make the obligation false on
  any input containing duplicates.

**Bad variant — strict ordering** (false on duplicates):

```lean
∀ xs : List Int, Sorted (insertionSort xs)
```

**Bad variant — vacuous termination** (dropped under audit rule R1,
flagged by `AUTHORING_VACUOUS_TERMINATION`):

```lean
∀ xs : List Int, ∃ ys : List Int, insertionSort xs = ys
```

This is a tautology in total Lean for any total function. It provides
no signal about termination or correctness.

---

### Example 5 — `sorting_insertion_sort_001`: permutation postcondition

**Instance**: `benchmark/v0.1/instances/sorting/sorting_insertion_sort_001/`
**Semantic unit**: `SU2` (the output is a permutation of the input).

**Good**:

```lean
∀ xs : List Int, Perm xs (insertionSort xs)
```

Why acceptable:
- `Perm` rules out the trivial implementation that returns a constant
  sorted list and the lossy implementation that drops duplicates.
- Paired with the sortedness obligation, the two together form the
  classical specification of a sort. Neither alone is sufficient.

**Bad variant — multiset equality weakened to length**:

```lean
∀ xs : List Int, (insertionSort xs).length = xs.length
```

Length preservation is much weaker than permutation: it accepts a
sort that replaces every element with `0`. Never substitute a length
claim for a multiset claim when authoring a sort obligation.

---

### Example 6 — `trees_bst_insert_001`: threaded BST invariant

**Instance**: `benchmark/v0.1/instances/trees/trees_bst_insert_001/`
**Semantic units**: `SU1` (input is a BST), `SU2` (output preserves
the BST invariant).

**Good**:

```lean
∀ (t : Tree) (k : Int), IsBst t → IsBst (bstInsert t k)
```

Why acceptable:
- `IsBst t` appears as a hypothesis of the invariant-preservation
  postcondition, not as a standalone precondition obligation. This
  covers `SU1` via linkage while keeping the obligation a genuine
  theorem.

**Bad variant — standalone BST precondition** (dropped under audit
rule R2, flagged by `AUTHORING_UNCONDITIONAL_PRECONDITION`):

```lean
∀ (t : Tree) (k : Int), IsBst t
```

This is simply false for arbitrary `t`. The author presumably meant
"assume the input is a BST", which belongs in the antecedent of every
obligation that actually needs it.

---

### Example 7 — `greedy_interval_scheduling_001`: multi-hypothesis threading

**Instance**: `benchmark/v0.1/instances/greedy/greedy_interval_scheduling_001/`
**Semantic units**: `SU1` (intervals have positive length), `SU2`
(feasibility), `SU3` (optimality).

**Good** (feasibility postcondition):

```lean
∀ (ivs : List Interval),
  (∀ i ∈ ivs, i.start < i.stop) →
  Feasible (intervalSchedule ivs)
```

Why acceptable:
- The positive-length hypothesis is threaded into both the feasibility
  and the optimality postconditions. Neither postcondition is vacuous
  on the empty input (`Feasible []` is trivially true, but the optimality
  statement is still meaningful whenever `ivs` is non-empty).
- `SU1` is linked to the feasibility obligation rather than emitted as
  a standalone `∀ ivs i ∈ ivs, i.start < i.stop` obligation, which
  would be false.

**Bad variant — split precondition into a separate obligation** (would
be flagged by `AUTHORING_UNCONDITIONAL_PRECONDITION`):

```lean
∀ (ivs : List Interval) (i : Interval), i ∈ ivs → i.start < i.stop
```

This is an input assumption masquerading as a theorem. Thread it
instead.

---

### Example 8 — `graph_dijkstra_001`: unreachability postcondition linked to the right SU

**Instance**: `benchmark/v0.1/instances/graph/graph_dijkstra_001/`
**Semantic unit**: `SU6` (if `dist[v] == None` then there is no path
from source to `v`).

**Good**:

```lean
∀ (n source v : Nat) (g : Graph),
  (dijkstra n source g).get? v = some none →
  ∀ d, ¬ PathWeight g source v d
```

Why acceptable:
- The negation is universally quantified over `d`, correctly encoding
  "no path of any weight exists".
- Linked only to `SU6`, the unreachability unit. Linking this to `SU5`
  (optimality) would be a category error because optimality is about
  the weight of existing paths, not their existence.

**Bad variant — linked to the wrong SU** (would not fire a lint but
the audit checklist rejects it):

```json
{ "id": "obl_004", "linked_semantic_units": ["SU5"], ... }
```

The linter cannot catch every semantic mislinking, but the authoring
checklist in `docs/evaluation_contract.md#gold_obligations` requires
that every obligation's linked SUs match its logical content. Reviewer
PRs must check this by hand.

---

## Checklist for a new obligation

Before committing a new `reference_obligations.json`, walk this list
and confirm that the authoring lint passes with `--strict-authoring`:

1. Every `kind == "precondition"` obligation has a non-trivial
   antecedent (no bare `∀ x, P x`).
2. Every `kind == "termination"` obligation has a well-founded measure;
   vacuous existential shapes `∃ y, f x = y` are banned.
3. Every obligation links to at least one SU, and every critical SU is
   linked to at least one obligation.
4. The NL gloss precisely describes the Lean theorem — no
   strengthening or weakening of the quantifier structure.
5. Threaded preconditions appear in the antecedent of exactly the
   postconditions that depend on them.
6. For instances whose `informal_statement.required_properties`
   mentions `invariant`, `loop`, or `recursion`, at least one
   obligation encodes the invariant rather than only the top-level
   postcondition (`AUTHORING_NO_INVARIANT_STRUCTURE`).

Running `cargo run -p cta_cli --quiet -- benchmark lint --version v0.1
--strict-authoring` locally should produce zero issues before opening
a PR.
