---
task: 50
packet: reviews/task-50/234-spire-scan-descriptor-setup-and-dead-rerank-path
head_sha: d62863752a3de4f1397299fb8fba7c0f5fb3f92f
timestamp: 2026-05-21T05:16:07-07:00
lane: SPIRE production scan unsafe burndown
storage_format: n/a
rerank_mode: production scan heap resolution
surface: IndexScanDesc setup and stale heap-rerank preparation path
---

# Manifest

## Code Checkpoint

- Commit: `d62863752a3de4f1397299fb8fba7c0f5fb3f92f`
- Summary:
  - moved `ambeginscan` descriptor opaque installation onto `SpireIndexScanView`;
  - deleted the unreferenced legacy `prepare_single_level_relation_snapshot_scan_candidates` heap-rerank preparation path and its private helper functions.
- Programs advanced: P2 PostgreSQL Handle Views, P5 Heap Source/Tuple Slot/Snapshot/Scorer Contracts, P10 Scan Opaque And Raw Ownership Contracts.
- Touched-file unsafe counts:
  - `src/am/ec_spire/scan/callbacks.rs`: `4 -> 4`
  - `src/am/ec_spire/scan/relation.rs`: `22 -> 14`
- Source unsafe count:
  - Previous packet count: `2504`
  - This packet count: `2496`
  - Delta: `-8`

## Validation Artifacts

- `artifacts/touched-file-unsafe-counts.log`
  - Command: before/after `rg -n unsafe | wc -l` for touched files using `HEAD^` and working tree.
  - Result: callbacks `4 -> 4`, relation `22 -> 14`.
- `artifacts/rustfmt-check.log`
  - Command: `rustfmt --check src/am/ec_spire/scan/callbacks.rs src/am/ec_spire/scan/relation.rs`
  - Result: passed; emitted only the existing stable-rustfmt warnings for `imports_granularity` and `group_imports`.
- `artifacts/git-diff-check.log`
  - Command: `git diff --check`
  - Result: passed with no output.
- `artifacts/src-unsafe-count.log`
  - Command: `rg -n 'unsafe' src | wc -l`
  - Result: `2496`.
- `artifacts/cargo-check-pg18-bench.log`
  - Command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Result: passed; emitted the known existing `src/am/mod.rs` unused SPIRE re-export warning.
- `artifacts/cargo-test-ec-spire-pg18-pg-test-no-run.log`
  - Command: `cargo test --lib ec_spire --no-default-features --features pg18,pg_test --no-run`
  - Result: passed; emitted the known existing Hadamard test helper dead-code warnings.

## Notes

- This was not a benchmark packet.
- No isolated index/table benchmark surface was used.
- The deleted heap-rerank preparation function was unreferenced by production code and tests. The remaining `heap_rerank_prefetch_block_numbers` helper is still used by scan tests.
