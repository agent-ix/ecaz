# Review Request: SPIRE DML Replacement PK Argument Shape

## Scope

This packet extends the DML front-door replacement decision with the primary-key
predicate argument shape the future DML CustomScan executor needs. Plan
rewriting remains disabled.

Code commit: `c1464cd0a1540cc0cb9b616d52aa6a790e37195c`

Changes:

- Extends `SpireDmlFrontdoorReplacementDecisionRow` with:
  - `pk_value_kind`
  - `pk_value_const`
  - `pk_value_param_id`
- Reuses the existing PK predicate extraction path so classifier support and
  replacement argument shape stay aligned.
- Decodes int2/int4/int8 constants to the canonical bigint value the executor
  will later encode through `int8send(...)::bytea`.
- Carries parameter ids for bigint-compatible parameter predicates.
- Extends `ec_spire_dml_frontdoor_replacement_sql(sql text)` to expose the new
  fields.
- Adds PG18 assertions that supported PK SELECT and non-embedding UPDATE
  replacement decisions report `const_bigint` and the expected `5` value.
- Updates the Phase 11 task file with the 30862 milestone.

## Validation

- `cargo test dml_frontdoor --lib`
  - 18 passed, 0 failed, 1648 filtered out.
- `cargo fmt --check`
  - Passed with the existing stable-rustfmt warnings about unstable import
    options.
- `git diff --check`
  - Passed.

Artifacts are recorded in `artifacts/manifest.md`.

## Review Focus

1. Confirm the PK predicate argument shape is sufficient for the next DML
   CustomScan executor slice to build primitive calls.
2. Confirm constant and parameter handling remain aligned with the existing
   bigint-compatible classifier rules.
3. Confirm the diagnostic SQL shape change is acceptable while plan rewriting
   remains disabled.
