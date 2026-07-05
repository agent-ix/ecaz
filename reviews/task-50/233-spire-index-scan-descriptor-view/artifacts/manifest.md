---
task: 50
packet: reviews/task-50/233-spire-index-scan-descriptor-view
head_sha: 295e4b0a96678b447376d2522724defd343ccc51
timestamp: 2026-05-21T05:11:33-07:00
lane: SPIRE production scan unsafe burndown
storage_format: n/a
rerank_mode: production scan heap resolution
surface: IndexScanDesc callback view
---

# Manifest

## Code Checkpoint

- Commit: `295e4b0a96678b447376d2522724defd343ccc51`
- Summary: added `SpireIndexScanView` for SPIRE scan callbacks and heap-rerank candidate preparation.
- Program advanced: P2 PostgreSQL Handle Views, P5 Heap Source/Tuple Slot/Snapshot/Scorer Contracts, P10 Scan Opaque And Raw Ownership Contracts.
- Touched-file unsafe counts:
  - `src/am/ec_spire/scan/callbacks.rs`: `4 -> 4`
  - `src/am/ec_spire/scan/relation.rs`: `24 -> 22`
- Source unsafe count:
  - Previous packet count: `2506`
  - This packet count: `2504`
  - Delta: `-2`

## Validation Artifacts

- `artifacts/touched-file-unsafe-counts.log`
  - Command: before/after `rg -n unsafe | wc -l` for touched files using `HEAD^` and working tree.
  - Result: callbacks `4 -> 4`, relation `24 -> 22`.
- `artifacts/rustfmt-check.log`
  - Command: `rustfmt --check src/am/ec_spire/scan/callbacks.rs src/am/ec_spire/scan/relation.rs`
  - Result: passed; emitted only the existing stable-rustfmt warnings for `imports_granularity` and `group_imports`.
- `artifacts/git-diff-check.log`
  - Command: `git diff --check`
  - Result: passed with no output.
- `artifacts/src-unsafe-count.log`
  - Command: `rg -n 'unsafe' src | wc -l`
  - Result: `2504`.
- `artifacts/cargo-check-pg18-bench.log`
  - Command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Result: passed; emitted the known existing `src/am/mod.rs` unused SPIRE re-export warning.
- `artifacts/cargo-test-ec-spire-pg18-pg-test-no-run.log`
  - Command: `cargo test --lib ec_spire --no-default-features --features pg18,pg_test --no-run`
  - Result: passed; emitted the known existing Hadamard test helper dead-code warnings.

## Notes

- This was not a benchmark packet.
- No isolated index/table benchmark surface was used.
- `SpireIndexScanView::from_raw` remains the explicit unsafe callback-boundary constructor. Safe methods now own heap relation fallback, snapshot fallback, recheck/orderby output mutation, and scan opaque access for the SPIRE scan callback surface.
