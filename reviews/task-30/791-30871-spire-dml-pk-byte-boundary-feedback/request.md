# Review Request: SPIRE DML PK Byte Boundary Feedback

## Scope

Code commit: `d8a00e18a9127a037252219e5fd5a1392e43d25a`

This packet folds reviewer feedback from 30868 into the DML CustomScan executor boundary:

- Documents that `paramFetch` may return a pointer into function-local `ParamExternData` workspace and that the parameter is consumed before returning.
- Regression-locks `i64::MAX` and `i64::MIN` through the const primitive-plan PK bytea path.
- Regression-locks `i64::MAX` and `i64::MIN` through the runtime bound-parameter PK bytea path and primitive-invocation builder.
- Updates the Phase 11 task file with the feedback follow-up milestone.

This remains a boundary/coverage packet. It does not enable planner rewrite or DML dispatch.

## Validation

- `cargo test dml_frontdoor --lib`
  - `23 passed; 0 failed; 0 ignored; 1648 filtered out`
  - artifact: `artifacts/cargo-test-dml-frontdoor-lib.log`
- `cargo fmt --check`
  - passed
  - emits the known stable-rustfmt warnings for unstable import grouping options
  - artifact: `artifacts/cargo-fmt-check.log`
- `git diff --check HEAD^ HEAD -- src/am/ec_spire/dml_frontdoor.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`
  - passed
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

1. Confirm the `paramFetch` workspace comment captures the lifetime constraint precisely enough for future executor refactors.
2. Confirm the `i64::MIN`/`i64::MAX` coverage exercises both const and runtime parameter PK bytea paths.
3. Confirm the packet remains limited to the 30868 feedback follow-up and does not change DML frontdoor behavior.
