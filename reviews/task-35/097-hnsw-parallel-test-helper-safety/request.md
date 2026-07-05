# Task 35 Review Request: HNSW Parallel Test Helper Safety

## Summary

Documented the remaining unsafe boundaries in `src/am/ec_hnsw/build_parallel.rs` and removed that file from the unsafe baseline.

The comments cover:

- parallel scan workspace sizing and shared scan descriptor pointer arithmetic
- DSM estimator mutation helpers
- concurrent DSM graph image test initialization and reattachment
- test readback and insertion fixtures over raw graph image parts
- aligned test DSM buffer allocation/deallocation
- synthetic test LWLock initialization and noop guard construction

## Code Under Review

- Code commit: `a88f8370a74fc07123df4d554dfabcb13a33d1a1`
- Files changed:
  - `src/am/ec_hnsw/build_parallel.rs`
  - `scripts/unsafe_comment_baseline.txt`

## Unsafe Baseline Movement

- Global baseline: `1069 -> 1030`
- Baseline files: `42 -> 41`
- `src/am/ec_hnsw/build_parallel.rs`: `39 -> 0`

## Validation

- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `awk 'BEGIN{n=0} index($0,"src/am/ec_hnsw/build_parallel.rs:")==1{print $0; n++} END{print "entries: " n}' scripts/unsafe_comment_baseline.txt`
- `cargo fmt --all`
- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

`cargo check` passed with the existing unrelated warnings for the unused `EC_PARALLEL_WORKER_SLOT_CLAIMED` import in `src/am/common/parallel.rs` and unused SPIRE re-exports in `src/am/mod.rs`.

## Artifacts

See `artifacts/manifest.md` for packet-local artifact metadata and command output paths.
