# Task 50 Review Request: SPIRE DML Parameter Helper Boundaries

## Summary

This packet reviews commit
`92918fd8d523947d7bf2a947be8e0d3d59986255`, which continues the Task 50
unsafe burndown in the SPIRE DML front-door primitive invocation path.

The slice removes `2` direct unsafe blocks from
`src/am/ec_spire/dml_frontdoor/mod.rs` (`30 -> 28`) by making the public
primitive parameter helpers safe to call and keeping raw `ParamListInfo` access
inside one internal helper boundary.

## What Changed

- Made `dml_frontdoor_primitive_plan_pk_value_bytes` safe to call.
- Made `dml_frontdoor_primitive_invocation_from_plan` safe to call.
- Kept `ParamListInfo` dereference, `paramFetch`, array indexing, and
  `ParamExternData` copying inside `dml_frontdoor_bound_param_bigint_value`.
- Updated the pg_test call site that no longer requires the unsafe test macro.

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
| `src/` total direct unsafe blocks | 1917 | 1915 | -2 |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 30 | 28 | -2 |
| `src/tests/dml_frontdoor.rs` | 5 | 5 | 0 |
| `src/` unsafe ledger rows | 1917 | 1915 | -2 |

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1915` current `src/`
  unsafe rows.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`.
