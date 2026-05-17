# Review Request: SPIRE 12c Remote Timeout Matrix Actions

- agent: coder1
- date: 2026-05-14
- code commit: `2c547a43a75347cf49b4c187ffe62ca7a61b2657`
- task rows: partial `12c.13.a`

## Summary

Adds strict/degraded Stage E matrix action assertions for the existing live
`remote_statement_timeout` transport fault.

The test stays in `remote_search/transport_faults.rs`, now 403 lines.

## Changes

- Added `test_ec_spire_prod_transport_remote_stmt_timeout_matrix_actions`.
- Strict mode uses one slow remote and one ready remote and asserts:
  - two dispatches sent
  - one ready remote and one failed remote
  - `first_transport_failure_category = remote_statement_timeout`
  - `next_executor_step = production_transport_adapter`
  - `status = remote_transport_failed`
- Degraded mode asserts:
  - the ready remote remains usable
  - the timed-out remote is skipped and reported
  - `first_degraded_skip_category = remote_statement_timeout`
  - `status = degraded_ready`

## Validation

- `cargo fmt --check`
  - Passed.
  - Existing rustfmt warnings about unstable `imports_granularity` /
    `group_imports` options were emitted.
- `git diff --check -- src/tests/remote_search/transport_faults.rs`
  - Passed.
- `cargo test --no-default-features --features pg18 test_ec_spire_prod_transport_remote_stmt_timeout_matrix_actions --no-run`
  - Passed compile-only.
  - Existing unused import warning in `src/am/mod.rs` was emitted.
- I did not rerun `cargo pgrx test pg18` for this slice because the local
  pgrx test harness is currently failing before test execution with the
  existing `undefined symbol: BufferBlocks` issue.

## Review Focus

- Confirm this is valid incremental coverage toward `12c.13.a`.
- Confirm strict/degraded summary expectations match the fault matrix row for
  `remote_statement_timeout`.
