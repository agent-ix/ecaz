# Task 50 Packet 266 Artifact Manifest

- head SHA at validation start: `51fbe05034c62e87ef05d2eccb78b1fc10466719`
- task bucket: `reviews/task-50/266-hnsw-frontier-top-level-debug-safe-helpers`
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
  - previous packet 265 baseline: `2251`
  - after this slice: `2249`

### `hnsw-frontier-top-level-wrapper-grep.log`

- command: `rg -n 'hnsw_(scan|recall|graph)_debug!\(am::debug_(rescan_entry_candidate_state|rescan_successor_candidate_state|rescan_candidate_frontier|all_top_level_heap_tids|top_level_reachable_heap_tids)' src/tests/ec_hnsw_*.rs`
- timestamp: 2026-05-21
- result: no matches; `rg` exited `1` after finding no remaining macro wrappers
  around these newly-safe helpers

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
