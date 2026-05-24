# Artifact Manifest: Task 50 Packet 275

- head SHA: `684e45847add5185960c23774614b39a020c64f6`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/275-hnsw-storage-lifecycle-debug-page-decode`
- timestamp: `2026-05-21T10:28:53-07:00`
- lane: HNSW/test unsafe burndown
- fixture: PqFastScan live insert storage lifecycle
- storage format: HNSW PqFastScan
- rerank mode: quantized rerank tuple decode from debug page snapshot
- isolated one-index-per-table or shared-table surface: test-created isolated HNSW index

## Artifacts

### `git-diff-check.log`

- command: `git diff --check`
- result: passed
- key lines: empty log

### `rustfmt-check.log`

- command: not run as standalone rustfmt on included test file
- result: skipped by policy for module-included test sources
- key lines: `standalone rustfmt skipped: changed file is a src/tests include file whose indentation is owned by the including module; formatting checked by cargo parser instead`

### `storage-lifecycle-unsafe-grep.log`

- command: `rg -n unsafe src/tests/ec_hnsw_storage_lifecycle.rs`
- result: no matches; expected command exit code `1`
- key lines: no output

### `unsafe-count.log`

- command: `rg -n unsafe src | wc -l`
- result: passed
- key lines: `2202`

### `cargo-check-pg18-bench.log`

- command: `cargo check --all-targets --no-default-features --features pg18,bench`
- result: passed
- key lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 12.62s`
- warnings: existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`

### `cargo-test-lib-pg18-pgtest-no-run.log`

- command: `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- result: passed
- key lines: `Finished test profile [unoptimized + debuginfo] target(s) in 50.00s`
- warnings: existing Hadamard test-only dead-code warnings
