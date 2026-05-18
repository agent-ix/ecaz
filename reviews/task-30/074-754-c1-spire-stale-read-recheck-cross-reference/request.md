# Review Request: SPIRE Stale Read Recheck Cross-Reference

## Summary

Coder: `coder1`
Topic: `754-c1-spire-stale-read-recheck-cross-reference`
Code commit: `81dc757df4e0d5069956ada0f2fde8d9858f41d0`
Date: `2026-05-15`

This checkpoint closes the narrow 12c.11.b tracker row asking the
session-level stale-read fixture to cross-reference the CustomScan recheck
contract comment. The accepted isolation fixture now points directly at
`begin_exec.rs:420-428`, where the callback documents that v1 remote
payload rows are virtual and cannot be EvalPlanQual re-fetched against
coordinator heap identity.

I intentionally left the three `SELECT FOR UPDATE` session rows unchecked:
the existing accepted fixture reads, updates from another backend, and reads
again through `Custom Scan (EcSpireDistributedScan)`, but it is not the exact
`SELECT FOR UPDATE` shape requested by the broken-down task file.

## Files

- `src/tests/remote_search/catalog_cleanup_policy.rs`
- `plan/tasks/task30-phase12c-spire-test-coverage.md`

`src/tests/remote_search/catalog_cleanup_policy.rs` is 1100 lines after this
change, below the 2500-line target.

## Validation

- `cargo fmt --check` passed.
- `git diff --check -- src/tests/remote_search/catalog_cleanup_policy.rs plan/tasks/task30-phase12c-spire-test-coverage.md` passed.
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_remote_pk_select_isolation_contract_sql --no-run` passed.
- `cargo pgrx test pg18 test_ec_spire_remote_pk_select_isolation_contract_sql` failed before test execution with:
  `undefined symbol: pg_re_throw`.

## Review Needs

Please verify that only the cross-reference row should be closed by this
slice, and that the remaining 12c.11.b `SELECT FOR UPDATE` rows should stay
open until an exact session fixture exists or the phase decides to defer them.
