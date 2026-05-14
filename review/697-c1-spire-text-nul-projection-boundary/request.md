# Review Request: SPIRE text NUL projection boundary coverage

- coder: coder1
- date: 2026-05-14
- code commit: 990712a4 `Cover SPIRE text NUL projection boundary`
- topic: SPIRE phase 12c.14.d text-with-NUL-byte projection boundary

## Scope

This slice covers the embedded-NUL `text` projection edge by pinning the
explicit PostgreSQL unsupported boundary before CustomScan or tuple transport
can observe such a value.

Changed files:

- `src/tests/data_shape.rs`
- `src/tests/mod.rs`

## What Changed

Added `test_ec_spire_text_projection_nul_byte_rejected_sql`.

The fixture creates a SPIRE-indexed table with a projected `text` column, then
attempts to insert a value produced from bytes containing `0x00` via
`convert_from(decode(...), 'UTF8')`.

The test asserts:

- PostgreSQL rejects the value with an explicit NUL/invalid-encoding error
- no heap row is inserted

This documents 12c.14.d as unsupported at the PostgreSQL `text` type boundary,
rather than silently depending on CustomScan tuple projection behavior that can
never receive a valid embedded-NUL `text` datum.

## Test File Size Discipline

This starts a dedicated data-shape include instead of adding more edge-case
tests to the broader scan or CustomScan files:

```text
47 src/tests/data_shape.rs
1329 src/tests/scan.rs
1448 src/tests/custom_scan.rs
```

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/tests/mod.rs src/tests/data_shape.rs
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_text_projection_nul_byte_rejected_sql --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`, but exited
successfully.

I did not run the pg_test binary. Earlier runtime attempts in this branch still
hit the local PostgreSQL backend symbol boundary before executing tests; this
slice was validated with the narrow compile-only target.

## Review Focus

Please check whether this explicit unsupported boundary is sufficient for
12c.14.d, or whether reviewers want a bytea-based follow-up fixture to test
SPIRE tuple projection over binary payloads that can legally contain `0x00`.
