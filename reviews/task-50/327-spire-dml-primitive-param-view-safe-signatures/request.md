# Task 50 Review Request: SPIRE DML Primitive Param View Safe Signatures

## Summary

This slice introduces `DmlFrontdoorParamListInfo` as the typed boundary for DML primitive executor-parameter reads and removes raw `ParamListInfo` from primitive helper signatures.

Code commit: `e337e33b64e084ec0529ce06a2b4b5ccb48742ab`

## What Changed

- Added `dml_frontdoor_param_list_info` as the explicit unsafe boundary for raw `ParamListInfo`.
- Converted `dml_frontdoor_primitive_plan_pk_value_bytes` and `dml_frontdoor_primitive_invocation_from_plan` to safe helpers over `DmlFrontdoorParamListInfo`.
- Updated DML frontdoor tests to construct the parameter-list view once and pass it through safe primitive helpers.

## Completion Audit Note

This packet does not close Task 50. The current audit still finds `1935` unsafe line hits under `src/`, so packet 030 Wave 5 closeout is not satisfied.

## Review Focus

- Please verify primitive helpers no longer accept raw `pg_sys::ParamListInfo`.
- Please check the null-parameter-list path remains valid only for const-PK plans that do not read executor parameters.
- Please check parameter validation still fails closed for null, NULL, missing, and non-bigint parameter slots.

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- No-match audit for removed unsafe primitive param helper signatures and old raw-param call paths.
- `make UNSAFE_LEDGER=reviews/task-50/327-spire-dml-primitive-param-view-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/327-spire-dml-primitive-param-view-safe-signatures unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/327-spire-dml-primitive-param-view-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

## Counts

- Unsafe line count: `1935` (down from packet 326 `1937`)
- Unsafe ledger rows: `1355`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/no-unsafe-dml-primitive-param-signatures.log`
- `artifacts/unsafe-line-count.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
