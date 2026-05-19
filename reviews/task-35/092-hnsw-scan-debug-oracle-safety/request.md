# Task 35 Review Request: HNSW Scan Debug Oracle Safety

## Summary

Documented the next `src/am/ec_hnsw/scan_debug.rs` unsafe cluster covering grouped/turboquant profile probes, graph page collectors, oracle seed traversal helpers, exact-seed scans, and grouped score-comparison scan helpers.

The comments cover:

- heap-backed profile scan ownership and gettuple loops
- turboquant scan opaque, cached quantizer, and prepared-query access
- graph page block counting, locked page reads, tuple-tag checks, and graph element loads
- top-level and layer oracle seed collection/search
- graph adjacency and layer-0 candidate traversal
- exact seed heap-TID to graph-element mapping
- score-comparison scan heap TID extraction
- grouped-storage debug classification from metadata

## Code Under Review

- Code commit: `27d3c5f0d2cd7cc95a34866d83d08bc3f69311e8`
- Files changed:
  - `src/am/ec_hnsw/scan_debug.rs`
  - `scripts/unsafe_comment_baseline.txt`

## Unsafe Baseline Movement

- Global baseline: `1520 -> 1414`
- Baseline files: `43 -> 43`
- `src/am/ec_hnsw/scan_debug.rs`: `287 -> 181`

## Validation

- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `awk 'BEGIN{n=0} index($0,"src/am/ec_hnsw/scan_debug.rs:")==1{print $0; n++} END{print "entries: " n}' scripts/unsafe_comment_baseline.txt`
- `cargo fmt --all`
- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

`cargo check` passed with the existing unrelated warnings for the unused `EC_PARALLEL_WORKER_SLOT_CLAIMED` import in `src/am/common/parallel.rs` and unused SPIRE re-exports in `src/am/mod.rs`.

## Artifacts

See `artifacts/manifest.md` for packet-local artifact metadata and command output paths.
