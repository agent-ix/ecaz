# SPIRE Health Diagnostics SQL

## Checkpoint

- Code commit: `fdf5661b` (`Expose SPIRE health diagnostics`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: SQL/admin health summary for active relation-backed `ec_spire`
  snapshots

## Summary

This checkpoint adds a conservative health/recommendation surface on top of
the existing persisted SPIRE snapshot diagnostics:

- Added `ec_spire_index_health_snapshot(index_oid)` as a stable, strict SQL
  table function for `ec_spire` indexes.
- The function validates the supplied OID as an `ec_spire` index and derives
  health state from the active root/control state plus active snapshot
  diagnostics.
- The row reports active epoch, consistency mode, status, healthy flag,
  recommendation text, compaction recommendation flag, object count,
  assignment counts, delta object count, and placement-state counts.
- Status is intentionally conservative:
  - empty active epoch reports `empty`.
  - unavailable, stale, or skipped placements report unhealthy placement
    statuses.
  - active delta objects report `maintenance_recommended` with
    `compaction_recommended = true`.
  - clean strict active snapshots report `ok`.
- A focused PG18 test verifies clean populated-build health and delta-pending
  maintenance recommendations after a live insert publishes a delta epoch.
- The Task 30 plan now records the health SQL surface while keeping deeper
  operator guidance, recall/latency evidence, physical cleanup, and real SQL
  `VACUUM` end-to-end coverage open.

This does not implement physical page reclamation, old-epoch cleanup, real SQL
`VACUUM` end-to-end validation, recall/latency summary rows, or PQ-FastScan
scorer binding.

## Changed Files

- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_health_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1083 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `203 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- This is not a recall/latency checkpoint.
- No measurement artifacts are included; validation is functional PG18 coverage
  only.
- The recommendation strings are conservative status guidance, not an automated
  maintenance scheduler.
