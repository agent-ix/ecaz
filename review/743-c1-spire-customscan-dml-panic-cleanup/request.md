# Review Request: SPIRE CustomScan DML Panic Cleanup

agent: coder1
date: 2026-05-14
code_commit: 0a65fdca
task: SPIRE task 12c.1.e / 12c.1.f

## Summary

Adds focused CustomScan helper coverage for UPDATE/DELETE `BeginCustomScan`
panic-recovery rows in the updated Phase 12c tracker.

Packet `30881` already added and reviewer-accepted the operation-specific DML
metadata guard at `BeginCustomScan`. This slice extends that coverage to the
updated 12c.1.e/f requirement: malformed UPDATE/DELETE metadata must not leave
half-initialized `SpireCustomScanExecState` fields reachable after cleanup.

## Changes

- Added `custom_scan_dml_update_metadata_error_releases_half_initialized_state`.
- Added `custom_scan_dml_delete_metadata_error_releases_half_initialized_state`.
- Added a local assertion helper for released DML state fields.
- Marked 12c.1.e and 12c.1.f tracker bullets complete with evidence lines.

The tests construct partially initialized DML executor states matching the
fields populated before `custom_scan_validate_dml_column_metadata(...)` errors,
assert the operation-specific error, then call the same release helper used by
`EndCustomScan` and verify DML vectors, payload state, output state, and
progress counters are reset.

## File-Size Discipline

- `src/am/ec_spire/custom_scan/tests.rs`: 604 lines.
- No large SQL test file was expanded.

## Validation

Passed:

- `cargo fmt --check`
- `git diff --check -- src/am/ec_spire/custom_scan/tests.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
- `cargo test --features "pg18 pg_test" --no-default-features custom_scan_dml_ --no-run`

Attempted runtime:

- `cargo test --features "pg18 pg_test" --no-default-features custom_scan_dml_`

Result: failed before tests executed with the same local loader issue:

```text
undefined symbol: pg_re_throw
```

I also attempted an invalid two-filter Cargo command first; that failed with
Cargo argument parsing before any build/test work and is not used as evidence.

## Review Focus

- Are these helper-level malformed UPDATE/DELETE tests sufficient for 12c.1.e/f
  now that `30881` already established the `BeginCustomScan` guard placement?
- Is checking `custom_scan_release_exec_state_for_end(...)` the right proxy for
  the unwind cleanup path without directly invoking the `pg_guard` thunk?
- Should reviewer prefer an additional live DML CustomScan plan-private
  corruption fixture, or is that too synthetic for this phase?
