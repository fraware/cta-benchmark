# Grid variant 001 (V001 baseline)

# greedy_interval_scheduling_001

Earliest-finish-time greedy for maximum-cardinality interval scheduling.
Used as a pilot optimization instance where the benchmark spec distinguishes
feasibility from optimality — models that state only one produce vacuous
specifications that the scorer must catch.

## Design notes

- The `NonOverlap` definition uses non-strict inequality on the shared
  boundary (`[a.start, a.stop)` and `[a.stop, b.stop)` are compatible).
  The instance's `obl_004` anchors this convention explicitly.
- `Feasible` is opaque: concrete implementations of the predicate live in
  future proof-scaffold modules, not in the benchmark gold file.
- The harness uses a 2^n exhaustive oracle; input sizes are capped at 16
  intervals.
