# Task 35 Packet 038: IVF Scan Rerank Probe Safety

## Code Under Review

- Commit: `b2b6b99e60ea44c0f6147dcd11744f71c8c3acad`
- Scope: `src/am/ec_ivf/scan.rs`

## Summary

This slice documents the heap rerank and probe materialization unsafe
boundaries in the IVF scan path. It covers ownership, pointer provenance, and
lifetime assumptions for the scan-owned rerank state, centroid score reads,
probe plan materialization, posting-block visitor references, heap tuple
fetch/rerank slots, prefetch/read-stream handling, and borrowed versus owned
heap relation/snapshot resolution.

Covered helpers:

- `free_heap_rerank_state`
- `configure_heap_rerank_state`
- `load_centroid_scores`
- `materialize_probe_candidates`
- `rerank_probe_candidates`
- `rerank_probe_candidates_heap_f32`
- `prefetch_heap_rerank_blocks`
- `resolve_scan_heap_relation`
- `resolve_scan_snapshot`

## Baseline Accounting

- Global unsafe-comment baseline: `2789 -> 2763`
- `src/am/ec_ivf/scan.rs`: `69 -> 43`

## Validation

- `artifacts/unsafe-baseline-report-before.log`: before-count report showing
  `69 src/am/ec_ivf/scan.rs`.
- `artifacts/unsafe-baseline-report-after.log`: after-count report showing
  `43 src/am/ec_ivf/scan.rs`.
- `artifacts/unsafe-baseline-update.log` and
  `artifacts/unsafe-baseline-update-after-fmt.log`: baseline regeneration
  logs after the comment updates and after formatting.
- `artifacts/unsafe-audit-after.log`: empty post-update unsafe audit.
- `artifacts/git-diff-check.log`: empty generated-baseline diff check.
- `artifacts/cargo-fmt.log`: `cargo fmt --all`.
- `artifacts/cargo-check-pg18-bench.log`:
  `cargo check --all-targets --no-default-features --features pg18,bench`
  completed successfully with the known unrelated warnings in
  `src/am/common/parallel.rs` and `src/am/mod.rs`.
- `artifacts/final-diff.patch`: final review diff for the slice.
