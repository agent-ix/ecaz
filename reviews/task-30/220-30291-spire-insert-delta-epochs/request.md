# SPIRE Insert Delta Epochs

## Checkpoint

- Code commit: `90b207e9` (`Publish SPIRE insert delta epochs`)
- Feedback response commit: `68102466`
  (`Cover SPIRE insert delta feedback`)
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

## Feedback Response

Reviewer feedback in `feedback.md` flagged the one-epoch-per-insert shape and
the missing multi-insert / same-leaf delta coverage. Response commit
`68102466`:

- Added a focused PG18 test for a multi-row post-build insert with `nlists = 1`,
  forcing three insert-delta epochs onto the same base leaf.
- Asserted the expected root/control cursor movement: build epoch plus three
  insert epochs, one delta PID per inserted row, and one local `vec_id` per
  inserted row.
- Asserted active diagnostics report three delta objects and three delta
  assignments.
- Asserted scan-placement diagnostics for the query touch all three delta PIDs
  and all three delta candidate rows.
- Kept insert batching/concurrent publish work explicitly open in the Task 30
  plan.

Additional validation for `68102466`:

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_insert_after_build_multiple_same_leaf_deltas --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1094 filtered out`
- `cargo test --lib test_ec_spire_insert_after_build_delta_epoch --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1094 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `214 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean before response commit
