# Review Request: Task 28 IVF Drift Snapshot

Scope: Phase 6 drift checkpoint. IVF now exposes queryable centroid-staleness
and REINDEX-pressure indicators through `ec_ivf_index_drift_snapshot(regclass)`.

Task: `plan/tasks/28-ivf-access-method.md` Phase 6

Branch: `task28-ivf`

Head SHA: `5b5c8bc18e9a7370c7363ed8201594f3df999f5f`

Owner: coder2

Files:

- `src/am/ec_ivf/admin.rs`
- `src/am/ec_ivf/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo check --no-default-features --features pg18 --tests`
- `cargo pgrx test pg18 test_ec_ivf_drift_snapshot_handles_empty_index`
- `cargo pgrx test pg18 test_ec_ivf_drift_snapshot_tracks_insert_and_vacuum_churn`
- `git diff --cached --check`

Validation notes:

- Validation was PG18-only per the current user direction to focus on PG18.
- The PG tests were run against PostgreSQL 18.3 through pgrx.
- No measurement claim is made in this packet.

## Summary

This slice closes the Phase 6 drift-snapshot item:

- Adds `ec_ivf_index_drift_snapshot(regclass)`.
- Reports live/dead tuple totals, inserted-since-build, changed-row fraction,
  average/max list live counts, list imbalance ratio, and empty-list count.
- Reports REINDEX recommendation thresholds and reason strings for changed-row
  churn and list imbalance.
- Handles empty IVF indexes without requiring a directory.
- Covers insert drift followed by vacuum churn in PG18.

## Review Focus

Please review for:

- Whether `changed_row_fraction = (inserted_since_build + total_dead_tuples) /
  (total_live_tuples + total_dead_tuples)` is the right operational definition.
- Whether the initial thresholds, `0.20` changed-row fraction and `4.0` list
  imbalance ratio, are reasonable defaults for a first observable surface.
- Whether this should stay as a drift-specific function or be merged into the
  future Phase 7 IVF admin snapshot.
- Whether list imbalance should use all lists, as implemented, or only nonempty
  lists.

## Non-Goals

This packet does not implement planner costing, EXPLAIN counters, a full IVF
admin snapshot, measurement sweeps, SQL `VACUUM` coverage, or page reclamation.
