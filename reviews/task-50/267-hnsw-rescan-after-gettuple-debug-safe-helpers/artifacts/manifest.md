# Artifact Manifest: Task 50 Packet 267

- head SHA: `d665b61169f250428665dadc308379ff6bca6d9e`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/267-hnsw-rescan-after-gettuple-debug-safe-helpers`
- timestamp: `2026-05-21T09:39:53-07:00`
- lane: HNSW unsafe burndown
- fixture: test/debug HNSW gettuple rescan helpers
- storage format: N/A
- rerank mode: N/A
- isolated one-index-per-table or shared-table surface: N/A; no benchmark run

## Artifacts

### `git-diff-check.log`

- command: `git diff --check`
- result: passed
- key lines: empty log

### `rustfmt-check.log`

- command: `rustfmt --edition 2021 --check src/am/ec_hnsw/scan_debug.rs`
- result: passed
- key lines: existing stable-channel warnings for nightly-only `imports_granularity` and `group_imports`

### `hnsw-rescan-after-wrapper-grep.log`

- command: `rg -n "hnsw_scan_debug!\(am::debug_gettuple_(rescan_after_exhaustion|backward_after_rescan|rescan_after_partial)" src/tests/ec_hnsw_scan_gettuple.rs`
- result: no matching wrappers remain
- key lines: `no hnsw_scan_debug wrappers remain for rescan_after_exhaustion/backward_after_rescan/rescan_after_partial`

### `unsafe-count.log`

- command: `rg -n "unsafe" src | wc -l`
- result: passed
- key lines: `2247`

### `cargo-check-pg18-bench.log`

- command: `cargo check --all-targets --no-default-features --features pg18,bench`
- result: passed
- key lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 12.84s`
- warnings: existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`

### `cargo-test-lib-pg18-pgtest-no-run.log`

- command: `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- result: passed
- key lines: `Finished test profile [unoptimized + debuginfo] target(s) in 35.68s`
- warnings: existing Hadamard test-only dead-code warnings

