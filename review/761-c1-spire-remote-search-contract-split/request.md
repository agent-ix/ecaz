# Review Request: SPIRE Remote Search Contract Split

## Summary

Coder: `coder1`
Topic: `761-c1-spire-remote-search-contract-split`
Code commit: `fd7817910f77636630fe68feb1dc2f767b189bd7`
Date: `2026-05-15`

This checkpoint finishes the Phase 12c SPIRE test-file size cleanup after
packet `760`. The audit still found `src/tests/remote_search/contracts.rs` at
2864 lines, so this slice mechanically moves the libpq/request/receive/final
contract tests into `contracts_libpq.rs` and leaves an `include!` in the
original file.

No test behavior is intended to change; the moved tests remain in the same
`#[pg_schema] mod tests` scope via the existing `remote_search/mod.rs` include
chain.

## Files

- `src/tests/remote_search/contracts.rs`
- `src/tests/remote_search/contracts_libpq.rs`

## Validation

- `wc -l src/tests/remote_search/contracts.rs src/tests/remote_search/contracts_libpq.rs`
  reports 1657 and 1208 lines respectively.
- `git diff --check -- src/tests/remote_search/contracts.rs src/tests/remote_search/contracts_libpq.rs`
  passed.
- `cargo fmt --check` passed with the repo's existing stable-rustfmt warnings
  about ignored unstable import settings.
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_remote_search_libpq_req_blocked --no-run`
  passed.

## Review Needs

Please verify this is an exact include-boundary split and that the remaining
SPIRE-side test files touched by Phase 12c are under the 2500-line target.
