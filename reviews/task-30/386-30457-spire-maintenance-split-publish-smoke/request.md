# Review Request: SPIRE Maintenance Split Publish Smoke

## Summary

Task 30 SPIRE Phase 2 now has focused PG18 runtime coverage for a populated
manual maintenance split publish.

Changes:
- Add `test_ec_spire_maintenance_run_split_publish_sql`.
- Build a skewed 60-row, ten-leaf fixture that produces one split candidate.
- Run `ec_spire_index_maintenance_run(index_oid)`.
- Verify the run reports `published` / `split`, advances to epoch 2, and expands
  the active leaf count from 10 to 11.

## Validation

- `cargo pgrx test pg18 test_ec_spire_maintenance_run_split_publish_sql`
- `cargo fmt --check`
- `git diff --check`

## Notes

This covers the heap-source split path of the manual scheduler entrypoint.
No measurement claims.
