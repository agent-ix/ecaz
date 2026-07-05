# Artifact Manifest: Task 50 Packet 281

- head SHA: `9c0b214cb477e9cf4047703a9e0c204fd57eb696`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/281-hnsw-current-result-opaque-scoped-reads`
- timestamp: `2026-05-21T10:57:30-07:00`
- lane: HNSW/debug unsafe burndown
- fixture: current-result and candidate-state opaque debug reads
- storage format: HNSW
- rerank mode: N/A
- isolated one-index-per-table or shared-table surface: N/A; no benchmark run

## Artifacts

### `git-diff-check.log`

- command: `git diff --check`
- result: passed
- key lines: empty log

### `rustfmt-check.log`

- command: not run separately
- result: skipped after `git diff --check` and cargo parser validation
- key lines: `standalone rustfmt skipped: changed src/am module was parsed by cargo; formatting checked by git diff --check`

### `current-result-opaque-grep.log`

- command: `rg -n 'debug_current_result_(comparison_score|approx_score|approx_rank)|debug_gettuple_current_result_state|debug_rescan_(entry|successor)_candidate_state|let opaque = unsafe \{ debug_scan_opaque\(scan\)' src/am/ec_hnsw/scan_debug.rs`
- result: passed
- key lines: touched helpers now use `debug_with_scan_opaque`; remaining direct opaque reads are outside this slice

### `unsafe-count.log`

- command: `rg -n unsafe src | wc -l`
- result: passed
- key lines: `2156`

### `cargo-check-pg18-bench.log`

- command: `cargo check --all-targets --no-default-features --features pg18,bench`
- result: passed
- key lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 12.60s`
- warnings: existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`

### `cargo-test-lib-pg18-pgtest-no-run.log`

- command: `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- result: passed
- key lines: `Finished test profile [unoptimized + debuginfo] target(s) in 40.02s`
- warnings: existing Hadamard test-only dead-code warnings
