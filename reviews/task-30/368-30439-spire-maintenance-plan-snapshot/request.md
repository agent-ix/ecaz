# Review Request: SPIRE Maintenance Plan Snapshot

## Summary

Task 30 SPIRE Phase 2 now has a read-only SQL maintenance planning surface for
the current scheduled split/merge candidate.

Changes:
- Add `ec_spire_index_maintenance_plan_snapshot(index_oid)`.
- Share the active leaf snapshot row collector so the planner uses one loaded
  active epoch snapshot.
- Report selected action, reason, affected leaf PIDs, replacement PIDs,
  successor publish epoch, and allocator cursors without writing relation
  objects.
- Add focused unit coverage for planned split and no-action outputs.
- Update the Phase 2 checklist.

## Validation

- `cargo test maintenance_plan_snapshot --lib`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims; no PG callback coverage. This is a read-only diagnostic
surface before live scheduler execution publishes replacement objects.
