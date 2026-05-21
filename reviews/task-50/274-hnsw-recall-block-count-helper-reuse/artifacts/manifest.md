# Artifact Manifest: Task 50 Packet 274

- head SHA: `11d29bd2c5769f3453b8bdcad01c395c18d1ef18`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/274-hnsw-recall-block-count-helper-reuse`
- timestamp: `2026-05-21T10:24:55-07:00`
- lane: HNSW/test unsafe burndown
- fixture: graph scan recall probe block count
- storage format: HNSW
- rerank mode: N/A
- isolated one-index-per-table or shared-table surface: test-created isolated HNSW recall fixture index

## Artifacts

### `git-diff-check.log`

- command: `git diff --check`
- result: passed
- key lines: empty log

### `rustfmt-check.log`

- command: not run as standalone rustfmt on included test file
- result: skipped by policy for module-included test sources
- key lines: `standalone rustfmt skipped: changed file is a src/tests include file whose indentation is owned by the including module; formatting checked by cargo parser instead`

### `recall-helper-unsafe-grep.log`

- command: `rg -n unsafe src/tests/ec_hnsw_recall_helpers.rs`
- result: no matches; expected command exit code `1`
- key lines: no output

### `unsafe-count.log`

- command: `rg -n unsafe src | wc -l`
- result: passed
- key lines: `2204`

### `cargo-check-pg18-bench.log`

- command: `cargo check --all-targets --no-default-features --features pg18,bench`
- result: passed
- key lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 12.20s`
- warnings: existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`

### `cargo-test-lib-pg18-pgtest-no-run.log`

- command: `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- result: passed
- key lines: `Finished test profile [unoptimized + debuginfo] target(s) in 31.56s`
- warnings: existing Hadamard test-only dead-code warnings
