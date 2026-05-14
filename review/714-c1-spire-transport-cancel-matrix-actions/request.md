# Review Request: SPIRE 12c Transport Cancel Matrix Actions

- agent: coder1
- date: 2026-05-14
- code commit: `f716a33e55ef0d603df0625a0cdd80c15c77977b`
- task rows: partial support for `12c.13.a`

## Summary

Extends the Stage E transport fault coverage with strict/degraded executor
action assertions for two rows that already had live single-node probes:
remote backend termination and remote query cancellation.

The test stays in `remote_search/transport_faults.rs`, now 507 lines.

## Changes

- Added `test_ec_spire_prod_transport_backend_terminated_matrix_actions`.
- Added `test_ec_spire_prod_transport_query_cancel_matrix_actions`.
- Each fixture pairs the failing loopback remote with a ready loopback remote.
- Strict mode asserts both dispatches are attempted, one fails, one remains
  ready, no degraded skip is recorded, and the summary fails closed with
  `remote_transport_failed`.
- Degraded mode asserts the failing node is skipped/reported, the ready node
  remains usable, and the summary continues as `degraded_ready`.
- Failure categories are pinned to:
  - `remote_backend_terminated`
  - `remote_query_cancelled`

## Validation

- `cargo fmt --check`
  - Passed.
  - Existing rustfmt warnings about unstable `imports_granularity` /
    `group_imports` options were emitted.
- `git diff --check -- src/tests/remote_search/transport_faults.rs`
  - Passed.
- `cargo test --no-default-features --features pg18 test_ec_spire_prod_transport_ --no-run`
  - Passed compile-only.
  - Existing unused import warning in `src/am/mod.rs` was emitted.
- I did not rerun `cargo pgrx test pg18` for this slice because the local
  pgrx test harness was just confirmed to fail before test execution with the
  existing `undefined symbol: BufferBlocks` loader issue.

## Review Focus

- Confirm these two strict/degraded summary fixtures correctly advance the
  broad `12c.13.a` matrix-action row.
- Confirm the fail-closed vs skip-and-report expectations match the Stage E
  transport fault matrix.
- Confirm no tracker checkbox should be flipped yet because `12c.13.a` remains
  a multi-row umbrella until every matrix row has executor-action assertions.
