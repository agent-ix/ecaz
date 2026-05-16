# Review Request: SPIRE 12c Remote OOM Transport Fault

- agent: coder1
- date: 2026-05-14
- code commit: `3d29804975d41c0568e2ad446cbab9d817b2d27a`
- task rows: `12c.2.e` (`remote_oom`), partial support for `12c.13.a`

## Summary

Adds live PG test coverage for the Stage E `remote_oom` transport fault.

The new coverage stays in the existing split `remote_search/transport_faults.rs`
file, which is now 301 lines. No large test file was expanded.

## Changes

- Added `test_ec_spire_prod_transport_remote_oom`, which drives a loopback
  remote query raising SQLSTATE `53200` and asserts the production transport
  probe reports:
  - `status = remote_transport_failed`
  - `failure_category = remote_query_failed`
  - `row_count = 0`
- Added `test_ec_spire_prod_transport_remote_oom_matrix_actions`, which runs
  one failing remote plus one ready remote through the production summary path:
  - strict mode asserts fail-closed via
    `next_executor_step = production_transport_adapter` and
    `status = production_transport_failed`
  - degraded mode asserts the fault is skipped and reported via
    `degraded_skipped_dispatch_count = 1`,
    `first_degraded_skip_category = remote_query_failed`, and
    `status = degraded_ready`
- Updated `plan/tasks/task30-phase12c-spire-test-coverage.md` to mark the
  completed atomic bullets for `12c.2.e`; `12c.13.a` remains open because only
  this one matrix row is covered by this slice.

## Validation

- `cargo fmt --check`
  - Passed.
  - Existing rustfmt warnings about unstable `imports_granularity` /
    `group_imports` options were emitted.
- `git diff --check -- src/tests/remote_search/transport_faults.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Passed.
- `cargo test --no-default-features --features pg18 test_ec_spire_prod_transport_remote_oom --no-run`
  - Passed compile-only.
  - Existing unused import warning in `src/am/mod.rs` was emitted.

## Review Focus

- Confirm the SQLSTATE `53200` loopback fixture is acceptable as the live
  `remote_oom` simulator for `12c.2.e`.
- Confirm strict/degraded summary assertions match the Stage E matrix action
  expected for `remote_oom`.
- Confirm leaving the broader `12c.13.a` row open is correct until the rest of
  the matrix rows have executor assertions.
