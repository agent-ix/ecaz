# Task 35 Packet 080: SPIRE Custom Scan Planner Safety Comments

## Code Under Review

- Commit: `4efbd1c7f19576911252c0d13bc809905c51d2c2`
- Files:
  - `src/am/ec_spire/custom_scan/planner.rs`
  - `scripts/unsafe_comment_baseline.txt`

## Scope

This slice burns down the remaining unsafe-comment baseline for the SPIRE custom scan planner hook and path construction surface.

The added comments document safety boundaries for:

- SPIRE root/control and placement-directory reads used by custom scan eligibility;
- PostgreSQL planner hook chaining and live planner pointer ownership;
- CustomPath and CustomScan node allocation and transfer back to PostgreSQL planner memory;
- DML custom scan expression copying from live planner trees;
- planner-owned relation/index lists and bounded SPIRE index eligibility probes;
- placement catalog lookups under relation, snapshot, scan, and slot guards.

## Baseline Movement

- Global unsafe-comment baseline: `1870 -> 1833`
- `src/am/ec_spire/custom_scan/planner.rs`: `37 -> 0`

## Validation

- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/unsafe-baseline-report-after.log`: baseline is `1833` entries across `53` files.
- `artifacts/custom-scan-planner-baseline-after.log`: planner entries are `0`.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed.

Known unrelated warnings remain:

- unused `EC_PARALLEL_WORKER_SLOT_CLAIMED` in `src/am/common/parallel.rs`;
- unused SPIRE imports/re-exports in `src/am/mod.rs`.

`cargo fmt --all` emitted the repo's existing stable-rustfmt warnings for unstable rustfmt options. It also touched `hardening/careful/src/lib.rs` and `src/quant/simd.rs`; those unrelated formatting changes were restored before commit.
