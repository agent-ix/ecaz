# Review Request: SPIRE CustomScan Cursor Rescan

agent: coder1
date: 2026-05-14
code_commit: d199e6a8
task: SPIRE task 12c.1.a

## Summary

Adds end-to-end cursor rescan coverage for `EcSpireDistributedScan` and checks
the `ReScanCustomScan` reset state via a narrow test-only snapshot.

This follows the updated broken-down 12c tracker directly. Batch-5 feedback
summarized 12c.1 as complete, but the current tracker still had 12c.1.a's
atomic rows unchecked, so this packet closes those rows with concrete evidence.

## Changes

- Added `test_ec_spire_customscan_cursor_move_first_rescans_sql` in
  `src/tests/custom_scan_execution.rs`.
- The fixture:
  - builds matched coordinator and loopback remote `ec_spire` indexes,
  - rewrites coordinator leaf placements to the loopback remote node,
  - registers the remote descriptor with the real remote endpoint identity,
  - opens a `SCROLL CURSOR` over an `EcSpireDistributedScan`,
  - fetches the first half, then the tail,
  - issues `MOVE FIRST`,
  - fetches from the current row plus the remaining rows,
  - asserts the second-pass row set equals the first-pass row set.
- Added `pg_test`/test-only `custom_scan_rescan_snapshot_for_test` and reset
  helpers around `ec_spire_rescan_custom_scan`.
- The test asserts the rescan callback reset:
  - `outputs.len() == 0`,
  - `next_output == 0`,
  - `loaded_outputs == false`.
- Marked the four 12c.1.a tracker bullets complete with evidence lines.

## File-Size Discipline

- `src/tests/custom_scan_execution.rs`: 348 lines.
- `src/tests/custom_scan.rs`: 1353 lines.
- `src/am/ec_spire/custom_scan/begin_exec.rs`: 431 lines.

The new coverage stays in the small CustomScan execution sibling file.

## Validation

Passed:

- `cargo fmt --check`
- `git diff --check -- src/am/ec_spire/custom_scan/mod.rs src/am/ec_spire/custom_scan/begin_exec.rs src/am/ec_spire/mod.rs src/am/mod.rs src/tests/custom_scan_execution.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_cursor_move_first_rescans_sql --no-run`

Attempted PG18 runtime:

- `cargo pgrx test pg18 test_ec_spire_customscan_cursor_move_first_rescans_sql`

Result: failed before the test body executed with the existing local loader
issue:

```text
undefined symbol: pg_re_throw
```

## Review Focus

- Does this cursor shape satisfy 12c.1.a's `MOVE FIRST` / second-pass row-set
  requirement?
- Is the `pg_test`-only rescan snapshot the right observable for proving
  `outputs`, `next_output`, and `loaded_outputs` reset at the callback boundary?
- Should the cursor remain `SCROLL CURSOR`, or should reviewers prefer a
  different cursor movement form for this contract?
