# Task 50 Packet 263 Artifact Manifest

- head SHA at validation start: `42b1e3bdb48c60c5a71eaf27c053a320180c1e9a`
- task bucket: `reviews/task-50/263-hnsw-scan-lifecycle-debug-safe-helpers`
- lane: unsafe burndown, HNSW debug/test API surface
- fixture: static validation and no-run compile coverage
- storage format: not applicable
- rerank mode: not applicable
- isolated one-index-per-table vs shared-table surface: not applicable

## Artifacts

### `unsafe-count.log`

- command: `rg -n unsafe src | wc -l`
- timestamp: 2026-05-21
- key result lines:
  - previous packet 262 baseline: `2261`
  - after this slice: `2256`

### `hnsw-lifecycle-wrapper-grep.log`

- command: `rg -n 'hnsw_scan_debug!\(am::debug_(begin_end_scan|end_scan_twice|rescan_query_dimensions|rescan_overwrites_query_dimensions|rescan_null_query|rescan_with_index_qual|rescan_with_unused_key_buffer|rescan_with_multiple_orderbys|gettuple_without_rescan|gettuple_after_rescan|gettuple_after_rescan_result)' src/tests/ec_hnsw_scan_gettuple.rs`
- timestamp: 2026-05-21
- result: no matches; `rg` exited `1` after finding no remaining macro wrappers
  around these newly-safe scan lifecycle helpers

### `rustfmt-check.log`

- command: `rustfmt --edition 2021 --check src/am/ec_hnsw/scan_debug.rs`
- timestamp: 2026-05-21
- result: passed; emitted the repository's stable-toolchain warnings for
  nightly-only rustfmt settings

### `git-diff-check.log`

- command: `git diff --check`
- timestamp: 2026-05-21
- result: passed

### `cargo-check-pg18-bench.log`

- command: `cargo check --all-targets --no-default-features --features pg18,bench`
- timestamp: 2026-05-21
- result: passed
- known warning: existing unused SPIRE DML test re-exports in `src/am/mod.rs`

### `cargo-test-lib-pg18-pgtest-no-run.log`

- command: `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- timestamp: 2026-05-21
- result: passed
- known warning: existing Hadamard test-only helper dead-code warnings
