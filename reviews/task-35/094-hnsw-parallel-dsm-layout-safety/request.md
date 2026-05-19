# Task 35 Review Request: HNSW Parallel DSM Layout Safety

## Summary

Documented the first safety-comment slice in `src/am/ec_hnsw/build_parallel.rs`, focused on the concurrent DSM layout and attachment boundary.

The comments cover:

- DSM insert-state atomic load/store/compare-exchange wrappers
- PostgreSQL `IndexInfo` parallel worker field access
- LWLock shared/exclusive operation dispatch
- concurrent DSM graph header reads and layout derivation
- insert configuration reads from graph images
- graph attachment and typed graph-region pointer derivation
- one-time graph image initialization writes

## Code Under Review

- Code commit: `e2f8df2329c7da12abe1f44dbf7be830baa1681b`
- Files changed:
  - `src/am/ec_hnsw/build_parallel.rs`
  - `scripts/unsafe_comment_baseline.txt`

## Unsafe Baseline Movement

- Global baseline: `1233 -> 1214`
- Baseline files: `42 -> 42`
- `src/am/ec_hnsw/build_parallel.rs`: `203 -> 184`

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
