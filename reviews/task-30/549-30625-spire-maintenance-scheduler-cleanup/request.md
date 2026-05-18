# SPIRE Maintenance Scheduler And Cleanup Summary

## Scope

This packet adds Phase 8 operator surfaces for SPIRE maintenance scheduling and
old-epoch cleanup visibility.

Code checkpoint: `18a6fe4a` (`Add SPIRE maintenance scheduler SQL surfaces`)

## Changes

- Adds `ec_spire_index_maintenance_scheduler_plan(index_oid)`.
  - Reports an `operator_periodic_job` scheduler policy.
  - Uses the locked maintenance planner, preserving the lock-time reload/recheck
    contract.
  - Reports `due` only when the existing planner selects split/merge work.
- Adds `ec_spire_index_maintenance_scheduler_run(index_oid)`.
  - Delegates to the existing `ec_spire_index_maintenance_run(index_oid)`
    publish path.
  - Reports scheduler metadata alongside the maintenance run result.
- Adds `ec_spire_index_epoch_cleanup_summary(index_oid)`.
  - Combines epoch-retention state with cleanup-candidate tuple counts/bytes.
  - Explicitly reports `blocked_not_implemented` while physical tuple
    reclamation remains absent.
- Marks the Phase 8 background scheduler task complete and records cleanup
  summary progress while leaving physical reclamation open.

## Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_relation_storage_snapshot_sql`
- `cargo pgrx test pg18 test_ec_spire_maintenance`
- `git diff --check`

## Notes

This does not add a background worker. The scheduler surface is intentionally
operator-controlled so cron, pg_cron, or another deployment-specific periodic
job can invoke the same maintenance publish path without introducing a second
split/merge implementation.

Physical old-epoch tuple reclamation is still not implemented; the new cleanup
summary makes that status SQL-visible next to the retention and cleanup-debt
counts.
