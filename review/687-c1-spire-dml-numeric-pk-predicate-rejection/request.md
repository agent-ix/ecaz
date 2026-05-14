# Review Request: SPIRE DML numeric PK predicate rejection

- coder: coder1
- date: 2026-05-14
- code commit: 4c6d23c7 `Cover SPIRE DML numeric PK predicate rejection`
- topic: SPIRE phase 12c.9 DML frontdoor predicate tightening

## Scope

This slice adds focused Rust unit coverage for DML frontdoor PK predicate classification without growing the already-large `src/tests/dml_frontdoor.rs` pg_test file.

Changed file:

- `src/am/ec_spire/dml_frontdoor/tests.rs`

## What Changed

Added `query_layer_rejects_float_and_numeric_pk_predicate_values`, which verifies both constant and parameter predicate values of these PostgreSQL types are classified as unsupported:

- `float4`
- `float8`
- `numeric`

This pins the classifier-time rejection path for float/numeric PK predicates, covering part of the 12c.9.c/12c.9.d gap at the Rust query-layer classifier boundary.

## Test File Size Discipline

The touched Rust unit-test file is now 437 lines:

```text
437 src/am/ec_spire/dml_frontdoor/tests.rs
2498 src/am/ec_spire/dml_frontdoor/mod.rs
```

This deliberately avoids adding to `src/tests/dml_frontdoor.rs`, which is already above the 2500-line target.

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/am/ec_spire/dml_frontdoor/tests.rs
cargo test --features "pg18 pg_test" --no-default-features query_layer_rejects_float_and_numeric_pk_predicate_values --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`, but exited successfully.

I did not run the test binary. Earlier runtime attempts in this branch still hit the local PostgreSQL backend symbol boundary before executing tests; this slice was validated with the narrow compile-only target.

## Review Focus

Please check whether this is sufficient classifier-layer coverage for 12c.9.c/12c.9.d, or whether follow-up pg_test coverage should also assert the user-visible registration/DML error shape for float and numeric PKs.
