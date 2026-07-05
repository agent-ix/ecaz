# Review Request: SPIRE DROP INDEX Catalog Cleanup

## Scope

This packet reviews commit `0d96d6a6 Clean SPIRE remote catalogs on DROP INDEX`.

The slice closes the remote catalog lifecycle automation gap:

- adds `ec_spire_remote_catalog_drop_index_cleanup_event()` as a SQL `event_trigger` function.
- adds `ec_spire_remote_catalog_drop_index_cleanup` on `sql_drop`.
- the trigger filters dropped objects to `object_type = 'index'` and delegates each dropped index OID to
  `ec_spire_remote_catalog_index_cleanup(...)`.
- updates the lifecycle contract from future/manual cleanup to automatic event-trigger cleanup.
- updates the Phase 7 task note.

## Validation

Focused PG18 coverage:

```text
cargo pgrx test pg18 test_ec_spire_remote_catalog_drop_index_event_cleanup
cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts
git diff --check
```

Both PG18 tests passed. The new test creates a real `ec_spire` index, inserts remote descriptor and
manifest rows keyed to its OID, drops the index, and verifies the event trigger removed all matching
remote catalog rows.

## Review Notes

- The exact cleanup helper remains available for explicit/operator use.
- Broad orphan cleanup remains separate and is still the right tool after restore-era OID churn.
