# Grid variant 001 (V001 baseline)

# dp_knapsack_01_001

0/1 knapsack maximum-value benchmark. The faithfulness distinction against
unbounded knapsack (`SU5`) is deliberately pilot-level critical because
generation systems that omit index-uniqueness routinely produce
text-faithful/code-inconsistent specs.

## Design notes

- `totalWeight` and `totalValue` are opaque; their computational definition
  is pinned in proof scaffolds, not in the benchmark gold file.
- `ValidSelection` uses `Nodup` to encode the 0/1 constraint explicitly so
  annotators and generators cannot paper over it.
- Harness capacity range is capped at 50; the exhaustive 2^n oracle is
  feasible up to 10 items.
