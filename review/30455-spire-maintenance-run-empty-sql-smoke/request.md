# Review Request: SPIRE Maintenance Run Empty SQL Smoke

## Summary

Task 30 SPIRE Phase 2 now has focused PG18 smoke coverage for the new manual
maintenance scheduler SQL entrypoint on an empty index.

Changes:
- Add `test_ec_spire_maintenance_run_empty_sql`.
- Create an empty `ec_spire` index and call
  `ec_spire_index_maintenance_run(index_oid)`.
- Verify the no-action row shape: `maintenance_status = 'no_action'`,
  `planned_action = 'none'`, `planned_reason = 'empty_index'`,
  `published = false`, and `active_epoch_after = 0`.

## Validation

- `cargo pgrx test pg18 test_ec_spire_maintenance_run_empty_sql`
- `cargo fmt --check`
- `git diff --check`

## Notes

This smoke covers SQL binding and no-op behavior. A populated split/merge
publish runtime test remains follow-up work.
No measurement claims.
