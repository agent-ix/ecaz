# Review Request: SPIRE DML Const-Plan Param Helper

Task: 50 unsafe burndown

Commit under review:

- `d45e9c22` - `Remove DML const plan null param unsafe`

## Summary

This packet removes a test-only direct unsafe block from the SPIRE DML primitive-plan test path.

- Adds `dml_frontdoor_const_plan_param_list_info()` behind `cfg(any(test, feature = "pg_test"))`.
- Re-exports it through the test facade as `am::spire_dml_frontdoor_const_plan_param_list_info`.
- Updates `src/tests/dml_frontdoor.rs` so const-PK primitive invocation tests no longer construct a null `ParamListInfo` in a local unsafe block.

The helper is intentionally scoped to test/pg_test builds. Const-PK plans do not read executor parameters; parameterized plans still use the real executor-provided `ParamListInfo` path.

## Unsafe Count Impact

- `src/tests/dml_frontdoor.rs`: `4 -> 3`
- Current `src/` total: `1348`

See `artifacts/unsafe-counts-before-after.log`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check HEAD~1..HEAD` passed. See `artifacts/git-diff-check.log`.
- Current generated ledger covers all `1348` current `src` unsafe rows. See `artifacts/unsafe-ledger-after.jsonl` and `artifacts/unsafe-ledger-check.log`.

Runtime DML pg tests remain blocked by the same PostgreSQL symbol-linkage issue captured in packet 329.

## Reviewer Focus

- Confirm the null `ParamListInfo` helper is acceptably limited to const-PK test invocation and cannot mask runtime parameter handling.
