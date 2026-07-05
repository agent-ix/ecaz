# Task 50 Review Request: HNSW Planner Cost Boundary

## Summary

This packet reviews commit
`e50d9e59f9fba3f2428cfa27bf59ccb4613cdf95`, which removes local unsafe
wrappers from HNSW planner-cost callbacks by routing them through the shared
`pg_am_callback!` AM boundary, groups planner-cost global reads into one
residual block, and makes the planner tree-height helper safe to call.

The slice removes `7` direct unsafe blocks from `src/` (`1763 -> 1756`).

## What Changed

- Reused `pg_am_callback!` for HNSW planner cost/tree-height/strategy callback
  bodies instead of local `pgrx_extern_c_guard` unsafe blocks.
- Grouped PostgreSQL planner-cost global reads into one residual unsafe block.
- Made `planner_tree_height_from_index_info` safe to call with a null
  `IndexOptInfo` guard.
- Left the remaining raw field/global reads centralized in
  `src/am/common/cost.rs`.

## Plan Coverage

This advances the comprehensive Task 50 plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`:

- P1 FFI And Callback Boundary Contracts: HNSW planner callbacks now use the
  shared AM callback guard instead of per-callback unsafe guard blocks.
- P2 PostgreSQL Handle Views: planner tree-height access is behind a checked
  helper rather than requiring caller-side unsafe.
- Wave 3 HNSW cleanup: reduces the common HNSW planner-cost surface before
  deeper HNSW scan/build slices.

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
| `src/` total direct unsafe blocks | 1763 | 1756 | -7 |
| `src/am/common/cost.rs` | 13 | 6 | -7 |
| `src/` unsafe ledger rows | 1763 | 1756 | -7 |

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check e50d9e59^ e50d9e59`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1756` current `src/`
  unsafe rows.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan.
