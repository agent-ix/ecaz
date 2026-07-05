# Task 35 Review Request: HNSW Scan Debug Lifecycle Safety

## Summary

Documented the first `src/am/ec_hnsw/scan_debug.rs` unsafe cluster covering debug scan setup, rescan/gettuple probes, heap-backed profile collection, and early debug state inspection helpers.

The comments cover:

- page tuple byte inspection through the shared bounds-checking helper
- debug graph adjacency loading for layer neighbor inspection
- scan opaque-owned debug sets (`visited_tids`, `expanded_source_tids`, `emitted_result_tids`)
- AM begin/rescan/gettuple/end lifecycle probes
- heap-backed scan state ownership and heap TID extraction
- ordered scan profile gettuple loops
- heap-fetch profile `index_rescan`, `index_getnext_slot`, projection, and slot clearing

## Code Under Review

- Code commit: `1e256fa59f0210f5d5b9d1b07c325440af005866`
- Files changed:
  - `src/am/ec_hnsw/scan_debug.rs`
  - `scripts/unsafe_comment_baseline.txt`

## Unsafe Baseline Movement

- Global baseline: `1587 -> 1520`
- Baseline files: `43 -> 43`
- `src/am/ec_hnsw/scan_debug.rs`: `354 -> 287`

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
