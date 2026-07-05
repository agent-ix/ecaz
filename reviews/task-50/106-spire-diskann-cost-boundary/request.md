# Task 50 Review Request: SPIRE And DiskANN Cost Boundary

## Summary

This packet reviews commit
`e76a31fca63b22fd10876fc50170658ed4fbacc7`, which removes repeated
caller-side unsafe from SPIRE cost snapshot paths and routes the DiskANN planner
cost callback through the shared AM callback boundary.

The slice removes `6` direct unsafe blocks from `src/` (`1756 -> 1750`).

## What Changed

- Converted the DiskANN `amcostestimate` callback body to `pg_am_callback!`.
- Added safe local SPIRE cost helpers for active snapshot diagnostics and
  hierarchy snapshots.
- Reused those helpers from SPIRE cost snapshot, tuning snapshot,
  `amcostestimate`, and tree-height callback logic.
- Left the remaining raw SPIRE snapshot calls centralized in two helper
  functions in `src/am/ec_spire/cost/mod.rs`.

## Plan Coverage

This advances the comprehensive Task 50 plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`:

- P1 FFI And Callback Boundary Contracts: DiskANN planner callbacks now use the
  shared AM callback guard.
- P2 PostgreSQL Handle Views: SPIRE cost paths no longer repeat unsafe snapshot
  calls at each cost/tuning/tree-height caller.
- SPIRE remains the production target; this packet reduces the SPIRE planner
  cost surface before returning to deeper SPIRE coordinator/custom-scan areas.

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
| `src/` total direct unsafe blocks | 1756 | 1750 | -6 |
| `src/am/ec_diskann/cost.rs` | 3 | 2 | -1 |
| `src/am/ec_spire/cost/mod.rs` | 7 | 2 | -5 |
| `src/` unsafe ledger rows | 1756 | 1750 | -6 |

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check e76a31fc^ e76a31fc`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1750` current `src/`
  unsafe rows.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan.
