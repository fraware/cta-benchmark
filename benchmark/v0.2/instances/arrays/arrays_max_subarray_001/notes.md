# arrays_max_subarray_001

Kadane's algorithm for the maximum-sum contiguous subslice problem. Used as
a pilot for optimization-style problems where the correct specification must
capture both an existential witness and a universal upper bound.

## Design notes

- The non-emptiness precondition is load-bearing. Handling the empty-slice
  case as "return 0" is a distinct and incorrect specification.
- Faithful obligations must state both (a) a contiguous-subslice witness and
  (b) the universal upper bound. Stating only one is a common critical-unit
  coverage failure.
- The harness compares against a quadratic brute-force oracle on
  random small inputs, which is sufficient to catch most falsifications.
