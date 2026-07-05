# Artifact Manifest: SPIRE Dropped Index No Descriptor Refresh

- head SHA: `f1d3e3fe2ca1cfb84f995c30a848a8c8d4513842`
- packet/topic: `757-c1-spire-dropped-index-no-descriptor-refresh`
- lane: Phase 12c test coverage
- fixture: `test_ec_spire_prod_receive_drop_remote_index_before_dispatch`
- storage format: `rabitq`
- rerank mode: not applicable
- command surface: focused Rust/pgrx test validation
- timestamp: `2026-05-15T02:07:45Z`
- isolated one-index-per-table vs shared-table surface: test-owned loopback table with ready and dropped indexes

## Commands

- `cargo fmt --check`
- `git diff --check -- src/tests/remote_search/receive_faults.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_prod_receive_drop_remote_index_before_dispatch --no-run`
- `cargo pgrx test pg18 test_ec_spire_prod_receive_drop_remote_index_before_dispatch`

## Key Result Lines

- Focused compile-only test build completed successfully.
- Runtime pgrx attempt failed before test execution:
  `undefined symbol: pg_re_throw`.
