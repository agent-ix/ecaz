---
task: 50
packet: reviews/task-50/241-diskann-dead-scan-heap-tid-decoder
head_sha: 9e16cb69f5674fe958005689e06aa97f8ec972ac
timestamp: 2026-05-21T07:02:39-07:00
lane: DiskANN unsafe burndown
storage_format: pq_fastscan
rerank_mode: n/a
surface: DiskANN scan-state dead helper removal
---

# Manifest

## Code Checkpoint

- Commit: `9e16cb69f5674fe958005689e06aa97f8ec972ac`
- Summary:
  - deleted unused `scan_state::decode_heap_tid`;
  - removed the unused `item_pointer_get_both` import from `scan_state.rs`;
  - preserved the live `ambuild::decode_heap_tid` callback helper.
- Programs advanced: DiskANN follow-up unsafe burndown.
- Touched-file unsafe counts:
  - `src/am/ec_diskann/scan_state.rs`: `18 -> 16`
- Source unsafe count:
  - Previous packet count: `2484`
  - This packet count: `2482`
  - Delta: `-2`

## Validation Artifacts

- `artifacts/unsafe-counts.log`
  - Command: before/after `unsafe` counts for touched file using `HEAD^`, current `src` count, and remaining `decode_heap_tid` references.
  - Result: DiskANN scan state `18 -> 16`, repo `2484 -> 2482`; remaining `decode_heap_tid` references are the live ambuild helper and its callers.
- `artifacts/rustfmt-check.log`
  - Command: `rustfmt --edition 2021 --check src/am/ec_diskann/scan_state.rs`
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
- This packet only removes a dead helper; it does not change live heap TID decoding behavior.
