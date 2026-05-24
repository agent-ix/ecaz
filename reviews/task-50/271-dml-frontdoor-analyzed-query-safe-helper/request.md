# Task 50 Review Request: DML Frontdoor Analyzed Query Safe Helper

## Summary

This slice makes the test-only `analyzed_query` helper safe. It constructs the analyzed PostgreSQL `Query*` internally from SQL text, with the parser/analyze raw calls isolated inside the helper.

The `dml_frontdoor_checked!` macro is removed because it only wrapped `analyzed_query`. DML-frontdoor test call sites now call `analyzed_query` directly. Production DML-frontdoor helpers that accept caller-owned raw `Query*` pointers remain explicitly unsafe; this packet does not hide that raw-pointer validity contract.

## Unsafe Burndown

- Previous broad count from packet 270: `2212`
- Current broad count: `2210`
- Net: `-2`

## Validation

Artifacts are under `reviews/task-50/271-dml-frontdoor-analyzed-query-safe-helper/artifacts/`.

- `git-diff-check.log`: passed
- `rustfmt-check.log`: standalone rustfmt skipped because changed files are module-included test sources; syntax/format viability was checked by cargo parsing
- `dml-frontdoor-analyzed-query-grep.log`: no `dml_frontdoor_checked` macro/call sites or unsafe `analyzed_query` signature remain
- `unsafe-count.log`: `2210`
- `cargo-check-pg18-bench.log`: passed with the existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`
- `cargo-test-lib-pg18-pgtest-no-run.log`: passed with existing Hadamard test-only dead-code warnings

