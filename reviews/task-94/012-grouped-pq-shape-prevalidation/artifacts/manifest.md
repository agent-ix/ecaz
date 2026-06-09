# Task 94 Packet 012 Artifact Manifest

- head SHA: `8ef7501b2395f5a299bab8fde1c351f52d5f4d41`
- task bucket: `reviews/task-94/012-grouped-pq-shape-prevalidation/`
- timestamp: `2026-06-09T18:09:42Z`
- lane: coder-1 LUT lane, Task 94 grouped-PQ block kernel
- fixture: local unit tests only
- storage format: n/a
- rerank mode: n/a
- isolated/shared table surface: n/a
- AWS/CI usage: none

## Artifacts

### `cargo-fmt-check.log`

- command: `script -q -c "cargo fmt --check" reviews/task-94/012-grouped-pq-shape-prevalidation/artifacts/cargo-fmt-check.log`
- result: passed
- key lines: stable rustfmt emitted existing nightly-only config warnings for `imports_granularity` and `group_imports`; command exited 0.

### `cargo-test-grouped-pq-batch.log`

- command: `script -q -c "cargo test grouped_pq_batch --lib" reviews/task-94/012-grouped-pq-shape-prevalidation/artifacts/cargo-test-grouped-pq-batch.log`
- result: passed
- key lines: `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 2046 filtered out`
- covered regression: `am::common::candidate_batch::tests::grouped_pq_batch_shape_error_scores_nothing_and_records_no_counters`
