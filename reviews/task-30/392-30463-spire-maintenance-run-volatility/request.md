# Review Request: SPIRE Maintenance Run Volatility

## Summary

Task 30 SPIRE Phase 2 now marks the mutating manual maintenance entrypoint as a
volatile SQL function.

Changes:
- Change `ec_spire_index_maintenance_run(index_oid)` from `STABLE` to
  `VOLATILE`.
- Add a PG18 catalog assertion in the no-candidate smoke to prove
  `pg_proc.provolatile = 'v'`.

## Validation

- `cargo pgrx test pg18 test_ec_spire_maintenance_run_no_candidate_sql`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims.
