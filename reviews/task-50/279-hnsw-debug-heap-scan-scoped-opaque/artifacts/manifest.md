# Artifact Manifest: Task 50 Packet 279

- head SHA: `993c4fa3e826906e6c12507b7cc35eea4dc6f9c8`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/279-hnsw-debug-heap-scan-scoped-opaque`
- timestamp: `2026-05-21T10:50:04-07:00`
- lane: HNSW/debug unsafe burndown
- fixture: heap-backed scan debug opaque reads
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

### `heap-scan-opaque-unsafe-grep.log`

- command: `rg -n 'let opaque = unsafe \{ debug_scan_opaque\(scan\)' src/am/ec_hnsw/scan_debug.rs`
- result: passed
- key lines: heap-backed profiling helpers now use `scan_state.with_opaque`; remaining matches are unrelated debug helpers outside this slice

### `unsafe-count.log`

- command: `rg -n unsafe src | wc -l`
- result: passed
- key lines: `2167`

### `cargo-check-pg18-bench.log`

- command: `cargo check --all-targets --no-default-features --features pg18,bench`
- result: passed
- key lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.19s`
- warnings: existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`

### `cargo-test-lib-pg18-pgtest-no-run.log`

- command: `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- result: passed
- key lines: `Finished test profile [unoptimized + debuginfo] target(s) in 59.77s`
- warnings: existing Hadamard test-only dead-code warnings
