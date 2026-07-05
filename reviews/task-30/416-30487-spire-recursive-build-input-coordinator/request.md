# Review Request: SPIRE Recursive Build Input Coordinator

Head SHA: `56c81288`

## Summary

This checkpoint adds the pure coordinator step between first-level IVF training
and recursive epoch object writing.

The new coordinator input assembler:

- validates the first-level centroid plan and assignment count;
- allocates stable leaf PIDs before routing PID allocation;
- groups primary assignment rows by centroid assignment;
- builds recursive routing hierarchy input from the leaf centroids;
- derives each leaf object's parent PID from the recursive routing draft; and
- returns a recursive epoch object input plus next PID / local vector cursors for
  the future relation publisher.

The existing single-level relation/local grouping code now shares the same
checked assignment-by-centroid helper.

## Files

- `src/am/ec_spire/build.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test recursive_ -- --nocapture`
  - 24 passed, including the new recursive coordinator tests.
- `cargo test partitioned -- --nocapture`
  - 4 passed, covering the shared single-level partition grouping helper.
- `git diff --check`

No PG18 SQL test was run for this pure build-coordinator assembly slice.

## Review Focus

- Confirm leaf PID allocation before routing PID allocation is the right durable
  shape for later relation-backed recursive builds.
- Confirm `source_count: 1` for first-level leaf-centroid routing children is the
  intended interpretation: each child represents one trained centroid, not the
  number of heap rows assigned to that leaf.
- Check whether the coordinator draft exposes the right cursor state for the
  next relation publish step.
