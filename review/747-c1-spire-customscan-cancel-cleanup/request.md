# Review Request: SPIRE CustomScan Cancel Cleanup

agent: coder1
date: 2026-05-15
code_commit: ff13af5a
task: SPIRE task 12c.1.b partial

## Summary

Converts the existing CustomScan read-cancel fixture from a
`#[should_panic]` smoke test into an assertion-based cleanup test.

This closes the updated tracker rows for canceling mid-`ExecCustomScan` and
asserting `EndCustomScan` runs exactly once on the unwind path. The
MemoryContextStats baseline/return rows remain unchecked.

## Changes

- Removed `#[should_panic]` from
  `test_ec_spire_customscan_read_cancel_releases_transport`.
- Wrapped the query in `PgTryBuilder` and asserted the captured PostgreSQL
  cancel error contains `canceling statement due to user request`.
- Reset the existing test-only CustomScan cleanup counters immediately before
  the canceled query.
- Asserted:
  - `EndCustomScan` count is exactly `1`.
  - executor-state `pfree` count is exactly `1`.
- Updated the 12c.1.b tracker rows for the interrupt and `EndCustomScan`
  callback evidence only.

## File-Size Discipline

- `src/tests/custom_scan.rs`: 1371 lines after this change.

No new broad test file growth; the change stays within the existing fixture
that already owns CustomScan read-cancel setup.

## Validation

Passed:

- `cargo fmt --check`
- `git diff --check -- src/tests/custom_scan.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_read_cancel_releases_transport --no-run`

Attempted PG18 runtime:

- `cargo pgrx test pg18 test_ec_spire_customscan_read_cancel_releases_transport`

Result: failed before the test body executed with the existing local loader
issue:

```text
undefined symbol: pg_re_throw
```

## Review Focus

- Confirm replacing `#[should_panic]` with `PgTryBuilder` plus explicit cleanup
  counter assertions is the right tightening for the cancel path.
- Confirm this should close only the cancel and `EndCustomScan` 12c.1.b rows,
  leaving the two MemoryContextStats rows unchecked.
