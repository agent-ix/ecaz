---
task: 50
packet: reviews/task-50/238-restore-planner-cost-unsafe-contract
head_sha: f4c9aa753112cf1270db7322babe9ad50227795b
timestamp: 2026-05-21T05:41:04-07:00
lane: feedback fix / planner cost unsafe contract
storage_format: n/a
rerank_mode: n/a
surface: PostgreSQL planner cost global accessors
addresses_feedback:
  - reviews/task-50/236-planner-cost-global-accessors/feedback/2026-05-21-01-reviewer.md
---

# Manifest

## Code Checkpoint

- Commit: `f4c9aa753112cf1270db7322babe9ad50227795b`
- Summary:
  - restored `current_planner_cost_constants` and `current_cpu_tuple_cost` to `unsafe fn`;
  - restored caller-side unsafe acknowledgements and SAFETY comments across IVF, SPIRE, SPIRE custom scan, DiskANN, HNSW, and common HNSW cost paths;
  - intentionally unwound the code change from packet 236 after reviewer block.
- Programs advanced: feedback processing; consistency with round-1 soundness-audit convention.
- Source unsafe count:
  - Previous packet count: `2478`
  - This packet count: `2490`
  - Delta: `+12`

## Validation Artifacts

- `artifacts/unsafe-contract-counts.log`
  - Command: verify the two planner cost accessors are `unsafe fn`, plus before/after `src` count.
  - Result: both accessors are unsafe; repo `2478 -> 2490`.
- `artifacts/rustfmt-check.log`
  - Command: `rustfmt --edition 2021 --check src/am/common/cost.rs src/am/ec_diskann/cost.rs src/am/ec_hnsw/shared.rs src/am/ec_ivf/cost.rs src/am/ec_spire/cost/mod.rs src/am/ec_spire/custom_scan/cost_helpers.rs`
  - Result: passed; emitted only the existing stable-rustfmt warnings for `imports_granularity` and `group_imports`.
- `artifacts/git-diff-check.log`
  - Command: `git diff --check HEAD^ HEAD`
  - Result: passed with no output.
- `artifacts/cargo-check-pg18-bench.log`
  - Command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Result: passed; emitted the known existing `src/am/mod.rs` unused SPIRE re-export warning.
- `artifacts/cargo-test-lib-cost-pg18-no-run.log`
  - Command: `cargo test --lib cost --no-default-features --features pg18,pg_test --no-run`
  - Result: passed; emitted the known existing Hadamard test helper dead-code warnings.

## Notes

- This was not a benchmark packet.
- No isolated index/table benchmark surface was used.
- This packet intentionally increases explicit unsafe count to preserve the previously accepted audit contract.
