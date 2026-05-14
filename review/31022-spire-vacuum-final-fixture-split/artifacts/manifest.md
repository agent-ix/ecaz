# Artifact Manifest: 31022 SPIRE Vacuum Final Fixture Split

Head SHA: `1feba980f6f7c3234179bf5cb94d044ac221aa4a`

Packet/topic: `31022-spire-vacuum-final-fixture-split`

Timestamp: `2026-05-13T18:33:26-07:00`

Lane / fixture / storage format / rerank mode: Phase 12b cleanup fixture
relocation; SPIRE vacuum fixtures; PostgreSQL/pgrx test storage; rerank mode
not applicable.

Isolation surface: local PG18 pgrx test database; not a measurement run; not
an isolated one-index-per-table or shared-table benchmark surface.

## Artifacts

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Result: pass
- Key lines: rustfmt emitted the repository's stable-channel warnings for
  unstable `imports_granularity` and `group_imports`; command exited 0.

### `git-diff-check.log`

- Command: `git diff --check`
- Result: pass
- Key lines: no output; command exited 0.

### `location-check.log`

- Command: `rg -n 'test_ec_spire_vacuum_delete_delta_suppresses_visible_row|test_ec_spire_vacuum_cleanup_compacts_insert_delta|test_pg18_ec_spire_sql_vacuum_mixed_delta|test_pg18_ec_spire_multistore_sql_vacuum_routes_local_stores|test_pg18_ec_spire_concurrent_insert_vacuum_scan|test_ec_spire_relation_object_tuple_roundtrip' src/tests/mod.rs src/tests/vacuum.rs`
- Result: pass
- Key lines:
  - `src/tests/vacuum.rs:923:    fn test_ec_spire_vacuum_delete_delta_suppresses_visible_row()`
  - `src/tests/vacuum.rs:1014:    fn test_ec_spire_vacuum_cleanup_compacts_insert_delta()`
  - `src/tests/vacuum.rs:1142:    fn test_pg18_ec_spire_sql_vacuum_mixed_delta()`
  - `src/tests/vacuum.rs:1228:    fn test_pg18_ec_spire_multistore_sql_vacuum_routes_local_stores()`
  - `src/tests/vacuum.rs:1344:    fn test_pg18_ec_spire_concurrent_insert_vacuum_scan()`
  - `src/tests/mod.rs:12539:    fn test_ec_spire_relation_object_tuple_roundtrip()`

### `line-counts.log`

- Command: `wc -l src/tests/mod.rs src/tests/vacuum.rs src/tests/insert.rs src/am/ec_spire/custom_scan/explain.rs`
- Result: informational
- Key lines:
  - `34592 src/tests/mod.rs`
  - `1477 src/tests/vacuum.rs`
  - `2814 src/tests/insert.rs`
  - `80 src/am/ec_spire/custom_scan/explain.rs`

### `pg18-test-vacuum-cleanup-compacts-insert-delta.log`

- Command: `cargo pgrx test pg18 test_ec_spire_vacuum_cleanup_compacts_insert_delta`
- Result: pass
- Key line: `test tests::pg_test_ec_spire_vacuum_cleanup_compacts_insert_delta ... ok`

### `pg18-test-sql-vacuum-mixed-delta.log`

- Command: `cargo pgrx test pg18 test_pg18_ec_spire_sql_vacuum_mixed_delta`
- Result: pass
- Key line: `test tests::pg_test_pg18_ec_spire_sql_vacuum_mixed_delta ... ok`

### `pg18-test-multistore-sql-vacuum-routes-local-stores.log`

- Command: `cargo pgrx test pg18 test_pg18_ec_spire_multistore_sql_vacuum_routes_local_stores`
- Result: pass
- Key line: `test tests::pg_test_pg18_ec_spire_multistore_sql_vacuum_routes_local_stores ... ok`

### `pg18-test-concurrent-insert-vacuum-scan.log`

- Command: `cargo pgrx test pg18 test_pg18_ec_spire_concurrent_insert_vacuum_scan`
- Result: fail
- Key lines:
  - `test tests::pg_test_pg18_ec_spire_concurrent_insert_vacuum_scan ... FAILED`
  - `ERROR:  ec_spire remote search target plan requested epoch 3 does not match active epoch 4`
  - `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 32.94s`

### `pg18-test-concurrent-insert-vacuum-scan-rerun.log`

- Command: `cargo pgrx test pg18 test_pg18_ec_spire_concurrent_insert_vacuum_scan`
- Result: fail
- Key lines:
  - `test tests::pg_test_pg18_ec_spire_concurrent_insert_vacuum_scan ... FAILED`
  - `ERROR:  ec_spire remote search target plan requested epoch 3 does not match active epoch 4`
  - `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 36.39s`
