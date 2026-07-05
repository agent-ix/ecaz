# Review Request: SPIRE DML Schema Drift Split

agent: coder1
date: 2026-05-14
code commit: `2f3c39388e3159b2897e1fb596e4c3d6d0483383`
task rows: closes `12c.9.f`

## Summary

This checkpoint uses the updated Phase 12c tracker row directly:
`12c.9.f` now has split UPDATE/DELETE schema-drift coverage for
coordinator-only, remote-only, and both-side mismatch variants.

The old combined coordinator-only test was removed from
`dml_frontdoor.rs`, and the replacement lives in
`src/tests/dml_schema_drift.rs` so the DML frontdoor test file stays
under the file-size target.

## Changes

- Added `test_ec_spire_update_schema_drift_variants_sql`.
- Added `test_ec_spire_delete_schema_drift_variants_sql`.
- Each test drives:
  - coordinator-only drift, asserting `coordinator side drifted`;
  - remote-only drift, asserting `remote side drifted`;
  - both-side mismatch, asserting `coordinator and remote schema
    fingerprints differ`.
- Each variant also asserts the remote row is untouched, no prepared
  xact is opened, and the placement row remains because the guard fails
  before remote dispatch.
- Moved the coverage out of `dml_frontdoor.rs`:
  - `src/tests/dml_frontdoor.rs`: 2404 lines.
  - `src/tests/dml_schema_drift.rs`: 334 lines.

## Validation

- `cargo fmt --check`
  - Passed, with existing stable-rustfmt warnings about unstable import
    grouping options.
- `git diff --check -- src/tests/dml_frontdoor.rs src/tests/dml_schema_drift.rs src/tests/mod.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Passed.
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_update_schema_drift_variants_sql --no-run`
  - Passed.
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_delete_schema_drift_variants_sql --no-run`
  - Passed.
- `cargo pgrx test pg18 test_ec_spire_update_schema_drift_variants_sql`
  - Failed before test execution with the existing loader error:
    `undefined symbol: pg_re_throw`.

## Review Focus

- Please check whether the both-side variant is the right observable
  contract for 12c.9.f: it applies independent coordinator/remote DDL,
  refreshes the descriptor so each side's current fingerprint is known,
  then pins the cross-side mismatch error.
- Please check the no-dispatch assertions: remote row unchanged, zero
  prepared xacts, placement retained.
- Please check the file split and include location against the
  post-split test layout.
