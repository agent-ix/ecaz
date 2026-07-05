# SPIRE Hierarchy Diagnostics

## Checkpoint

- Code commit: `2068f363`
  (`Expose SPIRE hierarchy diagnostics`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: SQL diagnostics for active SPIRE hierarchy metadata and recursion
  deferral

## Summary

This checkpoint exposes the current active hierarchy shape without implementing
recursive SPIRE build or scan routing:

- Added `ec_spire_index_hierarchy_snapshot(index_oid)` as a stable, strict SQL
  table function that returns one summary row.
- Empty indexes now report `status = 'empty'`, zero hierarchy counts, and
  `recursive_routing_supported = false`.
- Populated single-level indexes report the active root PID/level, observed
  max level/depth, routing/root/internal/leaf/delta object counts, centroid
  dimensions, root child count, and distinct leaf parent count.
- The diagnostic reports `recursive_routing_supported = false` and
  `per_level_nprobe_supported = false` so Phase 3 hierarchy gaps are visible
  before the recursive build coordinator lands.
- Added focused PG18 coverage for empty and populated local single-store
  hierarchy snapshots.
- Updated the Task 30 plan to record that root/leaf levels, parent/child PIDs,
  and root centroid dimensions are already persisted in the single-level
  foundation while recursive routing and per-level `nprobe` remain open.

This is read-only observability. It does not add internal routing objects,
multi-level build coordination, per-level `nprobe` storage, or recursive scan
routing.

## Changed Files

- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_hierarchy_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1099 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `219 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean before code commit

## Notes

- No measurement artifacts are included because this packet does not make a
  measurement claim.
- The current status label for populated local indexes is
  `single_level_foundation`.
- Recursive hierarchy construction, level-local scan routing, and per-level
  scan options remain follow-up work.
