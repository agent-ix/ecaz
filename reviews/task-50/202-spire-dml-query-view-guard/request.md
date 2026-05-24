# Task 50 Review Request: SPIRE DML Query View Guard

## Summary

This slice replaces several SPIRE DML frontdoor raw `Query` helper paths with a `DmlFrontdoorQueryView`.

The view has one unsafe constructor for the active PostgreSQL planner callback lifetime, then exposes safe read-only methods for:

- operation lookup
- target range-table index lookup
- range-table relation OID lookup
- join/subquery/returning shape checks
- jointree predicate access

It also removes the unused catalog-context classification helper pair and updates the local DML frontdoor unit test to use the view method instead of the removed free subquery helper.

## Unsafe Burn Down

- `src/am/ec_spire/dml_frontdoor/mod.rs` direct `unsafe { ... }` blocks: `43 -> 42`
- `src/` total `unsafe` token count after this slice: `2623`
- This is a structural cleanup: the remaining query raw pointer entry points still have unsafe signatures, but internal code now passes a typed query view instead of repeatedly re-borrowing raw `Query` pointers through safe helpers.

## Code Commit

- `0b908254261ed3525b6b57ded2c20bbf87f7561b` Add SPIRE DML query view guard

## Validation

- `rustfmt --edition 2021 --check src/am/ec_spire/dml_frontdoor/mod.rs src/am/ec_spire/dml_frontdoor/tests.rs`
  - log: `artifacts/rustfmt-dml-frontdoor.log`
  - passed with existing stable-rustfmt warnings for unstable import grouping options
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - log: `artifacts/cargo-check-pg18-bench.log`
  - passed with existing unused-import warning in `src/am/mod.rs`
- `cargo test --lib dml_frontdoor --no-default-features --features pg18,pg_test --no-run`
  - log: `artifacts/cargo-test-dml-frontdoor-no-run.log`
  - passed with existing Hadamard test-helper dead-code warnings
- `git diff --check`
  - log: `artifacts/git-diff-check.log`
  - passed
- Direct unsafe scan:
  - log: `artifacts/dml-frontdoor-direct-unsafe-scan.log`

