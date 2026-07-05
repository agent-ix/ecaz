# Review Request: SPIRE Remote Heap Score Coverage

- Code commit: `a747cfc1` (`Tighten SPIRE remote heap score coverage`)
- Scope: Phase 12c.6.c sign-convention pin extension.
- File changed: `src/am/ec_spire/coordinator/tests.rs`

## What Changed

- Extended `remote_heap_exact_score_uses_orderby_negative_inner_product`.
- Added a 128-dimensional exact-score case with expected `<#>` convention score `-707264.0`.
- Added rejection assertions for:
  - non-finite source vector component;
  - non-finite query component producing a non-finite score;
  - query/source dimension mismatch with exact error text.

## File-Size Discipline

`src/am/ec_spire/coordinator/tests.rs` is now 1,982 lines, still below the 2,500-line target.

## Validation

- `cargo fmt --check` passed.
- `git diff --check -- src/am/ec_spire/coordinator/tests.rs` passed.
- `cargo test --features "pg18 pg_test" --no-default-features remote_heap_exact_score_uses_orderby_negative_inner_product --no-run` passed.

## Review Focus

1. Confirm the high-dimensional expected value is the right exact inner-product sum for dimensions `1..=128`.
2. Confirm both non-finite paths are worth pinning separately: source-vector rejection and query-driven non-finite score rejection.
