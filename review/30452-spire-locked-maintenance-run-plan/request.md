# Review Request: SPIRE Locked Maintenance Run Plan

## Summary

Task 30 SPIRE Phase 2 now exposes a locked, no-write maintenance run-plan SQL
surface using the same result row shape intended for the future mutating manual
scheduler entrypoint.

Changes:
- Add `maintenance_run_result_from_rows`, which selects the current scheduled
  split/merge candidate from leaf snapshot rows and returns a projected
  run-result row without publishing.
- Add `index_locked_maintenance_run_plan`, which holds the SPIRE publish lock
  while loading root/control, active manifests, object store, leaf rows, and
  projected scheduler state.
- Expose `ec_spire_index_locked_maintenance_run_plan(index_oid)` through the
  SQL extension surface.
- Cover selected-split and no-candidate run-plan shaping with pure unit tests.

## Validation

- `cargo test maintenance_run_plan --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No relation objects are written and no root/control state is advanced in this
slice. Planned rows report `published = false`.
No measurement claims.
