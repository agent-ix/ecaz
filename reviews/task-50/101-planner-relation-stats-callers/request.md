# Task 50 Review Request: Planner Relation Stats Callers

## Summary

This packet reviews commit
`b1333a5eef6fe016791d2dd2836f07f13a08baf0`, which centralizes AM planner
relation-stat reads and removes caller-side unsafe wrappers across SPIRE,
IVF/RaBitQ, DiskANN, HNSW, and shared AM cost code.

The slice removes `15` direct unsafe blocks from `src/` (`1793 -> 1778`) and
drops `src/am/ec_ivf/cost.rs` to zero direct unsafe blocks.

## What Changed

- Added safe `relation_main_fork_block_count` in `src/am/common/cost.rs`.
- Added safe `relation_reltuples` in `src/am/common/cost.rs`.
- Replaced direct `RelationGetNumberOfBlocksInFork` and `rd_rel.reltuples`
  reads in SPIRE, IVF/RaBitQ, DiskANN, HNSW, and shared AM cost paths.
- Kept the residual PostgreSQL relation-stat boundary centralized in AM common
  with null relation checks.

## Plan Coverage

This advances the comprehensive Task 50 plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`:

- P1 FFI And Callback Boundary Contracts: AM planner callbacks no longer own
  repeated relation-stat unsafe reads.
- P2 PostgreSQL Handle Views: relation stats are accessed through shared safe
  helpers with a named residual owner.
- Wave 2 SPIRE/IVF priority work and Wave 3 HNSW/DiskANN fanout: this removes
  unsafe across all four AM families.

## Evidence

- Code diff: `artifacts/code-diff.patch`
- Validation: `artifacts/cargo-check-pg18-bench.log`
- Whitespace check: `artifacts/git-diff-check.log`
- Unsafe count: `artifacts/src-unsafe-block-count-after.log`
- Count summary: `artifacts/count-summary.md`
- Ledger: `artifacts/unsafe-ledger-after.jsonl`
- Ledger generation/check logs:
  `artifacts/unsafe-ledger-generate.log`,
  `artifacts/unsafe-ledger-check.log`

## Result

Direct unsafe movement:

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/` total direct unsafe blocks | 1793 | 1778 | -15 |
| `src/am/ec_diskann/cost.rs` | 9 | 5 | -4 |
| `src/am/ec_hnsw/shared.rs` | 43 | 42 | -1 |
| `src/am/ec_ivf/cost.rs` | 4 | 0 | -4 |
| `src/am/ec_spire/cost/mod.rs` | 13 | 7 | -6 |
| `src/` unsafe ledger rows | 1793 | 1778 | -15 |

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1778` current `src/`
  unsafe rows.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan.
