# Review Request: SPIRE Maintenance Run Result Shape

## Summary

Task 30 SPIRE Phase 2 now has the shared result row shape that the live
maintenance scheduler SQL entrypoints can return after no-op, planned, or
published scheduler runs.

Changes:
- Add `SpireIndexMaintenanceRunResult` with active epoch before/after,
  maintenance status, selected action/reason, affected and replacement PIDs,
  publish epoch, allocator cursors, and a `published` flag.
- Add no-op and selected-plan row helpers so future SQL entrypoints can report
  projected scheduler choices separately from committed publish results.
- Cover no-op, projected selected-plan, and published selected-plan result
  shaping with pure unit tests.

## Validation

- `cargo test maintenance_run_result --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No PostgreSQL callback behavior changed in this slice. This is result-shape
groundwork for the live manual scheduler entrypoint.
No measurement claims.
