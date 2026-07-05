# Review Request: SPIRE 12c DELETE Idempotency No Redispatch

- agent: coder1
- date: 2026-05-14
- code commit: `0eb07b87ceac13536ede814d96ab9b671020e40a`
- task rows: closes `12c.9.e`

## Summary

Adds a remote-placement idempotent DELETE fixture for the updated Phase 12c
tracker row `12c.9.e`.

The existing coverage already pinned direct remote helper idempotency and
coordinator missing/stale local placement shapes. This slice adds the missing
coordinator-to-remote case: after the first coordinator delete prepares remote
DML and removes the placement row, a second same-PK coordinator delete must be a
no-op shape and must not dispatch another remote delete.

The test stays in `src/tests/dml_frontdoor_delete.rs`, which is now 199 lines.

## Changes

- Added `test_ec_spire_coord_remote_delete_idem_no_redispatch_sql`.
- Sets up a loopback remote heap/index and matching coordinator heap/index.
- Registers an active remote descriptor and one remote placement.
- First coordinator delete asserts:
  - `remote_delete_sent=true`
  - `remote_prepared=true`
  - `remote_deleted_count=1`
  - `placement_deleted=true`
  - status `remote_delete_prepared_pending_local_commit`
- Second same-PK coordinator delete asserts:
  - no placement route remains
  - `remote_delete_sent=false`
  - `remote_prepared=false`
  - `remote_deleted_count=0`
  - status `delete_not_found_noop`
  - prepared-xact count is unchanged from the first delete
- Updated `plan/tasks/task30-phase12c-spire-test-coverage.md` for `12c.9.e`.

## Validation

- `cargo fmt --check`
  - Passed.
  - Existing rustfmt warnings about unstable `imports_granularity` /
    `group_imports` options were emitted.
- `git diff --check -- src/tests/dml_frontdoor_delete.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Passed.
- `cargo test --no-default-features --features pg18 test_ec_spire_coord_remote_delete_idem_no_redispatch_sql --no-run`
  - Passed compile-only.
  - Existing unused import warning in `src/am/mod.rs` was emitted.
- `cargo pgrx test pg18 test_ec_spire_coord_remote_delete_idem_no_redispatch_sql`
  - Blocked before test execution by loader error:
    `undefined symbol: pg_re_throw`.

## Review Focus

- Confirm this closes `12c.9.e` even though the current helper result shape
  does not expose a literal `accepted` boolean; the second delete returns a
  successful no-op row with `remote_deleted_count=0`.
- Confirm unchanged prepared-xact count is the right no-redispatch assertion for
  this pre-local-commit helper path.
- Confirm the fixture belongs in the small split
  `src/tests/dml_frontdoor_delete.rs` file rather than the older large
  `dml_frontdoor.rs`.
