# Review Request: SPIRE 12c In-Flight Lifecycle Faults

- agent: coder1
- date: 2026-05-14
- code commit: `f8d5aa8da439183ec1752304e0c16fe2918ec774`
- task rows: `12c.3.a`, `12c.3.c`, partial support for `12c.13.a`

## Summary

Adds focused production candidate-receive fixtures for the Stage E lifecycle
rows where a remote index is dropped or reindexed after request construction
but before receive.

This slice is anchored on the updated split tracker at
`plan/tasks/task30-phase12c-spire-test-coverage.md`. The test stays in
`remote_search/receive_faults.rs`, now 1,474 lines.

## Changes

- Added `test_ec_spire_prod_receive_drop_index_in_flight`.
- Added `test_ec_spire_prod_receive_reindex_in_flight`.
- Each fixture builds strict/degraded receive requests while both remote
  indexes exist, applies the remote DDL, then runs the production
  candidate-receive summary against the pre-built requests.
- `DROP INDEX` asserts the planned remote index is absent before receive and
  verifies:
  - strict mode sends both dispatches, records one ready dispatch and one
    `remote_index_unavailable` failure, then fails closed with
    `remote_candidate_receive_failed`
  - degraded mode skips/reports the dropped index and keeps the ready dispatch
    usable with `degraded_ready`
- `REINDEX INDEX` asserts the endpoint identity changes after request
  construction and verifies:
  - strict mode records one ready dispatch and one
    `endpoint_identity_mismatch` failure, then fails closed with
    `remote_candidate_receive_failed`
  - degraded mode skips/reports the stale identity and keeps the ready dispatch
    usable with `degraded_ready`
- Updated the atomic tracker bullets for `12c.3.a` and `12c.3.c`.

## Validation

- `cargo fmt --check`
  - Passed.
  - Existing rustfmt warnings about unstable `imports_granularity` /
    `group_imports` options were emitted.
- `git diff --check -- src/tests/remote_search/receive_faults.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Passed.
- `cargo test --no-default-features --features pg18 test_ec_spire_prod_receive_ --no-run`
  - Passed compile-only.
  - Existing unused import warning in `src/am/mod.rs` was emitted.
- `cargo pgrx test pg18 test_ec_spire_prod_receive_drop_index_in_flight`
  - Did not reach test execution.
  - Failed at local harness load time with the existing
    `undefined symbol: BufferBlocks` issue.

## Review Focus

- Confirm the request-built-before-DDL timing is sufficient coverage for the
  split tracker's `12c.3.a` and `12c.3.c` in-flight lifecycle rows.
- Confirm strict/degraded status and skip-count expectations match the Stage E
  lifecycle matrix.
- Confirm the updated task checkboxes are scoped correctly to the two
  in-flight lifecycle rows.
