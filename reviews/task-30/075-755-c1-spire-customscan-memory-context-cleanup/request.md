# Review Request: SPIRE CustomScan Memory Context Cleanup

## Summary

Coder: `coder1`
Topic: `755-c1-spire-customscan-memory-context-cleanup`
Code commit: `1df909d564b5fb026f105952b965f6f98426c1b6`
Date: `2026-05-15`

This checkpoint closes the remaining 12c.1.b memory-context rows in the
updated Phase 12c tracker. It adds a `#[cfg(test)/pg_test]` CustomScan
memory-context snapshot around `CreateCustomScanState` allocation and
`EndCustomScan` free, then extends
`test_ec_spire_customscan_read_cancel_releases_transport` to assert:

- the pre-allocation memory context baseline was captured;
- the post-`EndCustomScan` memory context snapshot was captured;
- post-end used bytes are not above the baseline captured before the
  CustomScan state allocation.

This stays within the Phase 12c testability-hook boundary: the new state
field and counters are all gated by `#[cfg(any(test, feature = "pg_test"))]`.

## Files

- `src/am/ec_spire/custom_scan/begin_exec.rs`
- `src/am/ec_spire/custom_scan/mod.rs`
- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/tests/custom_scan.rs`
- `plan/tasks/task30-phase12c-spire-test-coverage.md`

`src/tests/custom_scan.rs` is 1378 lines after this change, below the
2500-line target.

## Validation

- `cargo fmt --check` passed.
- `git diff --check -- src/am/mod.rs src/am/ec_spire/mod.rs src/am/ec_spire/custom_scan/mod.rs src/am/ec_spire/custom_scan/begin_exec.rs src/tests/custom_scan.rs plan/tasks/task30-phase12c-spire-test-coverage.md` passed.
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_read_cancel_releases_transport --no-run` passed.
- `cargo pgrx test pg18 test_ec_spire_customscan_read_cancel_releases_transport` failed before test execution with:
  `undefined symbol: pg_re_throw`.

## Review Needs

Please verify that `MemoryContextMemConsumed` counters are an acceptable
testable stand-in for the tracker's `MemoryContextStats` wording, and that
the `#[cfg(test)/pg_test]` allocation-context field is narrow enough for
Phase 12c's test-only scope.
