# Task 85 Packet 020: V5 Selected Segment Locators

## Summary

This checkpoint implements the Task 85 object-read/physical-layout workstream locally:

- adds SPIRE leaf summary segment format V5;
- stores a row-segment locator per leaf block summary for newly built V3/V4-style leaf objects;
- propagates that locator through selected leaf block row ranges;
- reads selected row segments directly from the summary locator when all selected ranges have locators;
- falls back to the legacy first-segment chain for old objects or invalid locators.

This does not change selected-block scoring, rerank width, candidate scoring semantics, or recall policy. It changes only the physical read path available to newly rebuilt indexes. The next acceptance step is an AWS 1M/q500 rebuild and same-recall latency measurement.

## Validation

All validation used `CARGO_DISABLE_GIT_DISCOVERY=1` because this checkout has very large review/benchmark artifact trees; plain Cargo was spending minutes fingerprinting Git/untracked state before compiling.

- `cargo test -p ecaz --lib --locked --offline leaf_partition_object_v -- --nocapture`
  - `9 passed; 0 failed`
  - log: `artifacts/cargo-test-leaf-partition-object-v.log`
- `cargo test -p ecaz --lib --locked --offline leaf_block_row_ranges -- --nocapture`
  - `6 passed; 0 failed`
  - log: `artifacts/cargo-test-leaf-block-row-ranges.log`
- `cargo test --manifest-path crates/ecaz-cli/Cargo.toml spire --locked --offline`
  - `56 passed; 0 failed`
  - log: `artifacts/cargo-test-ecaz-cli-spire.log`

## Review Notes

The local proof verifies that a selected V5 summary locator can read the selected segment even when the legacy `first_segment_locator` is unavailable. V3/V4 compatibility is covered by existing round-trip tests, now asserting that decoded new-format summaries carry real locators while old expected summaries compare equal after clearing those physical locators.

This packet should not be treated as Task 85 acceptance. Acceptance still requires a rebuilt AWS 1M/q500 index so the retained block16/global1152 point can be measured with V5 row-segment locators at matched recall and candidate surface.
