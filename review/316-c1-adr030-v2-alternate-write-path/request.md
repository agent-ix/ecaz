# Review Request: C1 ADR-030 V2 Alternate Write Path

## Context

Packet `315` added a builder-side seam that assembles one logical build tuple into:

- one grouped hot tuple
- one rerank tuple

The next narrow slice is to use that seam in a synthetic alternate write path that stages a full
v2-shaped `DataPageChain`, still without switching the live build path or metadata format.

## Problem

We now have:

1. tuple contracts
2. page placement helpers
3. builder assembly for one tuple

But we still do not have an end-to-end alternate path that proves a whole build can be staged into
v2-shaped pages coherently.

## Planned Slice

Add an alternate builder-side staging path that:

1. accepts existing `BuildTuple` data plus graph nodes
2. accepts caller-supplied grouped search-code payloads
3. stages:
   - neighbor tuples
   - rerank tuples
   - grouped hot tuples
4. patches neighbor refs to point at grouped hot tuple tids

This slice still excludes:

- no live build switchover
- no real grouped encoder in build yet
- no v2 metadata writes yet

## Implementation

Added an alternate builder-side staging path that produces a full v2-shaped `DataPageChain` from
existing build tuples plus caller-supplied grouped search-code payloads.

New seam:

- `stage_v2_grouped_page_chain(...) -> V2GroupedStagedChain`

Staged outputs:

1. neighbor tuples
2. rerank tuples
3. grouped hot tuples
4. TID vectors for each staged tuple kind

Behavior:

1. inserts placeholder neighbor tuples first
2. inserts rerank tuples next
3. assembles grouped hot tuples through the packet `315` build-assembly seam
4. inserts grouped hot tuples
5. patches neighbor refs to point at grouped hot tuple tids

Test added:

- validates that the staged chain links:
  - hot tuple to rerank tuple
  - neighbor tuple to grouped hot tuple
  - grouped search code to the expected hot payload

This is still synthetic because grouped search codes are supplied by the caller, but it is the
first end-to-end builder-side path that stages actual v2-shaped pages coherently.

## Measurements

This packet is a synthetic alternate build-path slice, so there are no new recall or latency
measurements.

Known validation results for this attempt:

- `cargo test stage_v2_grouped_page_chain_links_hot_neighbor_and_rerank_tuples --lib`: passed
- `cargo clippy --lib --tests -- -D warnings`: passed
- `cargo test`: passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed

## Outcome

ADR-030 v2 now has a coherent alternate write path in-memory.

What this de-risks:

1. hot, rerank, and neighbor tuple staging now works together instead of only in isolated pieces
2. grouped hot tuple neighbor refs can be patched consistently after staging
3. the future live builder switchover can target an existing alternate path instead of inventing
   one from scratch

## Next Slice

The next narrow slice should replace caller-supplied grouped search codes with a builder-side code
generation seam:

1. derive grouped search codes from existing build inputs in a deterministic way
2. feed those generated codes into the alternate v2 staging path
3. still keep the live build path unchanged
