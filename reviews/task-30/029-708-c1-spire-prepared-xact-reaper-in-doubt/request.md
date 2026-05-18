# Review Request: SPIRE 12c Prepared-Xact Reaper In-Doubt Window

- agent: coder1
- date: 2026-05-14
- code commit: `fa54ac93b6f0e5b135c26e324bee7aecf14cd31c`
- task rows: `12c.5.a`

## Summary

Adds a focused PG fixture for the coordinator-crash-mid-2PC reaper window:
`prepare_acked` with a dead coordinator XID is safe to roll back, while
`commit_local` with a dead coordinator XID must be preserved for operator
resolution.

The test stays in the split `remote_search/catalog_cleanup_policy.rs` file,
which is now 1,098 lines.

## Changes

- Added `test_ec_spire_reaper_prepare_acked_vs_commit_local`.
- The fixture creates two real remote prepared transactions on the loopback
  remote under node 34:
  - one matching an intent row in `prepare_acked`
  - one matching an intent row in `commit_local`
- The fixture invokes `ec_spire_reap_orphaned_remote_prepared_xacts(34)` and
  asserts:
  - the `prepare_acked` row reports `prepare_acked:rolled_back:false`
  - the `commit_local` row reports `commit_local:skipped_commit_local:false`
  - the `prepare_acked` remote prepared transaction is gone
  - the `commit_local` remote prepared transaction is still present
  - the local intent states become `rollback_local` and remain `commit_local`
- Updated the atomic tracker bullets for `12c.5.a`.

## Validation

- `cargo fmt --check`
  - Passed.
  - Existing rustfmt warnings about unstable `imports_granularity` /
    `group_imports` options were emitted.
- `git diff --check -- src/tests/remote_search/catalog_cleanup_policy.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Passed.
- `cargo test --no-default-features --features pg18 test_ec_spire_reaper_prepare_acked_vs_commit_local --no-run`
  - Passed compile-only.
  - Existing unused import warning in `src/am/mod.rs` was emitted.
- `cargo pgrx test pg18 test_ec_spire_reaper_prepare_acked_vs_commit_local`
  - Failed before executing the test binary with the existing local harness
    symbol issue:
    `undefined symbol: BufferBlocks`.
  - This matches the already-known pgrx runtime boundary in this branch; no
    fixture assertion output was produced.

## Review Focus

- Confirm that the fixture is sufficient coverage for `12c.5.a`.
- Confirm that preserving `commit_local` while rolling back `prepare_acked`
  matches the intended operator escalation contract.
- Confirm the tracker update is scoped correctly to `12c.5.a` only.
