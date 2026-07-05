# Task 94 Review Request: Grouped-PQ Shape Prevalidation

## Scope

This packet closes a local hardening gap in the shared AM grouped-PQ batch scorer.

The previous path validated LUT shape before scoring, but validated candidate metadata/code shape inside the scalar and block loops. For a batch with a malformed candidate after the first 32-candidate block, earlier scores could be written before the function returned an error. Counters were still gated on `Ok`, but the no-partial-score failure contract was weaker than the test-only grouped-PQ helper.

## Changes

- Added `validate_grouped_pq_batch_shapes(...)` in `src/am/common/candidate_batch.rs`.
- `score_grouped_pq_batch_inner(...)` now validates every grouped-PQ payload metadata/code shape before width gating or score writes.
- Added `grouped_pq_batch_shape_error_scores_nothing_and_records_no_counters`, which places a malformed code at candidate 33 and asserts:
  - the call returns the expected shape error,
  - all output scores remain at the sentinel value,
  - block-kernel counter snapshots remain empty,
  - legacy candidate-batch compatibility counters remain zero.

## Validation

- `cargo fmt --check`: passed; see `artifacts/cargo-fmt-check.log`.
- `cargo test grouped_pq_batch --lib`: passed, `7 passed`; see `artifacts/cargo-test-grouped-pq-batch.log`.

No CI or AWS runs were used for this packet.
