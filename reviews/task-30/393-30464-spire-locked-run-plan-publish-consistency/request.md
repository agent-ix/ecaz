# Review Request: SPIRE Locked Run-Plan Publish Consistency

## Summary

Task 30 SPIRE Phase 2 now verifies that the locked no-write run plan and the
live maintenance publish agree when the active snapshot does not change between
calls.

Changes:
- Extend `test_ec_spire_locked_maintenance_run_plan_no_write_sql`.
- Keep the existing assertion that the locked run-plan call does not advance
  active epoch, allocator cursor, or leaf count.
- Invoke `ec_spire_index_maintenance_run(index_oid)` immediately afterward.
- Verify the live publish reuses the projected action, affected PIDs,
  replacement PIDs, publish epoch, and next PID cursor from the locked plan.

## Validation

- `cargo pgrx test pg18 test_ec_spire_locked_maintenance_run_plan_no_write_sql`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims.
