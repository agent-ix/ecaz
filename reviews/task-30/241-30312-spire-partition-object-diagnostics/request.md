# SPIRE Partition Object Diagnostics

## Checkpoint

- Code commit: `d8480bfb`
  (`Expose SPIRE partition object diagnostics`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: SQL diagnostics for active PID-addressed partition objects

## Summary

This checkpoint exposes active SPIRE partition objects directly, one row per
active manifest PID:

- Added `ec_spire_index_object_snapshot(index_oid)` as a stable, strict SQL
  table function.
- Empty active epochs return zero rows.
- Populated single-level indexes report root and leaf partition objects with
  object kind, object version, published-epoch back-reference, level,
  parent PID, child/assignment counts, placement state, store identity, object
  bytes, and an `object_readable` flag.
- Post-build insert epochs expose the active delta partition object alongside
  carried-forward root/leaf objects, making delta fanout inspectable through
  the active manifest.
- Updated the Task 30 plan to record this per-PID diagnostic surface alongside
  the existing aggregate placement, leaf, root-routing, hierarchy, and epoch
  views.

This is read-only observability. It does not alter object storage, placement
publication, delta compaction, or recursive hierarchy behavior.

## Changed Files

- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_object_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1100 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `220 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean before code commit

## Notes

- No measurement artifacts are included because this packet does not make a
  measurement claim.
- In degraded snapshots, unavailable or skipped placements remain listed with
  `object_readable = false` and `object_kind = 'unreadable'`.
- The surface is active-epoch scoped and intentionally does not enumerate
  retired or superseded epoch object tuples.
