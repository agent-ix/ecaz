# Artifact Manifest: SPIRE Remote Search Contract Split

- head SHA: `fd7817910f77636630fe68feb1dc2f767b189bd7`
- packet/topic: `761-c1-spire-remote-search-contract-split`
- lane: Phase 12c test coverage / file-size discipline
- fixture: not applicable; mechanical test include split
- storage format: not applicable
- rerank mode: not applicable
- command surface: Rust test compile / file-size audit
- timestamp: `2026-05-15T02:27:42Z`
- isolated one-index-per-table vs shared-table surface: not applicable

## Commands

- `wc -l src/tests/remote_search/contracts.rs src/tests/remote_search/contracts_libpq.rs`
- `git diff --check -- src/tests/remote_search/contracts.rs src/tests/remote_search/contracts_libpq.rs`
- `cargo fmt --check`
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_remote_search_libpq_req_blocked --no-run`
- `git ls-remote origin refs/heads/task-30-spire`

## Key Result Lines

- `1657 src/tests/remote_search/contracts.rs`
- `1208 src/tests/remote_search/contracts_libpq.rs`
- `Finished test profile ... target(s) in 2m 25s`
- Remote branch `task-30-spire` points at
  `fd7817910f77636630fe68feb1dc2f767b189bd7`.
