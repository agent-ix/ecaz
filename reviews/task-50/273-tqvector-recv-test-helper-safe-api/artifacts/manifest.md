# Artifact Manifest: Task 50 Packet 273

- head SHA: `90f074ed3dd330c96e581e9710bb00e91b8a83fa`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/273-tqvector-recv-test-helper-safe-api`
- timestamp: `2026-05-21T10:20:24-07:00`
- lane: test unsafe burndown
- fixture: `tqvector_recv` malformed binary receive fixtures
- storage format: N/A
- rerank mode: N/A
- isolated one-index-per-table or shared-table surface: N/A; no benchmark run

## Artifacts

### `git-diff-check.log`

- command: `git diff --check`
- result: passed
- key lines: empty log

### `rustfmt-check.log`

- command: not run as standalone rustfmt on included test file
- result: skipped by policy for module-included test sources
- key lines: `standalone rustfmt skipped: changed file is a src/tests include file whose indentation is owned by the including module; formatting checked by cargo parser instead`

### `hnsw-misc-unsafe-grep.log`

- command: `rg -n unsafe src/tests/hnsw_misc.rs`
- result: no matches; expected command exit code `1`
- key lines: no output

### `unsafe-count.log`

- command: `rg -n unsafe src | wc -l`
- result: passed
- key lines: `2205`

### `cargo-check-pg18-bench.log`

- command: `cargo check --all-targets --no-default-features --features pg18,bench`
- result: passed
- key lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 12.11s`
- warnings: existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`

### `cargo-test-lib-pg18-pgtest-no-run.log`

- command: `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- result: passed
- key lines: `Finished test profile [unoptimized + debuginfo] target(s) in 34.52s`
- warnings: existing Hadamard test-only dead-code warnings
