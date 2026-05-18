# Review Request: SPIRE CustomScan Idle Cursor Timeout

- agent: coder1
- date: 2026-05-14
- code commit: `b83cfe76ca8276ace231e14a83ea76bdc9b16c41`
- task rows: closes `12c.8.d`

## Summary

Added a focused CustomScan concurrency/session fixture for idle-in-transaction
timeout while a cursor over `EcSpireDistributedScan` is open and unread.

## What Changed

Added `src/tests/custom_scan_concurrency.rs` and included it from
`src/tests/mod.rs`.

The fixture:

- builds coordinator and loopback remote tables/indexes
- rewrites coordinator placements to the loopback remote descriptor
- verifies the cursor query plans as `Custom Scan (EcSpireDistributedScan)`
- opens a transaction-scoped cursor over the CustomScan query and deliberately
  does not fetch from it
- sets `idle_in_transaction_session_timeout = '100ms'`
- waits for the backend to be disconnected by the timeout
- asserts no SPIRE prepared transaction state remains for the index

Updated `plan/tasks/task30-phase12c-spire-test-coverage.md` to mark all
`12c.8.d` bullets complete.

## Test File Size Discipline

This creates a new sibling file instead of growing `custom_scan.rs`:

```text
148 src/tests/custom_scan_concurrency.rs
1475 src/tests/custom_scan.rs
```

`src/tests/mod.rs` is already above the 2500-line target; this slice only adds
the include needed to keep the new fixture split out.

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/tests/mod.rs src/tests/custom_scan_concurrency.rs plan/tasks/task30-phase12c-spire-test-coverage.md
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_idle_transaction_timeout_cursor_sql --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`, but exited
successfully.

Blocked before test execution:

```text
cargo pgrx test pg18 test_ec_spire_customscan_idle_transaction_timeout_cursor_sql
```

Result:

```text
undefined symbol: pg_re_throw
```

The pg_test binary failed at local loader startup before the focused test body
could run.

## Review Focus

- Confirm opening an unread cursor with an explicit CustomScan EXPLAIN is the
  right fixture shape for `12c.8.d`.
- Confirm the cleanup assertion should remain limited to backend disconnect
  plus zero SPIRE prepared xacts, given this cursor-open/no-fetch path should
  not need remote prepared transaction state.
