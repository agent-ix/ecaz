# Task 50 Review Request: SPIRE DML Query View Safe Signatures

## Summary

This slice introduces `DmlFrontdoorQueryView` as the typed guard for DML frontdoor query-shape helpers and removes raw `Query*` from their safe signatures.

Code commit: `a38e0cfaf264a9bf86784124a83b2c5f5a3e45bc`

## What Changed

- Added `dml_frontdoor_query_view` as the explicit unsafe boundary for raw analyzed `Query*` conversion.
- Converted `dml_frontdoor_target_relation_oid`, `classify_dml_frontdoor_query`, `dml_frontdoor_replacement_decision_catalog_row`, and `dml_frontdoor_primitive_plan_expr_catalog_row` to take `DmlFrontdoorQueryView`.
- Updated SQL diagnostics, planner-hook internals, and DML frontdoor tests to construct the query view at the boundary and pass it through safe helpers.

## Completion Audit Note

This packet does not close Task 50. The current audit still finds `1937` unsafe line hits under `src/`, so packet 030 Wave 5 closeout is not satisfied.

## Review Focus

- Please verify the safe query-shape helpers no longer accept raw `pg_sys::Query` pointers.
- Please check that `DmlFrontdoorQueryView` is constructed only from analyzed/planner-owned query pointers with an explicit unsafe boundary.
- Please check the planner hook still observes and optionally replaces the same live query tree as before.

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- No-match audit for removed unsafe DML query-shape helper signatures and old raw-pointer call paths.
- `make UNSAFE_LEDGER=reviews/task-50/326-spire-dml-query-view-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/326-spire-dml-query-view-safe-signatures unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/326-spire-dml-query-view-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

## Counts

- Unsafe line count: `1937` (down from packet 325 `1944`)
- Unsafe ledger rows: `1356`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/no-unsafe-dml-query-shape-signatures.log`
- `artifacts/unsafe-line-count.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
