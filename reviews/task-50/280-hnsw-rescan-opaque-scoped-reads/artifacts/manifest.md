# Artifact Manifest: Task 50 Packet 280

- head SHA: `4ef014df9a1dabb7b34d9dc2ae2783ea4cc3b3df`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/280-hnsw-rescan-opaque-scoped-reads`
- timestamp: `2026-05-21T10:53:43-07:00`
- lane: HNSW/debug unsafe burndown
- fixture: rescan query-dimension debug opaque reads
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

### `rescan-opaque-grep.log`

- command: `rg -n 'let result = debug_with_scan_opaque\(scan|let opaque = unsafe \{ debug_scan_opaque\(scan\)' src/am/ec_hnsw/scan_debug.rs`
- result: passed
- key lines: rescan query-dimension helpers now use `debug_with_scan_opaque`; remaining direct opaque reads are outside this slice

### `unsafe-count.log`

- command: `rg -n unsafe src | wc -l`
- result: passed
- key lines: `2164`

### `cargo-check-pg18-bench.log`

- command: `cargo check --all-targets --no-default-features --features pg18,bench`
- result: passed
- key lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 12.72s`
- warnings: existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`

### `cargo-test-lib-pg18-pgtest-no-run.log`

- command: `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- result: passed
- key lines: `Finished test profile [unoptimized + debuginfo] target(s) in 38.86s`
- warnings: existing Hadamard test-only dead-code warnings
