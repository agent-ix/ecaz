# Task 50 Review Request: HNSW Grouped Storage Debug Classifier Safe API

## Summary

This slice makes `debug_scan_uses_grouped_storage` a safe helper. The helper
takes an index OID, opens its own `IndexRelationGuard`, and contains the raw
metadata/storage descriptor reads internally, so callers do not need to satisfy
an external raw pointer contract.

Four grouped-scan debug summaries now call the helper directly instead of
wrapping it in local unsafe blocks.

## Unsafe Burndown

- Previous broad count from packet 275: `2202`
- Current broad count: `2197`
- Net: `-5`

## Validation

Artifacts are under `reviews/task-50/276-hnsw-grouped-storage-debug-classifier-safe/artifacts/`.

- `git-diff-check.log`: passed
- `rustfmt-check.log`: standalone rustfmt skipped; syntax/format viability was checked by cargo parsing and `git diff --check`
- `grouped-storage-classifier-unsafe-grep.log`: no unsafe classifier signature or unsafe classifier call wrappers remain
- `unsafe-count.log`: `2197`
- `cargo-check-pg18-bench.log`: passed with the existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`
- `cargo-test-lib-pg18-pgtest-no-run.log`: passed with existing Hadamard test-only dead-code warnings
