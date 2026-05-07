# Review Request: SPIRE Update Mechanics Scheduler Design Refresh

## Summary

Task 30 SPIRE Phase 2 update-mechanics design text now reflects the landed
manual scheduler.

Changes:
- Update `plan/design/spire-update-mechanics.md` from a planning checkpoint to
  an implementation checkpoint.
- Replace stale "scheduler not decided yet" wording with the landed
  `ec_spire_index_maintenance_run(index_oid)` manual entrypoint and its locked
  no-write preflight.
- Record that the manual entrypoint is `VOLATILE`, takes the shared publish
  lock, reloads/rechecks the active candidate, and publishes split or merge
  replacement epochs.
- Leave background scheduling and PID-preserving rebalance as future work.

## Validation

- `git diff --check`

## Notes

Documentation/status-only checkpoint. No measurement claims.
