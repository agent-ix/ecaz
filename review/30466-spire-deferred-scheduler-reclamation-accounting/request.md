# Review Request: SPIRE Deferred Scheduler/Reclamation Accounting

## Summary

Task 30 SPIRE planning now explicitly accounts for two reviewer-called-out
items as later-phase work rather than Phase 2 blockers.

Changes:
- Add a Phase 8 checklist item for an automatic background/VACUUM/periodic
  maintenance scheduler around the existing manual
  `ec_spire_index_maintenance_run(index_oid)` path.
- Add a Phase 8 checklist item for old-epoch physical reclamation after
  active-query and retention rules prove old object/manifest tuples are safe to
  reclaim.

## Validation

- `git diff --check`

## Notes

Documentation/status-only checkpoint. No measurement claims.
