# Artifact Manifest: 30798 SPIRE Row Materialization Catalog Provider

- head SHA: `c2dea420ae71b586f0715b818df086ce6da1fd08`
- packet/topic: `30798-spire-row-materialization-catalog-provider`
- timestamp: `2026-05-10T19:44:20-07:00`
- lane: Task 30 Phase 11.5 Stage D catalog-backed remote row materialization
- fixture: Rust unit tests plus focused PG18 pgrx tests
- storage format: `ec_spire_remote_row_materialization` catalog rows plus synthetic production scan output rows
- rerank mode: not applicable
- isolated/shared surface: isolated test tables/indexes; catalog cleanup tests exercise shared extension catalog tables

## Artifacts

### `row-materialization-unit.log`

- command: `script -q -c "cargo test production_scan_row_materialization --no-default-features --features pg18" review/30798-spire-row-materialization-catalog-provider/artifacts/row-materialization-unit.log`
- key result lines:
  - `running 4 tests`
  - `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1596 filtered out`

### `am-output-unit.log`

- command: `script -q -c "cargo test production_scan_result_stream_am_outputs --no-default-features --features pg18" review/30798-spire-row-materialization-catalog-provider/artifacts/am-output-unit.log`
- key result lines:
  - `running 3 tests`
  - `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1597 filtered out`

### `pg18-row-materialization-catalog-register.log`

- command: `script -q -c "cargo pgrx test pg18 test_ec_spire_remote_row_materialization_catalog_register" review/30798-spire-row-materialization-catalog-provider/artifacts/pg18-row-materialization-catalog-register.log`
- key result lines:
  - `test tests::pg_test_ec_spire_remote_row_materialization_catalog_register ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1599 filtered out`

### `pg18-remote-catalog-cleanup.log`

- command: `script -q -c "cargo pgrx test pg18 test_ec_spire_remote_catalog" review/30798-spire-row-materialization-catalog-provider/artifacts/pg18-remote-catalog-cleanup.log`
- key result lines:
  - `test tests::pg_test_ec_spire_remote_catalog_index_cleanup ... ok`
  - `test tests::pg_test_ec_spire_remote_catalog_orphan_cleanup ... ok`
  - `test tests::pg_test_ec_spire_remote_catalog_drop_index_event_cleanup ... ok`
  - `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1597 filtered out`

## Notes

- ADR-065 records the catalog storage decision requested by reviewer feedback on packet 30797 before this provider landed.
- The AM path batch-loads candidate mappings from the catalog and validates heap visibility under the scan snapshot. It still performs no scan-time writes.
