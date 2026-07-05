# Task 50 Packet 261 Artifact Manifest

- head SHA at validation start: `805c9d731cf6ff6b2917ec383a81f997e76d23ca`
- task bucket: `reviews/task-50/261-hnsw-grouped-scan-debug-safe-helpers`
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
  - previous packet 260 baseline: `2272`
  - after this slice: `2264`

### `hnsw-grouped-wrapper-grep.log`

- command: `rg -n 'hnsw_[a-z_]+_debug!\(am::debug_grouped_scan_(comparison_rows|comparison_summary|order_drift_summary|windowed_rows|windowed_summary)' src/tests/ec_hnsw_*.rs`
- timestamp: 2026-05-21
- result: no matches; `rg` exited `1` after finding no remaining macro wrappers
  around these newly-safe grouped scan helpers

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
