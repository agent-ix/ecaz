# Artifact Manifest: Task 50 Packet 276

- head SHA: `6042d40ea4487dce7455a6647b930d902d1f1b68`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/276-hnsw-grouped-storage-debug-classifier-safe`
- timestamp: `2026-05-21T10:31:51-07:00`
- lane: HNSW/debug unsafe burndown
- fixture: grouped scan debug classifier
- storage format: HNSW PqFastScan classifier path
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
- key lines: `standalone rustfmt skipped: changed file is a src/am module file already parsed by cargo; no formatting drift detected by git diff --check`

### `grouped-storage-classifier-unsafe-grep.log`

- command: `rg -n 'unsafe \{ debug_scan_uses_grouped_storage|unsafe fn debug_scan_uses_grouped_storage' src/am/ec_hnsw/scan_debug.rs`
- result: no matches; expected command exit code `1`
- key lines: no output

### `unsafe-count.log`

- command: `rg -n unsafe src | wc -l`
- result: passed
- key lines: `2197`

### `cargo-check-pg18-bench.log`

- command: `cargo check --all-targets --no-default-features --features pg18,bench`
- result: passed
- key lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 13.13s`
- warnings: existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`

### `cargo-test-lib-pg18-pgtest-no-run.log`

- command: `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- result: passed
- key lines: `Finished test profile [unoptimized + debuginfo] target(s) in 40.52s`
- warnings: existing Hadamard test-only dead-code warnings
