# Task 35 Review Request: HNSW Parallel DSM Insert/Search Safety

## Summary

Documented the next `src/am/ec_hnsw/build_parallel.rs` unsafe slice, focused on concurrent DSM graph readback, insert orchestration, layer search, successor loading, and backlink mutation.

The comments cover:

- graph image readback into build nodes
- current-format flush staging from an attached concurrent DSM graph
- node insert begin/complete state transitions under per-node locks
- partition and participant insert orchestration
- upper-layer search calls and successor loading
- source/neighbor node lock acquisition and insert-state reads
- backlink target mutation while holding the target node lock

## Code Under Review

- Code commit: `85ed2b9563c7cb553544fc902692847540012b54`
- Files changed:
  - `src/am/ec_hnsw/build_parallel.rs`
  - `scripts/unsafe_comment_baseline.txt`

## Unsafe Baseline Movement

- Global baseline: `1214 -> 1171`
- Baseline files: `42 -> 42`
- `src/am/ec_hnsw/build_parallel.rs`: `184 -> 141`

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
