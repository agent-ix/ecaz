# Review Request: C1 ADR-030 V2 Grouped Score Shape Seam

## Context

Packet `330` extracted grouped dispatch into a dedicated helper stub:

- `score_grouped_candidate_input(...)`

That gave grouped scoring a dedicated helper boundary, but the helper still only received raw grouped
payload bytes. It did not yet receive the metadata-derived grouped search shape.

## Problem

The future grouped LUT scorer will need both:

1. the cached grouped hot payloads
2. the grouped search layout from index metadata

Without threading shape through now, the eventual scorer would still have to reach back into scan
state or metadata from inside the helper.

We need one narrow seam that makes grouped dispatch metadata-aware without changing runtime
behavior.

## Planned Slice

Add grouped score shape to grouped dispatch:

1. derive grouped score shape from `scan_graph_storage`
2. carry that shape together with grouped hot payload input
3. keep grouped runtime behavior unchanged

This still excludes:

- no grouped-v2 traversal enablement
- no grouped approximate scorer
- no rerank fetch path
- no behavior change for grouped-v2 scans

## Implementation

Updated `src/am/scan.rs`:

1. added `GroupedScoreShape` with:
   - `binary_word_count`
   - `search_code_len`
   - `rerank_code_len`
2. added `GroupedScoreShape::from_scan_graph_storage(...)`
3. added `GroupedScoreCall<'a>` to combine:
   - `shape: GroupedScoreShape`
   - `input: GroupedScoreInput<'a>`
4. changed `CandidateScoreDispatch::Grouped(...)` to carry `GroupedScoreCall<'a>`
5. changed `candidate_score_dispatch(...)` to derive grouped score shape from the scan’s
   `GraphStorageDescriptor`
6. changed `score_grouped_candidate_input(...)` to accept `GroupedScoreCall<'_>`

New tests:

- `grouped_score_shape_uses_grouped_scan_layout`
- updated grouped dispatch test to assert both the grouped payload and grouped shape

## Measurements

This packet is still scorer-seam work, so there are no new latency or recall measurements.

Known validation results for this attempt:

- focused validation:
  - `cargo test grouped_score_shape_uses_grouped_scan_layout --lib`: passed
  - `cargo test candidate_score_dispatch_uses_grouped_input_for_exact_unavailable --lib`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- full checkpoint:
  - `cargo test`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: in progress while drafting this
    packet
  - final result to be confirmed before checkpoint commit

## Outcome

ADR-030 grouped dispatch is now metadata-aware. The grouped score helper boundary can see the grouped
search shape it will eventually need, instead of only raw cached payload bytes.

What this de-risks:

1. the future grouped scorer no longer needs to rediscover shape from scan state
2. grouped dispatch now carries the same two classes of inputs the real scorer will need:
   payload bytes and layout shape
3. the next packet can shape the grouped scorer context further without widening dispatch again

## Next Slice

The next narrow slice should extract a dedicated grouped scorer context builder:

1. build grouped score context from scan state plus cached element
2. keep grouped helper behavior unchanged
3. make later LUT-scoring cutover local to one context builder and one helper
