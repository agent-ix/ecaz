# Review Request: SPIRE SELECT FOR UPDATE Stale Read Contract

## Summary

Coder: `coder1`
Topic: `756-c1-spire-select-for-update-stale-read`
Code commit: `bc4ce8c1b95298c56387382b76446b0dc862bcdd`
Date: `2026-05-15`

This checkpoint closes the exact 12c.11.b `SELECT FOR UPDATE` rows in the
updated Phase 12c tracker. It extends the accepted
`test_ec_spire_remote_pk_select_isolation_contract_sql` fixture with a
cursor-based two-session sequence:

- Session A declares a cursor over `SELECT ... FOR UPDATE` after the test
  asserts the plan still contains `Custom Scan (EcSpireDistributedScan)`.
- Session B updates the same remote-owned row and commits.
- Session A fetches from the pre-update cursor and asserts the pre-update
  remote title is surfaced, matching the documented v1 stale-read contract.

The existing test comment still cross-references the `begin_exec.rs:420-428`
recheck contract explaining that v1 remote payload rows are virtual and
cannot be EvalPlanQual re-fetched against coordinator heap identity.

## Files

- `src/tests/remote_search/catalog_cleanup_policy.rs`
- `plan/tasks/task30-phase12c-spire-test-coverage.md`

`src/tests/remote_search/catalog_cleanup_policy.rs` is 1174 lines after this
change, below the 2500-line target.

## Validation

- `cargo fmt --check` passed.
- `git diff --check -- src/tests/remote_search/catalog_cleanup_policy.rs plan/tasks/task30-phase12c-spire-test-coverage.md` passed.
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_remote_pk_select_isolation_contract_sql --no-run` passed.
- `cargo pgrx test pg18 test_ec_spire_remote_pk_select_isolation_contract_sql` failed before test execution with:
  `undefined symbol: pg_re_throw`.

## Review Needs

Please verify that the cursor-based `SELECT ... FOR UPDATE` sequence is the
right exact fixture shape for the three remaining 12c.11.b rows.
