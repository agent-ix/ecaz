# Task 50 Review Request: Planner Cost GUC Callers

## Summary

This packet reviews commit
`0e94c73dbdf818290793740206a8d22f3a242959`, which makes AM planner cost GUC
reads safe to call and removes caller-side unsafe wrappers across SPIRE,
IVF/RaBitQ, DiskANN, HNSW, and shared AM cost code.

The slice removes `8` direct unsafe blocks from `src/` (`1801 -> 1793`).

## What Changed

- Made `current_planner_cost_constants` safe to call.
- Added safe `current_cpu_tuple_cost` for SPIRE custom scan costing.
- Kept the residual backend-local PostgreSQL global reads centralized in
  `src/am/common/cost.rs`.
- Removed repeated caller-side unsafe wrappers from planner cost paths.

## Plan Coverage

This advances the comprehensive Task 50 plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`:

- P1 FFI And Callback Boundary Contracts: AM planner callbacks no longer own
  repeated unsafe GUC reads.
- P2 PostgreSQL Handle Views: planner cost state is accessed through a shared
  safe API rather than direct `pg_sys` globals at each caller.
- Wave 2 SPIRE/IVF priority work and Wave 3 HNSW/DiskANN fanout: this is a
  shared boundary that removes unsafe across all four AM families.

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
| `src/` total direct unsafe blocks | 1801 | 1793 | -8 |
| `src/am/ec_diskann/cost.rs` | 11 | 9 | -2 |
| `src/am/ec_hnsw/shared.rs` | 44 | 43 | -1 |
| `src/am/ec_ivf/cost.rs` | 6 | 4 | -2 |
| `src/am/ec_spire/cost/mod.rs` | 15 | 13 | -2 |
| `src/am/ec_spire/custom_scan/cost_helpers.rs` | 3 | 2 | -1 |
| `src/` unsafe ledger rows | 1801 | 1793 | -8 |

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1793` current `src/`
  unsafe rows.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan.
