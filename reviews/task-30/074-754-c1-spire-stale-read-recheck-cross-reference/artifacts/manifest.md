# Artifact Manifest: SPIRE Stale Read Recheck Cross-Reference

- head SHA: `81dc757df4e0d5069956ada0f2fde8d9858f41d0`
- packet/topic: `754-c1-spire-stale-read-recheck-cross-reference`
- lane: Phase 12c test coverage
- fixture: `test_ec_spire_remote_pk_select_isolation_contract_sql`
- storage format: existing test fixture default
- rerank mode: not applicable
- command surface: focused Rust/pgrx test validation
- timestamp: `2026-05-15T01:48:04Z`
- isolated one-index-per-table vs shared-table surface: test-owned coordinator and remote tables/indexes

## Commands

- `cargo fmt --check`
- `git diff --check -- src/tests/remote_search/catalog_cleanup_policy.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_remote_pk_select_isolation_contract_sql --no-run`
- `cargo pgrx test pg18 test_ec_spire_remote_pk_select_isolation_contract_sql`

## Key Result Lines

- Focused compile-only test build completed successfully.
- Runtime pgrx attempt failed before test execution:
  `undefined symbol: pg_re_throw`.
