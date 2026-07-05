# Review Request: C1 ADR-030 V2 Grouped Rerank Payload Seam

## Context

Packet `339` added the graph-side cold rerank fetch seam:

- typed grouped rerank payload loader
- pg proof that grouped-v2 can follow `reranktid` and decode the cold tuple

The grouped scorer boundary still only validated the hot grouped payload shape. It did not yet
compose that hot payload with the cold rerank tuple.

## Problem

Without a single helper that joins:

- grouped hot payload
- `reranktid`
- cold rerank tuple

the first real grouped scorer would still have to wire storage fetch, validation, and scoring inputs
all at once.

## Planned Slice

Add a scorer-local typed hot+cold payload seam:

1. typed merged rerank payload view
2. helper to compose hot grouped payload with loaded cold rerank payload
3. helper to load that merged payload from `GroupedScoreContext`
4. make the grouped scorer stub use that helper while still returning the existing grouped-v2
   unsupported error

This slice intentionally excludes:

- no grouped scorer implementation yet
- no runtime gate lift
- no rerank execution yet

## Implementation

Updated:

- `src/am/scan.rs`

Concrete changes:

1. added `GroupedScoreRerankPayload`
2. added `grouped_score_rerank_payload(...)`
3. added `load_grouped_score_rerank_payload(...)`
4. changed `score_grouped_candidate_context(...)` to require the merged hot+cold payload seam before
   returning the existing `ADR030_GROUPED_V2_SCAN_UNSUPPORTED` error
5. added unit tests for:
   - preserving fields across hot+cold composition
   - rejecting mismatched rerank tid / rerank payload width

## Measurements

This packet is a scorer-prep seam, so there are no new latency or recall measurements.

Known validation results for this attempt:

- focused validation:
  - `cargo test grouped_score_rerank_payload_preserves_hot_and_cold_fields --lib`: passed
  - `cargo test grouped_score_rerank_payload_rejects_mismatched_cold_payload --lib`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- full checkpoint:
  - `cargo test`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

The grouped scorer boundary now has a typed payload seam that already includes the cold rerank
payload.

What this de-risks:

1. the first real grouped scorer can focus on scoring logic instead of storage composition
2. the eventual tiny-rerank stage can reuse the same merged payload boundary
3. hot grouped payload validation and cold rerank validation are now explicit preconditions for the
   grouped scorer path

## Next Slice

The next runtime-hardening slice should keep tightening grouped-v2 assumptions:

1. grouped-v2 metadata validation for required payload flags
2. then metadata/runtime validation for any remaining grouped codec assumptions
