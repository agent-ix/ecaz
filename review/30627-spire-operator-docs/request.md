# SPIRE Operator Docs

## Scope

This packet documents the validated Phase 8 operator path for SPIRE maintenance
and cleanup diagnostics.

Code checkpoint: `2cf012d0` (`Add SPIRE local correctness matrix packet`)

## Changes

- Updates `docs/SPIRE_DIAGNOSTICS.md` to list:
  - `ec_spire_index_epoch_cleanup_summary(index_oid)`
  - `ec_spire_index_maintenance_plan_snapshot(index_oid)`
  - `ec_spire_index_locked_maintenance_run_plan(index_oid)`
  - `ec_spire_index_maintenance_scheduler_plan(index_oid)`
  - `ec_spire_index_maintenance_scheduler_run(index_oid)`
- Adds a maintenance-and-cleanup section describing the operator-periodic job
  flow:
  1. read scheduler plan
  2. run scheduler when `scheduler_status = 'due'`
  3. inspect the maintenance result
- Documents that `blocked_not_implemented` is the current old-epoch physical
  cleanup status.
- Marks the Phase 8 docs item complete.

## Files

- `docs/SPIRE_DIAGNOSTICS.md`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `git diff --check`

## Notes

No tests were run. This is a documentation-only checkpoint for SQL surfaces
covered by packet 30625.
