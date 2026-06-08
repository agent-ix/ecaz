# Task 85 Packet 022: Summary Scoring Single-Payload Fast Path

## Summary

This packet implements the first summary-scoring CPU slice after packet 021
showed summary scoring is the current retained-recall latency lever.

The change is exact-preserving: when a summary payload contains exactly one
zero-gamma representative, `SpirePreparedAssignmentScorer` now scores that
payload directly instead of routing it through the max-over-chunks batch path.
Multi-representative summaries still use the existing max-over-chunks logic.

## Code Change

- Commit: `f90c8202e0f79fc2df8e5ff2763d1fd856b427d3`
- Files:
  - `src/am/ec_spire/quantizer/mod.rs`
  - `src/am/ec_spire/quantizer/tests.rs`

The retained block16 summary surface is single-representative, so this targets
per-summary dispatch/scratch overhead while preserving score values, ordering,
candidate sets, recall semantics, and rerank width.

## Validation

- `cargo fmt --check`: passed.
- `CARGO_DISABLE_GIT_DISCOVERY=1 cargo test -p ecaz --lib --locked --offline assignment_scorer -- --nocapture`: passed, 9 tests.

The new unit test verifies:

- single-payload chunk-max scoring equals direct single-payload scoring;
- multi-payload chunk-max scoring still equals the max of direct payload
  scores;
- both TurboQuant and RaBitQ scorer variants are covered.

## Next Required Evidence

This packet is not an AWS acceptance packet. The next Task 85 checkpoint must
measure this commit on AWS 1M/q500 against the packet 021 V5 repeat and the
best retained packet 019 bar:

- recall@10 must stay at or above retained;
- `candidate_sum` and `heap_rerank_sum` must remain unchanged unless recall
  improves;
- summary-score and candidate-score funnel timing must move down enough to
  improve end-to-end p50/p95.
