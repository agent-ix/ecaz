# Review Request: SPIRE CustomScan Local Statement Timeout

agent: coder1
date: 2026-05-15
code_commit: 8aa6dd92
task: SPIRE task 12c.2.c

## Summary

Adds user-facing CustomScan coverage for the `local_statement_timeout`
Stage E fault row. The existing coverage only exercised the lower-level
transport bridge; this fixture drives the SQL CustomScan path.

## Changes

- Added `src/tests/custom_scan_timeout.rs` as a small timeout-specific
  CustomScan slice.
- Included the new slice from `src/tests/mod.rs`.
- Added `test_ec_spire_customscan_local_statement_timeout_sql`:
  - builds matched coordinator and loopback remote SPIRE indexes,
  - shadows loopback `ec_spire_remote_search(...)` through a search-path
    schema that sleeps before delegating to the real public function,
  - runs the coordinator CustomScan with `SET LOCAL statement_timeout = '20ms'`,
  - asserts the user-facing query fails with a statement-timeout error,
  - clears the timeout, routes the descriptor back to the normal loopback
    endpoint, and asserts the next CustomScan succeeds.
- Updated the 12c.2.c tracker rows with the new fixture as evidence.

## File-Size Discipline

- `src/tests/custom_scan_timeout.rs`: 117 lines.
- Existing adjacent CustomScan files remain under target:
  - `src/tests/custom_scan.rs`: 1371 lines.
  - `src/tests/custom_scan_execution.rs`: 348 lines.
  - `src/tests/custom_scan_tuple_transport.rs`: 154 lines.

This keeps timeout-specific coverage out of the broader CustomScan file.

## Validation

Passed:

- `cargo fmt --check`
- `git diff --check -- src/tests/mod.rs src/tests/custom_scan_timeout.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_local_statement_timeout_sql --no-run`

Attempted PG18 runtime:

- `cargo pgrx test pg18 test_ec_spire_customscan_local_statement_timeout_sql`

Result: failed before the test body executed with the existing local loader
issue:

```text
undefined symbol: pg_re_throw
```

## Review Focus

- Confirm the search-path `ec_spire_remote_search` shim is an acceptable way to
  make the loopback remote scan duration exceed local `statement_timeout` while
  preserving the real endpoint contract.
- Confirm the post-timeout successful CustomScan is sufficient no-leak evidence
  for the 12c.2.c tracker row.
