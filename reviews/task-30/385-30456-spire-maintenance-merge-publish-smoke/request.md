# Review Request: SPIRE Maintenance Merge Publish Smoke

## Summary

Task 30 SPIRE Phase 2 now has focused PG18 runtime coverage for a populated
manual maintenance merge publish.

Changes:
- Preserve empty affected leaf row groups when collecting scheduled replacement
  leaf rows.
- Add pure coverage for collecting an empty affected leaf.
- Add `test_ec_spire_maintenance_run_merge_publish_sql`, which builds a
  one-row, three-leaf fixture, runs `ec_spire_index_maintenance_run`, and
  verifies a merge publish to epoch 2 with two active leaves after publication.

## Validation

- `cargo test selected_scheduled_replacement_leaf_rows_keeps_empty_affected_leaf --lib`
- `cargo pgrx test pg18 test_ec_spire_maintenance_run_merge_publish_sql`
- `cargo fmt --check`
- `git diff --check`

## Notes

The first PG18 attempt found the empty affected-leaf gap; this packet documents
the fixed and passing run.
No measurement claims.
