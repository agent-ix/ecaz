# Review Request: SPIRE Empty CustomScan Cleanup Pin

agent: coder1
date: 2026-05-14
code commit: `51b7b9a88f21d419c52c3d18d9d2945354e222ab`
task rows: closes `12c.16.b`

## Summary

This checkpoint tightens the empty remote-result CustomScan fixture.
The existing test already pinned that the empty remote result returns
zero rows and keeps tuple transport status `ready`; it now also asserts
cleanup behavior for the counted execution.

## Changes

- Added `pg_test`/test-only CustomScan cleanup counters:
  - `EndCustomScan` callback count.
  - `pfree` count immediately before freeing the executor state.
- Wired the counters through the existing test-only `am` facade.
- Updated `test_ec_spire_customscan_empty_remote_result_returns_no_rows`
  to reset counters before the counted CustomScan execution and assert:
  - `EndCustomScan` runs exactly once.
  - the executor-state `pfree` path runs exactly once.
- Marked `12c.16.b` complete in the Phase 12c tracker.

File-size check:

- `src/tests/custom_scan.rs`: 1479 lines.
- `src/am/ec_spire/custom_scan/begin_exec.rs`: 383 lines.
- `src/am/ec_spire/custom_scan/mod.rs`: 193 lines.

## Validation

- `cargo fmt --check`
  - Passed, with existing stable-rustfmt warnings about unstable import
    grouping options.
- `git diff --check -- src/am/mod.rs src/am/ec_spire/mod.rs src/am/ec_spire/custom_scan/mod.rs src/am/ec_spire/custom_scan/begin_exec.rs src/tests/custom_scan.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Passed.
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_empty_remote_result_returns_no_rows --no-run`
  - Passed.
- `cargo pgrx test pg18 test_ec_spire_customscan_empty_remote_result_returns_no_rows`
  - Failed before test execution with the existing loader error:
    `undefined symbol: pg_re_throw`.

## Review Focus

- Please check that the counter hook is acceptably narrow: it is
  compiled only for `test`/`pg_test`, and the production callback path
  only gains no-op calls in non-test builds.
- Please check whether counting the `pfree` delta immediately before
  freeing the executor state is the right observable for the 12c.16.b
  cleanup requirement.
- Please check that resetting counters immediately before the counted
  query avoids counting the prior `EXPLAIN ANALYZE` execution in the
  same test.
