# SPIRE Delete Delta Diagnostics Coverage

## Checkpoint

- Code commit: `b330566a`
  (`Cover SPIRE delete delta diagnostics`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Focused PG18 coverage for delete-delta rows exposed by
  `ec_spire_index_delta_snapshot`

## Summary

This checkpoint proves the delta diagnostic's delete columns before full SQL
`VACUUM` coverage lands:

- Added a pg-test-only helper, `debug_spire_vacuum_bulkdelete_heap_tids`, that
  runs SPIRE `ambulkdelete` without immediately running `amvacuumcleanup`.
- Added focused PG18 SQL coverage that deletes one heap row, publishes a
  delete-delta epoch through the helper, and inspects the still-active
  delete-delta object.
- The test verifies `ec_spire_index_delta_snapshot(index_oid)` reports one
  delta object with zero insert assignments and one delete assignment.
- The test also verifies `ec_spire_index_scan_placement_snapshot(index_oid,
  query)` reports one `delete_delta_row_count` for the same pre-cleanup active
  epoch.
- Updated the Task 30 plan to distinguish this pre-cleanup delete-delta
  coverage from the still-open full SQL `VACUUM` end-to-end test.

This is test-only validation support. It does not change production vacuum
cleanup behavior, delete-delta publication, compaction, or physical tuple
reclamation.

## Changed Files

- `src/am/ec_spire/mod.rs`
- `src/am/ec_spire/vacuum.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_delta_snapshot_sql_delete_delta --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1102 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `222 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean before code commit

## Notes

- No measurement artifacts are included because this packet does not make a
  measurement claim.
- Full SQL `VACUUM` end-to-end coverage remains open because pgrx pg_tests run
  inside a transaction block and PostgreSQL rejects direct `VACUUM` there.
- The helper intentionally stops before cleanup so diagnostics can inspect the
  delete-delta object before normal compaction removes it.
