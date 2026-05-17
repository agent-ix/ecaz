# Review Request: SPIRE Split Replacement Boundary Fanout

- Code commit: `61b4816e` (`Fan out SPIRE split replacement boundary rows`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 5 boundary replication
- Agent: coder1

## Summary

This checkpoint extends boundary replica assignment fanout into scheduled split
replacement materialization.

Split replacement already fetches source vectors and trains replacement
centroids. It now also receives `boundary_replica_count` from the relation
options and routes each normalized source row to the top replacement centroids.
The first routed replacement leaf receives a `PRIMARY` row, and bounded
secondary leaves receive `BOUNDARY_REPLICA` rows with the same `vec_id`.

Replacement leaf input validation now accepts this replica shape:

- every physical replacement row must be visible-scored and non-delta;
- each physical leaf can contain a given `vec_id` at most once;
- each replicated `vec_id` must still have exactly one primary row.

Merge replacement remains primary-only because the scheduler publishes exactly
one replacement leaf for merge decisions.

## Review Focus

1. Confirm split replacement uses the intended top-N centroid predicate for
   replacement leaves.
2. Check that validation allows primary plus boundary rows for the same
   `vec_id` without allowing duplicate rows in a single leaf or multiple
   primaries.
3. Verify threading `boundary_replica_count` through the relation scheduled
   split path is enough for the current maintenance callback surface.
4. Confirm documenting merge as primary-only is appropriate while merge has one
   replacement leaf.

## Validation

- `cargo test --lib split_replacement_materialization_fans_out_boundary_rows --no-default-features --features pg18`
- `cargo test --lib split_replacement_materialization --no-default-features --features pg18`
- `cargo test --lib replacement_leaf_object_inputs --no-default-features --features pg18`
- `cargo test --lib selected_scheduled_split --no-default-features --features pg18`
- `cargo test --lib split_replacement --no-default-features --features pg18`
- `git diff --check`

PG17 was not run; this slice is centered on the PG18 SPIRE Phase 5 replacement
publication path.
