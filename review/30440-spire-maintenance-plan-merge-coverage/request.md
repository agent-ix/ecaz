# Review Request: SPIRE Maintenance Plan Merge Coverage

## Summary

Task 30 SPIRE Phase 2 maintenance-plan snapshot coverage now includes the
merge branch.

Changes:
- Add a focused unit test proving the read-only maintenance plan snapshot
  reports `merge`, the sparsest same-parent pair, one replacement PID, the
  successor publish epoch, and the advanced PID cursor.

## Validation

- `cargo test maintenance_plan_snapshot --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This is test coverage for the
read-only planning surface added in `30439`.
