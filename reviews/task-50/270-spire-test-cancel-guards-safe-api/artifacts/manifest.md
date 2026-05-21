# Artifact Manifest: Task 50 Packet 270

- head SHA: `11eca46b7f2a2247db02785db30dc3115291f5f1`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/270-spire-test-cancel-guards-safe-api`
- timestamp: `2026-05-21T10:04:13-07:00`
- lane: SPIRE unsafe burndown
- fixture: test-only SPIRE cancel/timeout guards
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

### `spire-test-guard-unsafe-grep.log`

- command: `rg -n "unsafe \\{ ScopedPg(QueryCancelFlags|StatementTimeoutSignal)::|unsafe fn (set_pending|clear_pending_for_test|trigger_after_ms)" src/tests`
- result: no matches
- key lines: `no unsafe calls or unsafe fn signatures remain for ScopedPgQueryCancelFlags::set_pending, ScopedPgQueryCancelFlags::clear_pending_for_test, or ScopedPgStatementTimeoutSignal::trigger_after_ms`

### `unsafe-count.log`

- command: `rg -n "unsafe" src | wc -l`
- result: passed
- key lines: `2212`

### `cargo-check-pg18-bench.log`

- command: `cargo check --all-targets --no-default-features --features pg18,bench`
- result: passed
- key lines: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.16s`
- warnings: existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`

### `cargo-test-lib-pg18-pgtest-no-run.log`

- command: `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- result: passed
- key lines: `Finished test profile [unoptimized + debuginfo] target(s) in 42.86s`
- warnings: existing Hadamard test-only dead-code warnings

