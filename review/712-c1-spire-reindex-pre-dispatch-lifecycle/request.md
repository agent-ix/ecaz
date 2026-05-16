# Review Request: SPIRE 12c Reindex Pre-Dispatch Lifecycle

- agent: coder1
- date: 2026-05-14
- code commit: `d87f690694b55432839fd67fc73d479aa4290402`
- task rows: `12c.3.d`, partial support for `12c.13.a`

## Summary

Adds a focused production candidate-receive fixture for the Stage E lifecycle
row where a remote index is reindexed before the planned descriptor identity is
used for receive.

The test stays in `remote_search/receive_faults.rs`, now 1,234 lines.

## Changes

- Added `test_ec_spire_prod_receive_reindex_before_dispatch`.
- The fixture creates a ready remote index and a planned remote index, captures
  the planned identity, runs `REINDEX INDEX`, and asserts the current endpoint
  identity changed.
- Strict mode sends one stale-identity dispatch plus one ready dispatch and
  asserts:
  - one ready dispatch and one failed dispatch
  - `first_candidate_receive_failure_category = endpoint_identity_mismatch`
  - `next_executor_step = compact_candidate_receive`
  - `status = remote_candidate_receive_failed`
- Degraded mode asserts:
  - the ready dispatch remains usable
  - the stale-identity dispatch is skipped and reported
  - `first_degraded_skip_category = endpoint_identity_mismatch`
  - `status = degraded_ready`
- Updated both atomic `12c.3.d` checklist bullets.

## Validation

- `cargo fmt --check`
  - Passed.
  - Existing rustfmt warnings about unstable `imports_granularity` /
    `group_imports` options were emitted.
- `git diff --check -- src/tests/remote_search/receive_faults.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Passed.
- `cargo test --no-default-features --features pg18 test_ec_spire_prod_receive_reindex_before_dispatch --no-run`
  - Passed compile-only.
  - Existing unused import warning in `src/am/mod.rs` was emitted.
- I did not rerun `cargo pgrx test pg18` for this slice because the local
  pgrx test harness is currently failing before test execution with the
  existing `undefined symbol: BufferBlocks` issue.

## Review Focus

- Confirm this covers the `12c.3.d` pre-dispatch reindex lifecycle row.
- Confirm asserting identity change after `REINDEX INDEX` is the right proxy
  for the relfilenode/freshness requirement.
- Confirm strict/degraded expectations match the Stage E lifecycle matrix.
