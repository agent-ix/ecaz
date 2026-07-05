# Review Request: SPIRE Top-Level Graph Design

- Code commit: `3a1933e2` (`Design SPIRE top-level graph routing`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 6 top-level graph
- Agent: coder1

## Summary

This checkpoint starts Phase 6 with a durable graph-choice design note:

- adds `plan/design/spire-top-level-graph.md`;
- chooses a single-layer Vamana/DiskANN-style top graph over top-level SPIRE
  routing centroids;
- explicitly reuses the pure `ec_diskann` Vamana graph core rather than nesting
  an `ec_diskann` access method inside SPIRE;
- rejects HNSW for the first checkpoint because it adds another hierarchy on
  top of SPIRE's recursive hierarchy and is currently tied to heap-row graph AM
  lifecycle;
- defers build-time graph algorithm selection until SPIRE has a second real
  graph implementation;
- defines the first graph node model, epoch/object placement boundary, routing
  fallback behavior, diagnostics, and measurement gate.

No runtime code changes are included in this checkpoint.

## Files

- `plan/design/spire-top-level-graph.md`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

1. Confirm Vamana/DiskANN-style single-layer graph is the right first SPIRE
   top-level graph choice.
2. Check that reusing only the pure `ec_diskann` graph core, not the
   `ec_diskann` AM storage/callback lifecycle, is the correct boundary.
3. Confirm the graph object should be a SPIRE epoch object rather than a nested
   PostgreSQL child index.
4. Review the scan fallback rules for missing, unavailable, or malformed graph
   objects.
5. Check whether the design's recursive build interaction is clear enough:
   initial code may validate graph plumbing over the current root child set,
   but the target shape needs a larger graph-searchable top routing child set.

## Validation

- `git diff --check`

No tests were run because this is a design-only checkpoint.
