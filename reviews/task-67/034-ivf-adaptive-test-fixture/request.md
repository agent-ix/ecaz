# Task 67 Review Request: IVF adaptive nprobe test fixture

## Summary

This checkpoint fixes a stale IVF scan unit-test fixture that blocked the Task 67 AC4 scan validation pass.

The production code is unchanged. The test named `adaptive_nprobe_keeps_requested_width_when_gap_is_small` claimed to model a small adaptive boundary gap, but its fixture put the boundary between scores `9.80` and `9.70`, which is a large score gap and correctly triggers adaptive reduction. The fixture now uses `9.7995` for the next centroid, making the boundary gap small enough for the assertion to match the test name.

## Code Under Review

- `src/am/ec_ivf/scan.rs`
- code commit: `f6bad4800a745b329130651f8c18005d42b765e6`

## Validation

Packet-local logs are under `artifacts/local/`; see `artifacts/manifest.md`.

- IVF scan tests failed before the fixture correction: 22 passed, 1 failed
- The exact failing test reproduced the same assertion
- IVF scan tests passed after the correction: 23 passed, 0 failed

Additional closeout-prep logs retained in this packet:

- `cargo test -p ecaz --lib quant::rabitq`: passed, 46 tests
- `cargo test -p ecaz --lib am::ec_diskann::scan::tests`: passed, 18 tests
- `cargo test -p ecaz --lib am::ec_hnsw::scan::tests`: passed, 73 tests

## Notes

This is not a RaBitQ kernel behavior change. It corrects a pre-existing test fixture so the AC4 scan-regression validation can be cited cleanly in the closeout audit.
