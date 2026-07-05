# SPIRE Remote Catalog Orphan Cleanup

## Scope

Task 30 SPIRE Phase 7 now has operator SQL surfaces for remote catalog
lifecycle cleanup.

Code checkpoint: `b06aab0b` (`Add SPIRE remote catalog orphan cleanup`)

## Changes

- Added `ec_spire_remote_catalog_orphan_summary()`.
- Added `ec_spire_remote_catalog_orphan_cleanup()`.
- Rows are considered live only when `coordinator_index_oid` resolves to a live
  `ec_spire` index in `pg_class`/`pg_am`.
- Cleanup removes orphan manifest headers, relies on FK cascade for manifest
  entries, and removes orphan descriptors.
- Added PG18 coverage using a synthetic dead OID to verify summary counts,
  cleanup counts, and post-cleanup ready status.
- Updated the Phase 7 task note.

## Validation

- `cargo fmt`
- `cargo pgrx test pg18 test_ec_spire_remote_catalog_orphan_cleanup`
- `git diff --check`

During validation, the first test shape accidentally invoked the destructive
cleanup function once per returned column. The final test fetches all cleanup
counts in a single SQL call.

## Review Focus

- Whether live-row detection should require a live `ec_spire` index, as it does
  here, rather than any `pg_class` row with the same OID.
- Whether manual cleanup is enough for Phase 7, with automatic `DROP INDEX`
  cleanup left as future lifecycle automation.
