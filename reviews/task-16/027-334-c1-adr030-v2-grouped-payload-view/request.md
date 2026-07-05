# Review Request: C1 ADR-030 V2 Grouped Payload View

## Context

Packet `333` moved grouped helper dispatch onto `GroupedScoreContext<'a>`, so the grouped helper
stub now receives one typed object containing:

1. element identity
2. grouped search shape
3. grouped hot payload bytes

But the grouped helper still had no local seam that validates the cached grouped payload lengths
against the metadata-derived grouped layout.

## Problem

Before the first real grouped LUT scorer lands, the grouped helper needs one narrow place that:

1. validates grouped hot payload lengths against grouped metadata shape
2. exposes a scorer-local borrowed payload view
3. fails loudly if the helper ever sees inconsistent cached payloads

Without that seam, the first scorer packet would have to mix scoring logic with helper-local shape
validation.

## Planned Slice

Add a grouped helper-local payload view:

1. derive one borrowed grouped payload view from `GroupedScoreContext<'a>`
2. validate `binary_words.len()` against `binary_word_count`
3. validate `search_code.len()` against `search_code_len`
4. keep runtime behavior unchanged

This still excludes:

- no grouped-v2 traversal enablement
- no grouped approximate scorer
- no rerank fetch path
- no behavior change for grouped-v2 scans

## Implementation

Updated `src/am/scan.rs`:

1. added `GroupedScorePayloadView<'a>` with:
   - `element_tid`
   - `reranktid`
   - borrowed `binary_words`
   - borrowed `search_code`
   - expected `rerank_code_len`
2. added `grouped_score_payload_view(...)`
3. validated grouped helper-local payload widths against grouped metadata shape
4. updated `score_grouped_candidate_context(...)` to build the payload view before returning the
   existing grouped-v2 unsupported error

New tests:

- `grouped_score_payload_view_preserves_context_fields`
- `grouped_score_payload_view_rejects_shape_mismatch`

## Measurements

This packet is still scorer-seam work, so there are no new latency or recall measurements.

Known validation results for this attempt:

- focused validation:
  - `cargo test grouped_score_payload_view_preserves_context_fields --lib`: passed
  - `cargo test grouped_score_payload_view_rejects_shape_mismatch --lib`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- full checkpoint:
  - `cargo test`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

The grouped helper now has a scorer-local payload-view seam that checks metadata-aligned grouped hot
payload widths before the eventual grouped LUT scorer is introduced.

What this de-risks:

1. the first grouped scorer no longer needs to add its own helper-local payload-shape checks
2. grouped helper code can work from one borrowed payload view instead of re-reading nested context
3. metadata/payload inconsistency is now caught at the grouped helper boundary, not later as recall
   noise

## Next Slice

The next narrow slice should add a grouped query-side prepared input seam:

1. derive grouped scorer query inputs from scan state in one helper
2. keep runtime behavior unchanged
3. set up the first real grouped LUT scoring implementation as `prepared query + payload view`
