---
task: 50
packet: reviews/task-50/236-planner-cost-global-accessors
head_sha: a05e664540ce31a5f0728f2ab3214524bd6fedcb
timestamp: 2026-05-21T05:31:48-07:00
lane: cross-lane planner cost unsafe burndown
storage_format: n/a
rerank_mode: n/a
surface: PostgreSQL planner cost global accessors
---

# Manifest

## Code Checkpoint

- Commit: `a05e664540ce31a5f0728f2ab3214524bd6fedcb`
- Summary:
  - made `current_planner_cost_constants` and `current_cpu_tuple_cost` safe common accessors;
  - removed caller-side unsafe blocks from IVF, SPIRE, SPIRE custom scan, DiskANN, HNSW, and common HNSW cost paths;
  - documented that the accessors copy PostgreSQL backend-local globals by value.
- Programs advanced: P2 PostgreSQL Handle Views, P13 IVF/RaBitQ Production Surface Cleanup, cross-lane planner cost cleanup.
- Touched-file unsafe counts:
  - `src/am/common/cost.rs`: `15 -> 12`
  - `src/am/ec_diskann/cost.rs`: `9 -> 7`
  - `src/am/ec_hnsw/shared.rs`: `64 -> 63`
  - `src/am/ec_ivf/cost.rs`: `10 -> 8`
  - `src/am/ec_spire/cost/mod.rs`: `26 -> 24`
  - `src/am/ec_spire/custom_scan/cost_helpers.rs`: `26 -> 24`
- Source unsafe count:
  - Previous packet count: `2492`
  - This packet count: `2480`
  - Delta: `-12`

## Validation Artifacts

- `artifacts/unsafe-counts.log`
  - Command: before/after `unsafe` counts for touched files using `HEAD^`, plus current `src` count.
  - Result: common cost `15 -> 12`, DiskANN cost `9 -> 7`, HNSW shared `64 -> 63`, IVF cost `10 -> 8`, SPIRE cost `26 -> 24`, SPIRE custom cost helpers `26 -> 24`, repo `2492 -> 2480`.
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
- The packet intentionally leaves relation descriptor dereferences out of scope; it only consolidates read-only planner-cost global access.
