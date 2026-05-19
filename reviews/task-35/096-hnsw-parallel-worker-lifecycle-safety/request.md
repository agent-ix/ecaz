# Task 35 Review Request: HNSW Parallel Worker Lifecycle Safety

## Summary

Documented the next `src/am/ec_hnsw/build_parallel.rs` unsafe slice, focused on raw DSM slice helpers, PostgreSQL parallel leader setup/cleanup, worker queue handoff, and worker entrypoint lifecycle.

The comments cover:

- concurrent DSM score/corpus slice derivation
- mutable and immutable DSM neighbor-slot slice helpers
- PostgreSQL parallel graph-build leader setup, DSM allocation, launch, wait, and cleanup
- PostgreSQL heap-build leader setup, shared header initialization, worker queue allocation, launch, drain, wait, and cleanup
- heap-build callback state access and worker entrypoints
- worker TOC lookups, relation opening, parallel table scan setup, worker counter publication, and instrumentation accounting
- worker `shm_mq` tuple/done message sends

## Code Under Review

- Code commit: `cf9102ea84ed46c46d53d9f3129759a746c35f4b`
- Files changed:
  - `src/am/ec_hnsw/build_parallel.rs`
  - `scripts/unsafe_comment_baseline.txt`

## Unsafe Baseline Movement

- Global baseline: `1171 -> 1069`
- Baseline files: `42 -> 42`
- `src/am/ec_hnsw/build_parallel.rs`: `141 -> 39`

## Validation

- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `rg '^src/am/ec_hnsw/build_parallel.rs:' scripts/unsafe_comment_baseline.txt`
- `rg -c '^src/am/ec_hnsw/build_parallel.rs:' scripts/unsafe_comment_baseline.txt`
- `cargo fmt --all`
- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

`cargo check` passed with the existing unrelated warnings for the unused `EC_PARALLEL_WORKER_SLOT_CLAIMED` import in `src/am/common/parallel.rs` and unused SPIRE re-exports in `src/am/mod.rs`.

## Artifacts

See `artifacts/manifest.md` for packet-local artifact metadata and command output paths.
