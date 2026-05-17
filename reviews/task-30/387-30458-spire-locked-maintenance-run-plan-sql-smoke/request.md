# Review Request: SPIRE Locked Maintenance Run-Plan SQL Smoke

## Summary

Task 30 SPIRE Phase 2 now has focused PG18 runtime coverage for the locked,
no-write maintenance run-plan SQL surface.

Changes:
- Add `test_ec_spire_locked_maintenance_run_plan_no_write_sql`.
- Build a populated three-leaf fixture with a merge candidate.
- Run `ec_spire_index_locked_maintenance_run_plan(index_oid)`.
- Verify the row reports a planned merge with `published = false`.
- Re-read active diagnostics and leaf snapshot state to prove active epoch,
  allocator cursor, and leaf count were not advanced by the locked plan call.

## Validation

- `cargo pgrx test pg18 test_ec_spire_locked_maintenance_run_plan_no_write_sql`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`

## Notes

This closes the SQL-level smoke gap for projected run-plan rows. No measurement
claims.
