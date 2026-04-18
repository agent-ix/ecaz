# Review Request: C1 ADR-030 V2 Grouped Score Context Seam

## Context

Packet `331` made grouped candidate-score dispatch metadata-aware by introducing:

- `GroupedScoreShape`
- `GroupedScoreCall<'a>`

That let grouped dispatch carry both:

1. the grouped hot payload bytes from the cached element
2. the grouped search layout from scan metadata

But `candidate_score_dispatch(...)` was still assembling that grouped call inline.

## Problem

The future grouped scorer will need a stable, typed context boundary that is built from:

1. scan-state shape
2. cached grouped hot payloads
3. element identity

If dispatch keeps assembling that context inline, the eventual grouped scorer packet would still
need to refactor dispatch and helper boundaries at the same time.

## Planned Slice

Extract one dedicated grouped scorer context builder:

1. build grouped score context from scan storage plus cached element
2. keep grouped runtime behavior unchanged
3. keep scalar exact scoring unchanged

This still excludes:

- no grouped-v2 traversal enablement
- no grouped approximate scorer
- no rerank fetch path
- no behavior change for grouped-v2 scans

## Implementation

Updated `src/am/scan.rs`:

1. added `GroupedScoreContext<'a>` with:
   - `element_tid: page::ItemPointer`
   - `call: GroupedScoreCall<'a>`
2. added `grouped_score_context_from_scan_state(...)`
3. changed `candidate_score_dispatch(...)` to build grouped dispatch through that helper
4. kept grouped runtime behavior unchanged:
   - grouped dispatch still reaches the same grouped helper stub
   - grouped helper still errors with `ADR030_GROUPED_V2_SCAN_UNSUPPORTED`

New tests:

- `grouped_score_context_uses_scan_shape_and_cached_payloads`
- `grouped_score_context_requires_grouped_scan_storage`

## Measurements

This packet is still scorer-seam work, so there are no new latency or recall measurements.

Known validation results for this attempt:

- focused validation:
  - `cargo test grouped_score_context_uses_scan_shape_and_cached_payloads --lib`: passed
  - `cargo test grouped_score_context_requires_grouped_scan_storage --lib`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- full checkpoint:
  - `cargo test`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

Grouped score dispatch now builds through one dedicated context seam instead of assembling shape and
payload inline inside the dispatch match.

What this de-risks:

1. the grouped scorer packet can replace one helper boundary instead of rewriting dispatch again
2. grouped dispatch now has a stable place to accumulate scorer-only inputs
3. element identity is now carried alongside grouped score inputs before real grouped scoring lands

## Next Slice

The next narrow slice should move the grouped helper boundary from raw grouped calls to full grouped
score context:

1. pass `GroupedScoreContext<'_>` into grouped score helper dispatch
2. keep runtime behavior unchanged
3. prepare one direct cut point for the first real grouped scorer implementation
