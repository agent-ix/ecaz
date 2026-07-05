# Review Request: SPIRE 12c DML Frontdoor Tracker Reconciliation

- agent: coder1
- date: 2026-05-14
- code commit: `b38321e955f5e6f7152822de98ceb2f259653ef6`
- task rows: closes `12c.9.a`, `12c.9.b`, `12c.9.c`, `12c.9.d`

## Summary

Tracker-only reconciliation against the updated split Phase 12c task file.

The relevant DML-frontdoor tests already exist in the split files, but the
tracker still had the rows unchecked.

## Evidence

- `12c.9.a`: `src/tests/dml_frontdoor_select.rs`
  - `test_ec_spire_dml_frontdoor_non_pk_select_passes_through_sql`
  - Builds a SPIRE-fronted table, runs a non-PK predicate SELECT, asserts the
    EXPLAIN plan is an ordinary PostgreSQL scan and not
    `Custom Scan (EcSpireDistributedScan)`, then asserts returned rows match
    the predicate.
- `12c.9.b`: `src/tests/dml_frontdoor_select.rs`
  - `test_ec_spire_dml_frontdoor_composite_pk_rejected_sql`
  - Defines a composite primary key, creates the ec_spire index, and asserts
    relation context status `unsupported_pk_shape` with
    `ec_spire_distributed_table=false`.
- `12c.9.c`: `src/tests/dml_frontdoor_select.rs`
  - `test_ec_spire_dml_frontdoor_float_pk_rejected_sql`
  - Covers both `float4` and `float8` primary keys and asserts
    `unsupported_pk_shape`.
- `12c.9.d`: `src/tests/dml_frontdoor.rs`
  - `test_ec_spire_dml_frontdoor_rejects_pk_predicate_edge_shapes`
  - Includes the `numeric_outside_int8` classifier case and asserts
    `false|unsupported_pk_predicate|unsupported_shape`.

## Changes

- Checked the corresponding rows in
  `plan/tasks/task30-phase12c-spire-test-coverage.md`.
- No test code changed in this checkpoint.

## Validation

- `git diff --check -- plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Passed.
- No compile or runtime test was run for this tracker-only checkpoint; the
  request points to existing tests only.

## Review Focus

- Confirm these four rows should be marked closed from existing split-test
  coverage.
- Confirm `12c.9.d` is satisfied by classifier-time rejection for
  `numeric_outside_int8`, even though the same test also contains a separate
  prepared-parameter runtime failure assertion.
