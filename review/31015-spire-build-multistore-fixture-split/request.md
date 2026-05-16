# Review Request: SPIRE Multistore Build Fixture Split

## Summary

Code commit: `f39a2af68aad30ad76e2d6062b9fb60c8a8b1a75`

This checkpoint extends the Phase 12b.2 `tests/build.rs` concern file by moving the auxiliary-store relcache, multistore, reindex, tqvector populated-build, and two-store scan fixture block from `src/tests/mod.rs` into `src/tests/build.rs`.

The move keeps the textual include pattern, so fixture names and `pg_schema` scope remain unchanged.

## Scope

- Moved:
  - `test_ec_spire_aux_store_relcache_disables_autovacuum`
  - `test_ec_spire_multistore_large_fixture_routes_all_stores`
  - `test_ec_spire_singlestore_reindex_succeeds`
  - `test_ec_spire_multistore_reindex_rejected`
  - `test_ec_spire_tqvector_populated_build_scans_with_heap_rerank`
  - `test_ec_spire_relation_two_store_scan_roundtrip`
- Updated `plan/tasks/task30-phase12b-spire-cleanup.md` to record packet `31015`.

## Validation

- `cargo fmt --check`
- `cargo test --no-default-features --features pg18 test_ec_spire_multistore_large_fixture_routes_all_stores -- --nocapture`
- `cargo test --no-default-features --features pg18 test_ec_spire_multistore_reindex_rejected -- --nocapture`
- `cargo test --no-default-features --features pg18 test_ec_spire_relation_two_store_scan_roundtrip -- --nocapture`
- Packet-local location and line-count checks under `artifacts/`.
- `git diff --check`

All focused PG18 tests passed. The multistore reindex fixture passed through its expected `should_panic` path. The tests emitted the pre-existing unused-import warning in `src/am/mod.rs`.

## Remaining 12b.2 Work

`tests/build.rs` remains open because recursive-fanout and top-graph build fixtures still live outside the concern file.
