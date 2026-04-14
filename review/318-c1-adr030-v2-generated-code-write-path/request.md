# Review Request: C1 ADR-030 V2 Generated-Code Write Path

## Context

Packet `317` added a grouped-code generation seam that can train a grouped PQ model from
`build_source_column` vectors and derive grouped packed search codes from those vectors.

The next narrow slice is to connect that seam to the alternate v2 write path so staged v2 pages no
longer depend on externally supplied grouped codes.

## Problem

The alternate write path is coherent, but until now it still required the caller to provide grouped
search-code payloads.

That means there was still no builder-side path that:

1. trains grouped search-code state from source-backed build tuples
2. derives grouped codes per tuple
3. stages grouped hot / rerank / neighbor tuples end-to-end

## Planned Slice

Add a source-backed alternate v2 staging path that:

1. trains the grouped PQ model from build state
2. derives grouped search codes per tuple
3. feeds those codes into the existing alternate v2 page staging path

This slice still excludes:

- no live build switchover
- no v2 metadata writes
- no runtime grouped scan path

## Implementation

Connected the real grouped-code generation seam from packet `317` to the alternate v2 page staging
path from packet `316`.

New wrapper:

- `stage_v2_grouped_page_chain_from_source(...)`

Behavior:

1. trains the grouped PQ model from source-backed build state
2. derives grouped packed search codes per tuple from source vectors
3. feeds those generated codes into the existing alternate v2 staging path

Test added:

- validates that a source-backed build state now stages:
  - generated grouped search codes in grouped hot tuples
  - rerank tuples containing the existing scalar payload
  - neighbor tuples that point at grouped hot tuple tids

This is the first packet where the alternate v2 write path stops depending on externally supplied
grouped search codes.

## Measurements

This packet is still a build-path slice, so there are no new recall or latency measurements.

Known validation results for this attempt:

- `cargo test stage_v2_grouped_page_chain_from_source_derives_codes_and_links_pages --lib`: passed
- `cargo clippy --lib --tests -- -D warnings`: passed
- `cargo test`: passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17` run 1:
  one-off failure in `pg_test_tqhnsw_successor_candidate_from_entry_adjacency`
- isolated rerun of `tests::pg_test_tqhnsw_successor_candidate_from_entry_adjacency`: passed

## Outcome

ADR-030 v2 now has a source-backed alternate build path that can:

1. train grouped search state
2. derive grouped codes
3. stage grouped hot / rerank / neighbor tuples coherently

What this de-risks:

1. the alternate v2 path is now using real generated grouped codes instead of synthetic placeholders
2. grouped-code generation and staged page layout are now proven to compose
3. the next step can focus on switching a real build lane over, rather than inventing missing seams

## Next Slice

The next narrow slice should expose a minimal alternate builder flush path behind an explicit
source-backed gate:

1. use the generated-code staging path during build when source vectors are present
2. keep current-format metadata and runtime unchanged
3. validate that the alternate flush produces a self-consistent on-disk layout without yet making
   it the default index format
