# greedy_coin_change_canonical_002

Greedy coin change restricted to canonical denomination systems. Chosen to
expose a classic specification trap: "minimum coins" is optimization, not
just feasibility, and the canonicality precondition is essential — without
it greedy fails on systems such as `[1, 3, 4]` for amount 6.

## Design notes

- `Canonical` is modeled as an opaque predicate. Reference obligations
  require annotators to keep the precondition even though implementations
  cannot check it cheaply.
- `obl_002` is classified as supporting because the alignment property is
  implementation-incidental; the real content lives in the decomposition and
  optimality obligations.
- Harness systems are all well-known canonical systems (US coins, Euro
  coins, powers of 3); the DP oracle cross-checks optimality.
