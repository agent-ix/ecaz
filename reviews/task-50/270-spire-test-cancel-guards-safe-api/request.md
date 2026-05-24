# Task 50 Review Request: SPIRE Test Cancel Guards Safe API

## Summary

This slice makes the SPIRE test-only query-cancel and statement-timeout guard constructors safe:

- `ScopedPgQueryCancelFlags::set_pending`
- `ScopedPgQueryCancelFlags::clear_pending_for_test`
- `ScopedPgStatementTimeoutSignal::trigger_after_ms`

These helpers resolve PostgreSQL backend symbols internally, check for null pointers, and restore global interrupt state through guard `Drop` implementations where appropriate. The raw symbol lookup, transmute, and backend-global writes remain internal explicit unsafe operations. Call sites in SPIRE custom scan, insert, and transport-fault tests now call the guard APIs directly.

## Unsafe Burndown

- Previous broad count from packet 269: `2221`
- Current broad count: `2212`
- Net: `-9`

## Validation

Artifacts are under `reviews/task-50/270-spire-test-cancel-guards-safe-api/artifacts/`.

- `git-diff-check.log`: passed
- `rustfmt-check.log`: standalone rustfmt skipped because changed files are module-included test sources; syntax/format viability was checked by cargo parsing
- `spire-test-guard-unsafe-grep.log`: no unsafe calls or unsafe fn signatures remain for these three guard APIs
- `unsafe-count.log`: `2212`
- `cargo-check-pg18-bench.log`: passed with the existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`
- `cargo-test-lib-pg18-pgtest-no-run.log`: passed with existing Hadamard test-only dead-code warnings

