# Grid variant 001 (V001 baseline)

# arrays_binary_search_001

Classical binary search on a sorted slice. Intentionally easy; used as a
calibration pilot for annotators and as a smoke-test instance for every
generation system.

## Design notes

- Duplicates are allowed. Returning any valid index is correct.
- The termination obligation (`obl_004`) is phrased as existence of a
  result rather than a well-founded measure to keep it independent of the
  particular implementation.
- Edge cases include empty slices, single-element slices, and targets that
  are smaller than all elements or larger than all elements. The harness
  must exercise each of these.
