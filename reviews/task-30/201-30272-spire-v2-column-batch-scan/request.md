# Review Request: SPIRE V2 Column Batch Scan

Status: open
Branch: `task30-spire-partition-object-spec`
Checkpoint commit: `1d785c8c Batch score SPIRE V2 leaf columns`

## Scope

This packet covers the A2 hot-path follow-up for Task 30: quantized routed
candidate scans now score build-produced V2 leaf columns directly instead of
reconstructing owned assignment rows before scoring.

Changed files:

- `src/am/ec_spire/scan.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
- `plan/design/spire-foundation-architecture-feedback-response.md`

## What Changed

- Reworked `collect_quantized_routed_probe_candidates` to:
  - validate the snapshot once
  - load and route through the root object
  - read each routed V2 leaf object
  - batch-score each V2 column segment with
    `SpirePreparedAssignmentScorer::score_batch_ip`
  - construct scored candidates only for visible primary rows
- Preserved V1 leaf row-scoring fallback for compatibility helpers and tests.
- Shared candidate dedupe insertion between the row and V2 column scan paths.
- Updated the bad-payload scan test to assert the new batch stride validation
  error.
- Marked the Task 30 borrowed-read/batch-scoring architecture item complete for
  the Phase 1 quantized scan route.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - Result: `181 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check`

Known formatting warning remains unchanged from prior checkpoints: stable
rustfmt reports that `imports_granularity` and `group_imports` require nightly.

## Review Notes

The generic `collect_ranked_routed_probe_candidates` row helper remains for
tests and non-quantized compatibility. The persisted Phase 1 scan path should
continue through `collect_quantized_routed_probe_candidates`, which now uses
the V2 column route.
