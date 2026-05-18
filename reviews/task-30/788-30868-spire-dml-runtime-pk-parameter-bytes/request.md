# Review Request: SPIRE DML Runtime PK Parameter Bytes

## Scope

This packet adds runtime bound-parameter conversion for typed DML primitive
plans. A future DML CustomScan executor can now turn either a constant bigint
PK argument or a bound `$n` bigint/integer PK parameter into the ADR-069
`pk_value bytea` primitive argument.

Code commit: `aebdf0c5dd0648fa4be0f85a36c5e2ca9a1d6f11`

Changes:

- Adds `dml_frontdoor_primitive_plan_pk_value_bytes(...)`, taking a primitive
  plan plus PostgreSQL `ParamListInfo`.
- Supports `INT2OID`, `INT4OID`, and `INT8OID` bound parameters by coercing to
  bigint before using the existing `int8send`-compatible encoder.
- Fails closed for missing parameter lists, out-of-range parameter ids, NULL
  parameter values, fetch failures, and unsupported runtime parameter types.
- Preserves the const-only helper from packet 30867 for call sites that need a
  pre-runtime guard.
- Extends PG18 coverage to allocate a bound `ParamListInfo`, encode `$1 = -7`,
  and reject a NULL bound parameter.
- Updates the Phase 11 task file with the 30868 milestone.

## Validation

- `cargo test dml_frontdoor --lib`
  - 22 passed, 0 failed, 1648 filtered out.
- `cargo fmt --check`
  - Passed with the existing stable-rustfmt warnings about unstable import
    options.
- `git diff --check`
  - Passed.

Artifacts are recorded in `artifacts/manifest.md`.

## Review Focus

1. Confirm direct `ParamListInfo` access and `paramFetch` handling are safe for
   the upcoming executor call site.
2. Confirm runtime integer parameter coercion matches the classifier contract.
3. Confirm NULL/type/count failures are the right fail-closed boundary before
   invoking coordinator DML primitives.
