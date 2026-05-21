# Artifact Manifest: Task 50 Packet 271

- head SHA: `557bd79cefa8ce6020e6c0f2674f2edcc22eaf3b`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/271-dml-frontdoor-analyzed-query-safe-helper`
- timestamp: `2026-05-21T10:09:33-07:00`
- lane: SPIRE/test unsafe burndown
- fixture: DML-frontdoor analyzed query test helper
- storage format: N/A
- rerank mode: N/A
- isolated one-index-per-table or shared-table surface: N/A; no benchmark run

## Artifacts

### `git-diff-check.log`

- command: `git diff --check`
- result: passed
- key lines: empty log

### `rustfmt-check.log`

- command: not run as standalone rustfmt on included test files
- result: skipped by policy for module-included test sources
- key lines: `standalone rustfmt skipped: changed files are src/tests include files whose indentation is owned by the including module; formatting checked by cargo parser instead`

### `dml-frontdoor-analyzed-query-grep.log`

- command: `rg -n "dml_frontdoor_checked|unsafe fn analyzed_query" src/tests/dml_frontdoor.rs src/tests/mod.rs`
- result: no matches
- key lines: `no dml_frontdoor_checked macro/call sites or unsafe analyzed_query signature remain`

### `unsafe-count.log`

- command: `rg -n "unsafe" src | wc -l`
- result: passed
- key lines: `2210`

### `cargo-check-pg18-bench.log`

- command: `cargo check --all-targets --no-default-features --features pg18,bench`
- result: passed
- key lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.16s`
- warnings: existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`

### `cargo-test-lib-pg18-pgtest-no-run.log`

- command: `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- result: passed
- key lines: `Finished test profile [unoptimized + debuginfo] target(s) in 35.82s`
- warnings: existing Hadamard test-only dead-code warnings

