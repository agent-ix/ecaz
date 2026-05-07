# Review Request: SPIRE Batch Assignment Scoring

Status: open
Branch: `task30-spire-partition-object-spec`
Checkpoint commit: `46b91144 Add SPIRE batch assignment scoring`

## Scope

This packet covers a partial A2 pre-persistence architecture feedback slice:
add a batch-oriented assignment scorer API that V2 column-major leaf reads can
drive later.

Changed files:

- `src/am/ec_spire/quantizer.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
- `plan/design/spire-foundation-architecture-feedback-response.md`

## What Changed

- Added `SpirePreparedAssignmentScorer::score_batch_ip`.
- The batch scorer validates:
  - gamma count matches output count
  - payload byte count matches `payload_stride * row_count`
  - payload stride matches the prepared scorer's expected payload format and
    dimensions
- TurboQuant and RaBitQ initially loop over fixed-stride payload chunks behind
  the batch API.
- RaBitQ batch scoring preserves the existing `gamma == 0` invariant.
- Added tests proving batch scores match scalar scoring for TurboQuant and
  RaBitQ, and that bad batch shapes are rejected.
- Updated the Task 30 plan and architecture feedback response to record the
  batch-scorer slice while keeping A2 open until V2 column views and scan
  migration land.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - Result: `175 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check`

Known formatting warning remains unchanged from prior checkpoints: stable
rustfmt reports that `imports_granularity` and `group_imports` require nightly.

## Review Notes

This checkpoint does not change the scan hot path yet. Current scans still call
the scalar scorer row-by-row; this adds the batch contract needed before V2
column views can hand contiguous payload/gamma arrays to the scorer.
