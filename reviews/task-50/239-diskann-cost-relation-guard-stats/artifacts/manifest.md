---
task: 50
packet: reviews/task-50/239-diskann-cost-relation-guard-stats
head_sha: 82385aff1ffe6c9dfb6e8fb872b0907256d69508
timestamp: 2026-05-21T06:51:40-07:00
lane: DiskANN unsafe burndown
storage_format: pq_fastscan
rerank_mode: DiskANN rerank budget relation/session options
surface: DiskANN cost relation stats
---

# Manifest

## Code Checkpoint

- Commit: `82385aff1ffe6c9dfb6e8fb872b0907256d69508`
- Summary:
  - reused `DiskannInsertRelation` for cost-path main-fork block count and reltuples reads;
  - added a guarded `reltuples` method to `DiskannInsertRelation`;
  - removed direct `relation_reltuples(index_relation)` unsafe blocks from DiskANN cost code.
- Programs advanced: P2 PostgreSQL Handle Views, DiskANN follow-up unsafe burndown.
- Touched-file unsafe counts:
  - `src/am/ec_diskann/cost.rs`: `9 -> 7`
  - `src/am/ec_diskann/insert.rs`: `17 -> 18`
- Source unsafe count:
  - Previous packet count: `2490`
  - This packet count: `2489`
  - Delta: `-1`

## Validation Artifacts

- `artifacts/unsafe-counts.log`
  - Command: before/after `unsafe` counts for touched files using `HEAD^`, plus current `src` count.
  - Result: DiskANN cost `9 -> 7`, DiskANN insert `17 -> 18`, repo `2490 -> 2489`.
- `artifacts/rustfmt-check.log`
  - Command: `rustfmt --edition 2021 --check src/am/ec_diskann/cost.rs src/am/ec_diskann/insert.rs`
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
- The change intentionally keeps `DiskannInsertRelation::from_raw` unsafe to preserve the PostgreSQL live-relation contract.
