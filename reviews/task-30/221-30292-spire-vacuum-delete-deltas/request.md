# SPIRE Vacuum Delete Deltas

## Checkpoint

- Code commit: `a60a49c3` (`Publish SPIRE vacuum delete deltas`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: populated-index vacuum delete-delta publication and routed scan
  suppression

## Summary

This checkpoint implements the first strict local delete path for populated
relation-backed `ec_spire` indexes:

- `ambulkdelete` now reads the active root/control state and published
  epoch/object/placement manifests for populated active indexes.
- Vacuum walks visible base-leaf and delta-insert assignments, applies the
  PostgreSQL bulk-delete callback to stored heap TID locators, and groups dead
  assignments by base leaf PID.
- The delete path writes row-encoded `DELTA_DELETE` partition objects, carries
  prior placements forward into a new strict epoch, persists placement and
  manifest tuples, and advances root/control to the replacement active epoch.
- Active routed scans now collect delete deltas for each probed base leaf PID
  and suppress covered `vec_id`s from both base V2 leaf objects and routed
  delta-insert objects.
- `amvacuumcleanup` reports live assignment counts from the active snapshot
  after delete-delta suppression.
- A focused PG18 test verifies delete-delta epoch publication, vacuum stats,
  root/control cursors, and ordered scan suppression.

This does not implement physical cleanup, compaction into rewritten V2 base
objects, empty-index insert bootstrap, batching, or full SQL VACUUM
end-to-end coverage.

## Changed Files

- `src/am/ec_spire/vacuum.rs`
- `src/am/ec_spire/scan.rs`
- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_vacuum_delete_delta_suppresses_visible_row --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1079 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `199 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- This is not a recall/latency checkpoint.
- No measurement artifacts are included; validation is functional PG18 coverage
  only.
- Physical object cleanup, compaction, empty-index insert bootstrap,
  PQ-FastScan scorer binding, SQL/admin diagnostics, and full SQL VACUUM
  end-to-end coverage remain open.
