# Review Request: SPIRE Maintenance No-Candidate SQL Smoke

## Summary

Task 30 SPIRE Phase 2 now covers the live manual scheduler's populated
no-candidate branch.

Changes:
- Add `test_ec_spire_maintenance_run_no_candidate_sql`.
- Build a healthy populated two-leaf fixture.
- Run `ec_spire_index_maintenance_run(index_oid)`.
- Verify the result reports `no_action` / `no_candidate`, keeps active epoch 1
  before and after, returns `published = false`, and leaves the active leaf
  count unchanged.

## Validation

- `cargo pgrx test pg18 test_ec_spire_maintenance_run_no_candidate_sql`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims.
