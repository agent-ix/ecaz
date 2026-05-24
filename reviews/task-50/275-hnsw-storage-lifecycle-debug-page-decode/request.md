# Task 50 Review Request: HNSW Storage Lifecycle Debug Page Decode

## Summary

This slice removes the raw graph-loader calls from the PqFastScan live insert
storage lifecycle test. The test already captures copied page tuple bytes from
`am::debug_index_pages`, so it now decodes the newly inserted grouped-hot tuple
and its rerank tuple with the safe page decoders:

- `am::page::TqGroupedHotTuple::decode`
- `am::page::TqRerankTuple::decode`

That keeps the assertion on the persisted tuple contents while avoiding a
second relation open and two direct unsafe graph reads in the test body.

## Unsafe Burndown

- Previous broad count from packet 274: `2204`
- Current broad count: `2202`
- Net: `-2`

## Validation

Artifacts are under `reviews/task-50/275-hnsw-storage-lifecycle-debug-page-decode/artifacts/`.

- `git-diff-check.log`: passed
- `rustfmt-check.log`: standalone rustfmt skipped because changed file is a module-included test source; syntax/format viability was checked by cargo parsing
- `storage-lifecycle-unsafe-grep.log`: no `unsafe` matches remain in `src/tests/ec_hnsw_storage_lifecycle.rs`
- `unsafe-count.log`: `2202`
- `cargo-check-pg18-bench.log`: passed with the existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`
- `cargo-test-lib-pg18-pgtest-no-run.log`: passed with existing Hadamard test-only dead-code warnings
