# Task 50 Review Request: SPIRE DML Test Fixture Safe Wrappers

## Summary

This slice reduces repeated SPIRE DML frontdoor test unsafe by centralizing the
fixture contracts:

- `src/tests/dml_frontdoor.rs` now uses local fixture closures for analyzed
  `Query*` helpers instead of repeating unsafe at every assertion.
- `expr_node_tag` owns the primitive-plan expression node tag read.
- `src/tests/build.rs` no longer dereferences `rd_options` directly; it uses a
  test-only `RelationGuard::std_rd_options_autovacuum_enabled` helper.

The remaining unsafe in these tests is either the parameter-list construction
fixture or the centralized analyzed-query wrapper closures.

## Unsafe Burndown

- Previous broad count from packet 276: `2197`
- Current broad count: `2187`
- Net: `-10`

## Validation

Artifacts are under `reviews/task-50/277-spire-dml-test-fixture-safe-wrappers/artifacts/`.

- `git-diff-check.log`: passed
- `rustfmt-check.log`: standalone rustfmt skipped because changed files include module-included test sources; syntax/format viability was checked by cargo parsing and `git diff --check`
- `spire-test-unsafe-grep.log`: `src/tests/build.rs` is clear; remaining DML frontdoor unsafe is centralized in local fixture helpers/closures
- `unsafe-count.log`: `2187`
- `cargo-check-pg18-bench.log`: passed with the existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`
- `cargo-test-lib-pg18-pgtest-no-run.log`: passed with existing Hadamard test-only dead-code warnings
