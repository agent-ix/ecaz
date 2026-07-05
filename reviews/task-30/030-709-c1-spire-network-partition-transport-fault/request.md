# Review Request: SPIRE 12c Network Partition Transport Fault

- agent: coder1
- date: 2026-05-14
- code commit: `2d91a9f4039b4ec3ad5ce4e4e0617cc45d2631f9`
- task rows: `12c.2.f`, partial support for `12c.13.a`

## Summary

Adds live transport-summary coverage for the Stage E
`simulated_network_partition` row by pairing one unreachable remote endpoint
with one ready loopback remote.

The test stays in `remote_search/transport_faults.rs`, now 350 lines.

## Changes

- Added `test_ec_spire_prod_transport_network_partition_matrix_actions`.
- The strict-mode summary asserts:
  - two dispatches sent
  - one ready remote and one failed remote
  - `first_transport_failure_category = connect_failed`
  - `next_executor_step = production_transport_adapter`
  - `status = remote_transport_failed`
- The degraded-mode summary asserts:
  - one ready remote remains
  - the unreachable remote is skipped and reported
  - `first_degraded_skip_category = connect_failed`
  - `status = degraded_ready`
- Updated the `12c.2.f` tracker bullets to reflect this live fixture.
- Corrected the existing `remote_oom` strict summary assertion from the
  non-canonical string `production_transport_failed` to the production status
  vocabulary string `remote_transport_failed`.

## Validation

- `cargo fmt --check`
  - Passed.
  - Existing rustfmt warnings about unstable `imports_granularity` /
    `group_imports` options were emitted.
- `git diff --check -- src/tests/remote_search/transport_faults.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Passed.
- `cargo test --no-default-features --features pg18 test_ec_spire_prod_transport_network_partition_matrix_actions --no-run`
  - Passed compile-only.
  - Existing unused import warning in `src/am/mod.rs` was emitted.
- I did not rerun `cargo pgrx test pg18` for this slice because the immediately
  preceding focused pgrx attempt failed before test execution with the existing
  local `undefined symbol: BufferBlocks` harness issue.

## Review Focus

- Confirm the unreachable-endpoint fixture is acceptable live coverage for
  `simulated_network_partition`.
- Confirm strict and degraded summary assertions match the Stage E matrix row.
- Confirm the `remote_oom` strict status correction is the intended vocabulary.
