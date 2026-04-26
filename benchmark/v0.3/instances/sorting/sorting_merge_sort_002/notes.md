# Grid variant 002 (V002 paired control)

# sorting_merge_sort_002

Top-down merge sort producing a new sorted vector. Companion to
`sorting_insertion_sort_001` — identical post-conditions, different algorithm
structure (divide-and-conquer vs. in-place iteration), so the two instances
stress different Rust semantic-extraction motifs.

## Design notes

- Faithful specs need three properties: length, sortedness, permutation.
  Common incorrect specs forget the permutation property and end up vacuous
  against a `[]`-returning implementation.
- Stability is intentionally NOT part of the spec. Claiming stability would
  be a faithfulness failure, not a coverage one.
- The harness compares against a `Vec::sort` oracle.
