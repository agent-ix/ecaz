# Review Request: SPIRE DML composite-PK rejection coverage

- coder: coder1
- date: 2026-05-14
- code commit: 3b4d34b3 `Cover DML composite PK rejection`
- topic: SPIRE phase 12c.9.b composite-PK rejection

## Scope

This slice adds focused coverage for the ADR-069 v1 single-bigint-PK boundary.

Changed file:

- `src/tests/dml_frontdoor_select.rs`

## What Changed

Added `test_ec_spire_dml_frontdoor_composite_pk_rejected_sql`.

The fixture creates a SPIRE-fronted table with a composite primary key
`(tenant_id, id)`, then checks the DML frontdoor relation-context registration
surface.

The test asserts:

- `status = 'unsupported_pk_shape'`
- `next_step` asks for one bigint primary-key column
- `ec_spire_distributed_table = false`
- `pk_column` is NULL
- `pk_type` is NULL

## Test File Size Discipline

The focused DML SELECT include remains small:

```text
115 src/tests/dml_frontdoor_select.rs
```

This keeps the new 12c.9 coverage out of `src/tests/dml_frontdoor.rs`, which
was already above the 2500-line target before this work.

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/tests/dml_frontdoor_select.rs
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_dml_frontdoor_composite_pk_rejected_sql --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`, but exited
successfully.

I did not run the pg_test binary. Earlier runtime attempts in this branch still
hit the local PostgreSQL backend symbol boundary before executing tests; this
slice was validated with the narrow compile-only target.

## Review Focus

Please check whether the relation-context surface is the correct "registration"
surface for 12c.9.b, or whether reviewers want a follow-up that also drives a
planner-hook query over the composite-PK relation and asserts the fail-closed
diagnostic path.
