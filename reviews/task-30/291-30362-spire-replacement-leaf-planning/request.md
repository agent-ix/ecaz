# SPIRE Replacement Leaf Planning

## Checkpoint

- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE Phase 2 update mechanics
- Scope: Pure replacement-leaf planning helpers for split/merge/rebalance

## Summary

This checkpoint starts Phase 2 implementation with a pure helper boundary in
`src/am/ec_spire/update.rs`.

The new helper surface records the reviewed replacement rules before live
scheduler or relation publish wiring lands:

- split replacement requires one affected leaf and allocates two or more new
  leaf PIDs from the same PID allocator cursor used by root/control
- merge replacement allocates one new leaf PID from that cursor
- rebalance reuses the affected PID only when the parent-routing centroid is
  byte-equal; otherwise it fails closed so callers must model the operation as
  a coverage rewrite
- replacement row collection reads the active epoch snapshot, folds active
  insert/delete deltas into replacement base-leaf rows, clears delta-insert
  flags on survivors, and rejects affected PIDs that are not active leaves

This does not implement a scheduler, root routing rewrite, or publish path for
split/merge yet.

## Changed Files

- `src/am/ec_spire/update.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test replacement_ --lib`
- `git diff --check`

The focused unit filter also triggered the crate's existing PG18 pgrx feature
build and two filtered PG tests whose names contain `replacement`; all passed.

## Notes

- No measurement claims.
- PQ-FastScan populated support remains deferred.
- Remote placement and replicas remain deferred.
