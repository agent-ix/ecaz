# SPIRE Delta Diagnostics

## Checkpoint

- Code commit: `172c9eea`
  (`Expose SPIRE delta diagnostics`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: SQL diagnostics for active readable delta partition objects

## Summary

This checkpoint exposes active delta partition objects directly:

- Added `ec_spire_index_delta_snapshot(index_oid)` as a stable, strict SQL
  table function.
- Empty active epochs and populated no-delta indexes return zero rows.
- Post-build insert epochs report the active delta PID, parent leaf PID,
  object version, published-epoch back-reference, store placement, assignment
  count, insert assignment count, delete assignment count, and object bytes.
- The focused SQL test verifies that a post-build insert publishes one active
  readable delta object, that it points at an active leaf PID, and that it
  reports one insert assignment and zero delete assignments.
- Updated the Task 30 plan to record this per-delta diagnostic alongside the
  existing leaf/object/insert-debt surfaces.

This is read-only observability. It does not change delta publication,
insert batching, delete-delta publication, vacuum compaction, or physical
cleanup.

## Changed Files

- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_delta_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1101 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `221 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean before code commit

## Notes

- No measurement artifacts are included because this packet does not make a
  measurement claim.
- The function is active-epoch scoped and intentionally skips non-available
  placements because it cannot prove object kind without reading the object
  header.
- Delete-delta rows are represented in the shape, but this checkpoint's
  focused SQL test covers the post-build insert-delta path only.
