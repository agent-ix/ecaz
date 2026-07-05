# Task 35 Review Request: HNSW Scan Debug Result-State Safety

## Summary

Documented the remaining unsafe boundaries in `src/am/ec_hnsw/scan_debug.rs` and removed the file from `scripts/unsafe_comment_baseline.txt`.

The comments cover:

- grouped scan comparison/window wrappers
- gettuple exhaustion, rescan, and backward-direction probes
- order-by score pointer/null handling
- current-result, candidate-frontier, and visited-set lifecycle probes
- bootstrap frontier consume/refill helpers
- graph adjacency lookup for current-result and entry-point debug helpers
- heap TID extraction from successful gettuple calls
- AM cleanup and `IndexScanEnd` ownership boundaries

## Code Under Review

- Code commit: `ddbdfcffa644f9ddad07275b215c029e6774bfc8`
- Files changed:
  - `src/am/ec_hnsw/scan_debug.rs`
  - `scripts/unsafe_comment_baseline.txt`

## Unsafe Baseline Movement

- Global baseline: `1414 -> 1233`
- Baseline files: `43 -> 42`
- `src/am/ec_hnsw/scan_debug.rs`: `181 -> 0`

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
