# Artifact Manifest: SPIRE Test Aggregator Split

- head SHA: `9a6de38c9800f04bb69e00ebb70e50437029e413`
- packet/topic: `760-c1-spire-test-aggregator-split`
- lane: Phase 12c test coverage / file-size discipline
- fixture: not applicable; mechanical test include split
- storage format: not applicable
- rerank mode: not applicable
- command surface: Rust test compile / file-size audit
- timestamp: `2026-05-15T02:23:21Z`
- isolated one-index-per-table vs shared-table surface: not applicable

## Commands

- `wc -l src/tests/mod.rs src/tests/type_registration.rs src/tests/psql_helpers.rs src/tests/hnsw_misc.rs`
- `git diff --check -- src/tests/mod.rs src/tests/type_registration.rs src/tests/psql_helpers.rs src/tests/hnsw_misc.rs`
- `cargo fmt --check`
- `cargo test --features "pg18 pg_test" --no-default-features test_binary_send_matches_internal_layout --no-run`
- `git ls-remote origin refs/heads/task-30-spire`

## Key Result Lines

- `2492 src/tests/mod.rs`
- `119 src/tests/type_registration.rs`
- `166 src/tests/psql_helpers.rs`
- `48 src/tests/hnsw_misc.rs`
- `Finished test profile ... target(s) in 2m 35s`
- Remote branch `task-30-spire` points at
  `9a6de38c9800f04bb69e00ebb70e50437029e413`.
