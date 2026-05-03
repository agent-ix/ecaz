# SPIRE Insert Delta Epochs

## Checkpoint

- Code commit: `90b207e9` (`Publish SPIRE insert delta epochs`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: post-build `aminsert` delta publication

## Summary

This checkpoint implements the first strict local insert path for populated
relation-backed `ec_spire` indexes:

- `aminsert` now reuses SPIRE build tuple decoding for `ecvector`/`tqvector`
  indexed values and the configured assignment payload format.
- Inserts acquire a relation-level publish lock, read the active root/control
  state, decode the active manifests, and route the source vector to one leaf
  PID.
- The insert path writes a row-encoded `DELTA_INSERT` partition object for the
  target leaf PID, writes a full placement directory for the new epoch,
  persists manifest tuples, and advances root/control to the new active epoch.
- Relation-backed object storage can now write/read delta objects.
- Routed scans now include available delta insert objects whose parent PID is
  one of the probed leaves, then heap-rerank the combined candidate set.
- A focused PG18 test verifies post-build insert publication by checking epoch
  cursors and querying back the inserted row through an ordered `ec_spire` scan.

This does not implement empty-index insert bootstrap, delete/tombstone deltas,
vacuum cleanup, batching, or compaction.

## Changed Files

- `src/am/ec_spire/insert.rs`
- `src/am/ec_spire/scan.rs`
- `src/am/ec_spire/storage.rs`
- `src/am/ec_spire/build.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_insert_after_build_delta_epoch --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1078 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `198 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- This is not a recall/latency checkpoint.
- No measurement artifacts are included; validation is functional PG18 coverage
  only.
- Empty-index insert bootstrap, delete/vacuum cleanup, compaction, PQ-FastScan
  scorer binding, and SQL/admin diagnostics remain open.
