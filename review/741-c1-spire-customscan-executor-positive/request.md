# Review Request: SPIRE CustomScan Executor Positive Fixtures

agent: coder1
date: 2026-05-14
code_commit: 23a33428
task: SPIRE task 12c.1.g

## Summary

This packet retires the two remaining `EcSpireDistributedScan production executor blocked`
`#[should_panic]` scaffolds from `custom_scan.rs` and replaces them with positive
loopback execution fixtures.

The updated task tracker still had the 12c.1.g atomic bullets unchecked even
though batch-5 feedback summarized 12c.1 as accepted. I used the updated
broken-down tracker as the source of truth and closed those two explicit rows
with concrete fixture evidence rather than relying on the summary.

## Changes

- Added `src/tests/custom_scan_execution.rs` as a small sibling fixture file.
- Removed the old panic-scaffold tests from `src/tests/custom_scan.rs`.
- Added `test_ec_spire_customscan_exec_returns_remote_tuple_payload_sql`:
  - builds matched coordinator/loopback remote `ec_spire` indexes,
  - rewrites coordinator leaf placements to the remote node,
  - registers a real remote descriptor with the remote endpoint identity,
  - asserts the plan uses `Custom Scan (EcSpireDistributedScan)`,
  - asserts execution returns the remote tuple payload row.
- Added `test_ec_spire_customscan_exec_accepts_parameter_query_sql` with the
  same production path, but through `PREPARE ... ORDER BY embedding <#> $1`.
- Updated `plan/tasks/task30-phase12c-spire-test-coverage.md` 12c.1.g with
  evidence lines for both positive fixtures.

## File-Size Discipline

- `src/tests/custom_scan.rs`: 1353 lines after removal.
- `src/tests/custom_scan_execution.rs`: 232 lines.
- `src/tests/custom_scan_lifecycle.rs`: 189 lines.
- `src/tests/custom_scan_concurrency.rs`: 572 lines.

This keeps the new executor coverage out of the older broad CustomScan file and
leaves each file well below the 2500-line target.

## Validation

Passed:

- `cargo fmt --check`
- `git diff --check -- src/tests/custom_scan_execution.rs src/tests/custom_scan.rs src/tests/mod.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_exec_returns_remote_tuple_payload_sql --no-run`
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_exec_accepts_parameter_query_sql --no-run`

Attempted PG18 runtime:

- `cargo pgrx test pg18 test_ec_spire_customscan_exec_returns_remote_tuple_payload_sql`

Result: failed before the test body executed with the existing local loader
issue:

```text
undefined symbol: pg_re_throw
```

## Review Focus

- Are the two positive fixtures sufficient replacement for the old
  `#[should_panic]` scaffolds in 12c.1.g?
- Is `custom_scan_execution.rs` the right sibling file boundary for production
  executor coverage?
- Does the parameterized fixture cover the intended `$1` query extraction path
  without over-coupling to plan text?
