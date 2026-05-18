# Review Request: SPIRE numerical edge scan coverage

- coder: coder1
- date: 2026-05-14
- code commit: 87f5fca6 `Cover SPIRE numerical edge scans`
- topic: SPIRE phase 12c.14.c numerical-extreme vector handling

## Scope

This slice adds focused data-shape coverage for numerical edge vectors without growing an already-large test file.

Changed file:

- `src/tests/scan.rs`

## What Changed

Added `test_ec_spire_numerical_extreme_vector_scan_matches_exact_set`.

The fixture inserts:

- subnormal `real` components
- near-`f32::MAX` finite components
- a normal finite comparison row

It builds a small SPIRE index, forces the indexed scan path, compares SPIRE top-k IDs against exact heap scoring, asserts the near-max row is the best match for the tiny query, and checks all returned scores remain finite.

Added `test_ec_spire_non_finite_vector_inserts_rejected`, which creates an indexed SPIRE table and verifies `NaN`, `+Infinity`, and `-Infinity` vector insert attempts are rejected with an explicit finite-value error.

## Test File Size Discipline

The touched test file remains below the 2500-line target:

```text
1238 src/tests/scan.rs
```

No new test file was needed; this keeps the 12c.14 scan-shape fixtures grouped with the existing single-row and duplicate-vector scan tests.

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/tests/scan.rs
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_numerical_extreme_vector_scan_matches_exact_set --no-run
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_non_finite_vector_inserts_rejected --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`, but exited successfully.

I did not run the pg_test binaries. Earlier runtime attempts in this branch still hit the local PostgreSQL backend symbol boundary before executing tests; this slice was validated with narrow compile-only targets.

## Review Focus

Please check whether this is sufficient for 12c.14.c, or whether reviewers want a follow-up runtime-only fixture that reaches lower-level SPIRE AM insertion after bypassing `encode_to_ecvector`'s finite-value validation.
