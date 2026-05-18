# Review Request: SPIRE DML float-PK rejection coverage

- coder: coder1
- date: 2026-05-14
- code commit: f1e28029 `Cover DML float PK rejection`
- topic: SPIRE phase 12c.9.c float primary-key rejection

## Scope

This slice adds focused coverage for rejecting non-bigint floating-point primary
keys at the DML frontdoor relation-context boundary.

Changed file:

- `src/tests/dml_frontdoor_select.rs`

## What Changed

Added `test_ec_spire_dml_frontdoor_float_pk_rejected_sql` plus a small helper
that exercises both `float4` and `float8` primary-key tables.

For each type, the fixture creates a SPIRE-indexed table and asserts the DML
frontdoor relation context reports:

- `status = 'unsupported_pk_shape'`
- `next_step` asks for one bigint primary-key column
- `ec_spire_distributed_table = false`
- `pk_type` is NULL

This complements the lower-level classifier coverage for float-like PK
predicate values by pinning the catalog/registration boundary for actual
floating-point primary keys.

## Test File Size Discipline

The focused DML SELECT include remains small:

```text
172 src/tests/dml_frontdoor_select.rs
```

This keeps new 12c.9 coverage out of `src/tests/dml_frontdoor.rs`, which was
already above the 2500-line target before this work.

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/tests/dml_frontdoor_select.rs
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_dml_frontdoor_float_pk_rejected_sql --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`, but exited
successfully.

I did not run the pg_test binary. Earlier runtime attempts in this branch still
hit the local PostgreSQL backend symbol boundary before executing tests; this
slice was validated with the narrow compile-only target.

## Review Focus

Please check whether covering both `float4` and `float8` at relation-context
registration is sufficient for 12c.9.c, or whether reviewers want a follow-up
planner-hook fail-closed assertion for a float-PK SELECT shape as well.
