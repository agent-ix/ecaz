# Review Request: SPIRE DML Primitive PK Bytes

## Scope

This packet adds the bytea conversion boundary for typed DML primitive plans.
Constant bigint PK arguments can now become the ADR-069 `pk_value bytea`
primitive argument, while parameterized plans remain explicitly blocked until
the executor expression-context slice evaluates `$n` values at runtime.

Code commit: `c7baa829e4129979f042ca01d0d3ee632832ce22`

Changes:

- Adds `dml_frontdoor_primitive_plan_const_pk_value_bytes(...)`.
- Reuses the existing PostgreSQL-compatible bigint encoder from packet 30864.
- Returns a fail-closed error for `ParamBigint`, naming the required executor
  parameter evaluation step.
- Re-exports the helper through the `ec_spire` and `am` module boundaries.
- Extends the primitive-plan PG18 coverage to assert constant byte output and
  parameter-plan blocking.
- Updates the Phase 11 task file with the 30867 milestone.

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

1. Confirm const PK byte conversion is the right narrow executor-boundary helper.
2. Confirm parameter PK plans fail closed until runtime parameter evaluation is
   wired through the executor expression context.
3. Confirm this packet does not imply planner rewrite or remote DML dispatch is
   enabled.
