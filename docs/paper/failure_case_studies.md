# Failure case studies (manuscript pool)

These examples explain **why theorem-shaped Lean can still be semantically unfaithful**—the core motivation for CTA-Bench. Use **two** in the main paper and move the rest to the appendix.

Concrete obligation text for each packet lives under `benchmark/v0.3/annotation/review_packets/<system_id>/<instance_id>/packet.json` (field `generated_obligations`). Semantic units are authoritative in each instance’s `benchmark/v0.3/instances/<family>/<instance_id>/semantic_units.json`.

---

## 1. Binary search — success assumes the hit; absence is restated, not proved

| Field | Content |
|--------|---------|
| **instance_id** | `arrays_binary_search_002` (representative; same failure pattern appears across the family grid) |
| **system_id** | `full_method_v1` (illustrative; check adjudicated rows in `results/paper_strict_instance_level.csv`) |
| **Generated theorem shape (pattern)** | A main theorem assumes `Some idx` and proves a disjunction-style consequence without deriving the **absence** case from search invariants; an auxiliary “absence” statement collapses to `binarySearch = none` without proving **∀ i, arr[i] ≠ target**. |
| **Missing semantic units** | Typically **SU3** (“if `None`, target does not occur anywhere in the slice”) and sometimes weakened linkage to **SU2** (index validity + `arr[i] == target` on success). |
| **Why a human might be fooled** | The file contains plausible loop invariants and `Option` reasoning; skimming suggests “binary search correctness” even when the **global absence quantifier** never appears. |
| **Correct obligation sketch** | Success: valid index + equality; failure: **forall indices**, membership contradicts sorted structure / search invariant; optionally termination (**SU4**) as supporting. |
| **Annotation labels (typical)** | Low semantic faithfulness or missing critical unit on SU3; proof utility may still look high if syntax elaborates. |
| **failure_mode_label** | `missing_critical_semantic_unit` or `low_semantic_faithfulness` (strict exports in `results/paper_strict_failure_modes.csv`). |
| **Source artifact path** | `benchmark/v0.3/instances/arrays/arrays_binary_search_002/semantic_units.json`; adjudicated metrics in `results/raw_metrics_strict.json`. |

---

## 2. LCS — length exists without witness or maximality linkage

| Field | Content |
|--------|---------|
| **instance_id** | `dp_longest_common_subsequence_003` |
| **system_id** | `naive_concat_v1` or `text_only_v1` (high missing-unit rate in strict aggregates; confirm per row). |
| **Generated theorem shape (pattern)** | States `∃ k, k = lcsLength a b` or a tight length bound **without** a **common-subsequence witness** indexed into both inputs, and **without** the **maximality** link (“no common subsequence longer than `k`”). |
| **Missing semantic units** | Witness + alignment to DP table semantics; maximality / upper-bound argument tied to subproblem decomposition (see instance `reference_obligations.json`). |
| **Why a human might be fooled** | “Length exists” mirrors textbook prose; without reading carefully, the theorem reads like “defines LCS” rather than “characterizes optimum.” |
| **Correct obligation sketch** | Exhibit indices/lists witnessing common subsequence of length `k`; show **any** common subsequence has length ≤ `k` (often via standard recurrence facts). |
| **Annotation labels** | Missing critical units; partial coverage; faithfulness often collapsed via rubric. |
| **failure_mode_label** | `missing_critical_semantic_unit`. |
| **Source artifact path** | `benchmark/v0.3/instances/dp/dp_longest_common_subsequence_003/semantic_units.json`; strict row in `results/paper_strict_instance_level.csv`. |

---

## 3. Dijkstra — distances listed without path-weight semantics

| Field | Content |
|--------|---------|
| **instance_id** | `graph_dijkstra_004` |
| **system_id** | `full_method_v1` (representative for polished but detached obligations). |
| **Generated theorem shape (pattern)** | Table-level statements mention distance entries without simultaneously tying them to **achievable path weights**, **optimality** vs any other walk, and **unreachability** (`none`) vs negative/absurd cases where the benchmark expects explicit separation. |
| **Missing semantic units** | Optimality (`∀ path weight …`), reachability / PathWeight predicates as in `reference_obligations.json`, separation for “no path”. |
| **Why a human might be fooled** | Mathlib-style `Finset` distance boilerplate looks authoritative; reviewers must check whether output distances are **connected** to path existence and weight ordering. |
| **Correct obligation sketch** | Standard: predecessor relaxation invariants ⇒ upon termination, stored distance ≤ any path weight; equality witnesses shortest path; `none` ⇔ no path under stated side conditions. |
| **Annotation labels** | Low faithfulness with partial proof utility; vacuity checks matter when predicates detach. |
| **failure_mode_label** | `low_semantic_faithfulness` and/or `missing_critical_semantic_unit`. |
| **Source artifact path** | `benchmark/v0.3/instances/graph/graph_dijkstra_004/reference_obligations.json`; metrics in `results/raw_metrics_strict.json`. |

---

## 4. Sorting — sorted output without permutation / multiset preservation

| Field | Content |
|--------|---------|
| **instance_id** | `sorting_merge_sort_004` |
| **system_id** | `text_only_v1` (often omits preservation obligations). |
| **Generated theorem shape (pattern)** | Proves `List.Sorted` (or pairwise order) on output **without** **permutation** / multiset equality / stability facts required by the benchmark contract. |
| **Missing semantic units** | “Output is a permutation of input” / multiset preservation; sometimes stability only where marked critical. |
| **Why a human might be fooled** | Sorted output is the salient textbook property; readers anchor on order and skip preservation. |
| **Correct obligation sketch** | Sorted order **and** `output ~ input` as multisets (or explicit bijection of positions), plus family-specific preconditions. |
| **Annotation labels** | Missing units on preservation; coverage `partial` common. |
| **failure_mode_label** | `missing_critical_semantic_unit`. |
| **Source artifact path** | `benchmark/v0.3/instances/sorting/sorting_merge_sort_004/semantic_units.json`; failure aggregates in `results/paper_strict_failure_modes.csv`. |
