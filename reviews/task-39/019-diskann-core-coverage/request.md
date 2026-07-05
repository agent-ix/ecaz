# Task 39 Review Request: DiskANN Core Coverage

Code checkpoint: `3a2c6f86aa1f85524a8c64ef6bdc0f5acfc56717`

## Summary

This packet raises Task 39 coverage gates for the pgrx-free DiskANN core build
and scan modules.

Changes:

- Added DiskANN `build`, `scan`, `persist`, and `reader` modules to the
  `hardening/careful` harness so their existing pure-Rust tests contribute to
  the coverage ratchet.
- Re-exported `am::common::training` in the careful harness and provided a
  no-op careful-only `maybe_check_for_interrupts` shim for the scan module.
- Switched `src/am/ec_diskann/scan.rs` from a relative `super` import to the
  crate-qualified DiskANN module path so the same source compiles in both the
  production crate and careful harness.
- Raised the coverage baseline for `am/ec_diskann/build.rs` from `0.00%` to
  `96.69%`.
- Raised the coverage baseline for `am/ec_diskann/scan.rs` from `0.00%` to
  `96.95%`.

`am/ec_diskann/routine.rs` remains at `0.00%` in this packet because it is
PG callback glue rather than pgrx-free core logic; it is still an open Task 39
coverage gap.

## Evidence

- Focused careful DiskANN tests:
  `artifacts/careful-diskann-core-tests-rerun.log`
  - 111 passed, 0 failed.
- Coverage: `artifacts/coverage/summary.txt`
  - `am/ec_diskann/build.rs`: 272 lines, 9 missed, `96.69%` line coverage.
  - `am/ec_diskann/scan.rs`: 884 lines, 27 missed, `96.95%` line coverage.
  - `am/ec_diskann/routine.rs`: still `0.00%`, intentionally not claimed by
    this pure-Rust harness packet.
- Coverage baseline completeness:
  `artifacts/coverage-baseline-check.log`
  - `coverage baseline complete for 40 critical paths`.
- Production compile check: `artifacts/cargo-check-pg18-bench.log`
  - passed with pre-existing warnings.
- Whitespace check: `artifacts/git-diff-check.log`
  - no whitespace errors.

## Review Notes

Please focus on whether the careful harness exports are narrow enough and
whether raising only `build.rs` and `scan.rs` is the right Task 39 ratchet
boundary for this DiskANN core slice.
