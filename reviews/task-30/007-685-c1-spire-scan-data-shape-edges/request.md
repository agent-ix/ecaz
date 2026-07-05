# Review Request: SPIRE scan data-shape edge coverage

- coder: coder1
- date: 2026-05-14
- code commit: 4f94a2c0 `Cover SPIRE scan data shape edges`
- topic: SPIRE phase 12c.14 scan data-shape coverage

## Scope

This slice adds two focused SPIRE scan fixtures for data-shape edge cases without expanding any large test file.

Changed file:

- `src/tests/scan.rs`

## What Changed

Added small top-k helper functions in `scan.rs` and two pg_test fixtures:

- `test_ec_spire_single_row_corpus_scan_returns_only_row`
  - builds a one-row SPIRE corpus
  - runs a top-k scan with `LIMIT 10`
  - compares the SPIRE result set to brute-force `ecvector_negative_query_inner_product`
  - pins the score for the exact one-row case

- `test_ec_spire_duplicate_vector_corpus_scan_matches_exact_set`
  - builds an all-duplicate-vector SPIRE corpus
  - runs a top-k scan with `LIMIT 10`
  - compares the SPIRE result set to the brute-force exact result set
  - asserts all returned scores are identical

These address the 12c.14.a single-row corpus and 12c.14.b duplicate-vector corpus gaps at the local SPIRE scan layer.

## Test File Size Discipline

The touched test file is now 1108 lines:

```text
1108 src/tests/scan.rs
```

The new helpers are intended to keep follow-on scan edge fixtures from repeating top-k query boilerplate. No large test file was expanded past the 2500-line target.

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/tests/scan.rs
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_single_row_corpus_scan_returns_only_row --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`, but exited successfully.

I did not run the pg_test binary. Earlier runtime attempts in this branch still hit the local PostgreSQL backend symbol boundary before executing tests; this slice was validated with the narrow compile-only target.

## Review Focus

Please check whether the 12c.14 data-shape gap should require these same cases through distributed `EcSpireDistributedScan`/remote tuple payload as well, or whether local SPIRE scan coverage is the right first layer for the single-row and duplicate-vector corpus contracts.
