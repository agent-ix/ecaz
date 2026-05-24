# Task 50 Review Request: HNSW Recall Block Count Helper Reuse

## Summary

This slice removes a repeated raw relation block-count read from the graph scan
recall probe body. The probe now reuses the existing `recall_index_block_count`
helper instead of opening the HNSW index relation and calling
`main_fork_block_count` directly.

This keeps the raw relation lifetime contract centralized in the helper already
used by neighboring recall fixtures and removes the local unsafe block from
`src/tests/ec_hnsw_recall_helpers.rs`.

## Unsafe Burndown

- Previous broad count from packet 273: `2205`
- Current broad count: `2204`
- Net: `-1`

## Validation

Artifacts are under `reviews/task-50/274-hnsw-recall-block-count-helper-reuse/artifacts/`.

- `git-diff-check.log`: passed
- `rustfmt-check.log`: standalone rustfmt skipped because changed file is a module-included test source; syntax/format viability was checked by cargo parsing
- `recall-helper-unsafe-grep.log`: no `unsafe` matches remain in `src/tests/ec_hnsw_recall_helpers.rs`
- `unsafe-count.log`: `2204`
- `cargo-check-pg18-bench.log`: passed with the existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`
- `cargo-test-lib-pg18-pgtest-no-run.log`: passed with existing Hadamard test-only dead-code warnings
