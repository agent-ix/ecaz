# Artifact Manifest: SPIRE Stale Manifest Endpoint Status

- head SHA: `e8024ac2b2249771a889f3ab3ec3ecc19e5a97f0`
- packet/topic: `753-c1-spire-stale-manifest-endpoint-status`
- lane: Phase 12c test coverage
- fixture: `test_ec_spire_remote_epoch_manifest_persist_ready`
- storage format: existing test fixture default
- rerank mode: not applicable
- command surface: focused Rust/pgrx test validation
- timestamp: `2026-05-15T01:42:02Z`
- isolated one-index-per-table vs shared-table surface: one test-owned table/index fixture

## Commands

- `cargo fmt --check`
- `git diff --check -- src/tests/remote_search/epoch_manifest.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_remote_epoch_manifest_persist_ready --no-run`
- `cargo pgrx test pg18 test_ec_spire_remote_epoch_manifest_persist_ready`

## Key Result Lines

- Focused compile-only test build completed successfully.
- Runtime pgrx attempt failed before test execution:
  `undefined symbol: pg_re_throw`.
