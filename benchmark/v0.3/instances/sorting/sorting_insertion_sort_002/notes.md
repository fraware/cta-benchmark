# Grid variant 002 (V002 paired control)

# sorting_insertion_sort_002

Canonical stable-under-semantics sorting instance. Used to calibrate how
systems handle the two standard obligations of sorting (sortedness and
permutation) without being distracted by algorithmic cleverness.

## Design notes

- No stability obligation: annotators should flag any such claim as
  unfaithful/overspecified.
- The reference implementation mutates its slice in place; the Lean
  scaffold models the operation as a pure `List Int -> List Int` function
  for ease of reasoning.
