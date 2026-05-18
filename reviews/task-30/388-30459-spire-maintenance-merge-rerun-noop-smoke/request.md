# Review Request: SPIRE Maintenance Merge Rerun No-Op Smoke

## Summary

Task 30 SPIRE Phase 2 now verifies that a manual maintenance merge publish does
not keep publishing once the selected merge work is exhausted.

Changes:
- Extend `test_ec_spire_maintenance_run_merge_publish_sql`.
- After the first `ec_spire_index_maintenance_run(index_oid)` publishes a merge,
  call the same SQL entrypoint again.
- Verify the second row reports `no_action` / `no_candidate`, keeps active epoch
  2 before and after, returns `published = false`, and leaves the active leaf
  count at 2.

## Validation

- `cargo pgrx test pg18 test_ec_spire_maintenance_run_merge_publish_sql`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`

## Notes

No measurement claims.
