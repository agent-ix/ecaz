# Review Request: SPIRE 12c Drop-Index Pre-Dispatch Lifecycle

- agent: coder1
- date: 2026-05-14
- code commit: `140f2ea1242f953762134e83e0ef79ed284afda4`
- task rows: `12c.3.b`, partial support for `12c.13.a`

## Summary

Adds a focused production candidate-receive fixture for the Stage E lifecycle
row where a remote index is dropped before receive/dispatch uses the planned
remote index identity.

The new test stays in `remote_search/receive_faults.rs`, which is now 1,099
lines.

## Changes

- Added `test_ec_spire_prod_receive_drop_remote_index_before_dispatch`.
- The fixture creates a ready remote index and a second planned remote index,
  captures both identities, then drops the planned index before candidate
  receive.
- Strict mode asserts:
  - two candidate receive dispatches are sent
  - one ready dispatch and one failed dispatch
  - `first_candidate_receive_failure_category = remote_index_unavailable`
  - `next_executor_step = compact_candidate_receive`
  - `status = remote_candidate_receive_failed`
- Degraded mode asserts:
  - the ready dispatch remains usable
  - the dropped-index dispatch is skipped and reported
  - `first_degraded_skip_category = remote_index_unavailable`
  - `status = degraded_ready`
- Updated only the first `12c.3.b` atomic checklist item. The separate
  descriptor-refresh/no-remote-SQL instrumentation bullet remains open.

## Validation

- `cargo fmt --check`
  - Passed.
  - Existing rustfmt warnings about unstable `imports_granularity` /
    `group_imports` options were emitted.
- `git diff --check -- src/tests/remote_search/receive_faults.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Passed.
- `cargo test --no-default-features --features pg18 test_ec_spire_prod_receive_drop_remote_index_before_dispatch --no-run`
  - Passed compile-only.
  - Existing unused import warning in `src/am/mod.rs` was emitted.
- I did not rerun `cargo pgrx test pg18` for this slice because the local
  pgrx test harness is currently failing before test execution with the
  existing `undefined symbol: BufferBlocks` issue.

## Review Focus

- Confirm this is valid coverage for the action/category part of `12c.3.b`.
- Confirm keeping the descriptor-refresh/no-remote-SQL bullet open is correct.
- Confirm strict/degraded summary expectations match the Stage E lifecycle
  matrix row.
