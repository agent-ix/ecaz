---
task: 50
packet: reviews/task-50/240-diskann-scan-descriptor-view
head_sha: 4aaf5ea0a56f690f14543d8a9a7063d4447743fa
timestamp: 2026-05-21T06:58:42-07:00
lane: DiskANN unsafe burndown
storage_format: pq_fastscan
rerank_mode: heap rerank during scan
surface: DiskANN IndexScanDesc heap/snapshot resolution
---

# Manifest

## Code Checkpoint

- Commit: `4aaf5ea0a56f690f14543d8a9a7063d4447743fa`
- Summary:
  - introduced `DiskannScanDescView` as the local boundary around PostgreSQL `IndexScanDesc` field reads;
  - moved heap relation and snapshot resolution onto the view;
  - removed the standalone `resolve_scan_heap_relation` and `resolve_scan_snapshot` raw-pointer helpers.
- Programs advanced: P2 PostgreSQL Handle Views, P10 Scan Opaque And Raw Ownership Contracts, DiskANN follow-up unsafe burndown.
- Touched-file unsafe counts:
  - `src/am/ec_diskann/scan_state.rs`: `23 -> 18`
  - `src/am/ec_diskann/routine.rs`: `58 -> 58`
- Source unsafe count:
  - Previous packet count: `2489`
  - This packet count: `2484`
  - Delta: `-5`

## Validation Artifacts

- `artifacts/unsafe-counts.log`
  - Command: before/after `unsafe` counts for touched files using `HEAD^`, plus current `src` count.
  - Result: DiskANN scan state `23 -> 18`, DiskANN routine `58 -> 58`, repo `2489 -> 2484`.
- `artifacts/rustfmt-check.log`
  - Command: `rustfmt --edition 2021 --check src/am/ec_diskann/scan_state.rs src/am/ec_diskann/routine.rs`
  - Result: passed; emitted only the existing stable-rustfmt warnings for `imports_granularity` and `group_imports`.
- `artifacts/git-diff-check.log`
  - Command: `git diff --check HEAD^ HEAD`
  - Result: passed with no output.
- `artifacts/cargo-check-pg18-bench.log`
  - Command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Result: passed; emitted the known existing `src/am/mod.rs` unused SPIRE re-export warning.
- `artifacts/cargo-test-lib-ec-diskann-pg18-no-run.log`
  - Command: `cargo test --lib ec_diskann --no-default-features --features pg18,pg_test --no-run`
  - Result: passed; emitted the known existing Hadamard test helper dead-code warnings.

## Notes

- This was not a benchmark packet.
- No isolated index/table benchmark surface was used.
- `DiskannScanDescView::from_raw` intentionally remains `unsafe fn` because the caller must provide a live PostgreSQL scan descriptor.
