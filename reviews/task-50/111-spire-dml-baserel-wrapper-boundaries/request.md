# Task 50 Review Request: SPIRE DML Baserel Wrapper Boundaries

## Summary

This packet reviews commit
`9f897e2df3d57f54ccedc69d0ead645cb142715f`, which removes caller-side unsafe
blocks from SPIRE DML baserel primitive-plan helpers and the custom-scan planner
PK-select candidate path.

The slice removes `4` direct unsafe blocks from `src/` (`1680 -> 1676`).

## What Changed

- Made `dml_frontdoor_primitive_plan_expr_from_baserel` safe to call, with raw
  planner pointer validation and dereferences retained inside the shared
  extractor.
- Made the PK SELECT, UPDATE, and DELETE baserel primitive-plan mode wrappers
  safe to call.
- Removed the now-unnecessary unsafe call at the custom-scan PK SELECT
  candidate check.

## Plan Coverage

This advances the comprehensive Task 50 plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`:

- P1 FFI And Callback Boundary Contracts: custom-scan planner callers no longer
  need unsafe invocation for the DML primitive-plan helper.
- P2 PostgreSQL Handle Views: raw planner pointer access is concentrated inside
  the shared DML baserel extractor instead of repeated by mode-specific wrappers.
- SPIRE remains the production target; this packet continues the DML/custom-scan
  cleanup after the remote candidate wrapper passes.

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
| `src/` total direct unsafe blocks | 1680 | 1676 | -4 |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 28 | 25 | -3 |
| `src/am/ec_spire/custom_scan/planner.rs` | 10 | 9 | -1 |
| `src/` unsafe ledger rows | 1680 | 1676 | -4 |

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check 9f897e2d^ 9f897e2d`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1676` current `src/`
  unsafe rows.

No live pgrx smoke was run for this slice because it is a wrapper-boundary
cleanup and does not add a new PostgreSQL callback or runtime path.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan.
