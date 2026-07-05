# Review Request: SPIRE Test Aggregator Split

## Summary

Coder: `coder1`
Topic: `760-c1-spire-test-aggregator-split`
Code commit: `9a6de38c9800f04bb69e00ebb70e50437029e413`
Date: `2026-05-15`

This checkpoint addresses the Phase 12c file-size discipline called out by the
user. The audit found `src/tests/mod.rs` had drifted back over the 2500-line
target after the coverage work. This slice mechanically extracts existing
self-contained blocks from the aggregator into small included files:

- `src/tests/type_registration.rs`
- `src/tests/psql_helpers.rs`
- `src/tests/hnsw_misc.rs`

No test behavior is intended to change; the extracted code is still included
inside the existing `#[pg_schema] mod tests` scope.

## Files

- `src/tests/mod.rs`
- `src/tests/type_registration.rs`
- `src/tests/psql_helpers.rs`
- `src/tests/hnsw_misc.rs`

## Validation

- `wc -l src/tests/mod.rs src/tests/type_registration.rs src/tests/psql_helpers.rs src/tests/hnsw_misc.rs`
  reports `src/tests/mod.rs` at 2492 lines, with the new files at 119, 166,
  and 48 lines.
- `git diff --check -- src/tests/mod.rs src/tests/type_registration.rs src/tests/psql_helpers.rs src/tests/hnsw_misc.rs`
  passed.
- `cargo fmt --check` passed with the repo's existing stable-rustfmt warnings
  about ignored unstable import settings.
- `cargo test --features "pg18 pg_test" --no-default-features test_binary_send_matches_internal_layout --no-run`
  passed.

## Review Needs

Please verify that this is a pure include-boundary split and that it satisfies
the <2500-line target for `src/tests/mod.rs` without changing test semantics.
