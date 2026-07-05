# Review Request: C1 ADR-030 V2 Page Placement Contract

## Context

Packet `313` defined the tuple contracts for the intended ADR-030 v2 hot/cold split:

- hot tuple for duplicate tids, graph link, rerank link, binary sidecar, grouped search code
- cold rerank tuple for gamma plus higher-fidelity rerank payload

The next narrow slice is to turn those tuple contracts into concrete page-chain storage helpers.

## Problem

The tuple types exist, but the page codec still only has chain helpers for current scalar element
tuples and neighbor tuples.

That means the rebuild path still has no concrete storage primitives for:

1. placing grouped hot tuples onto data pages
2. placing rerank tuples onto data pages
3. reading and updating those tuples through the same page-chain abstraction the builder will use

## Planned Slice

Add page and page-chain helpers for the new ADR-030 v2 tuple kinds:

1. `insert/read/update` for grouped hot tuples on `DataPage`
2. `insert/read/update` for rerank tuples on `DataPage`
3. matching `DataPageChain` helpers
4. page roundtrip and multi-page extension tests

This slice still excludes:

- no builder writes yet
- no scan reads yet
- no v2 index build path yet

## Implementation

Extended the page codec so the new ADR-030 v2 tuples can be placed, read, and updated through the
same `DataPage` / `DataPageChain` abstraction already used by the current builder.

Added `DataPage` helpers for:

1. grouped hot tuples
   - `insert_grouped_hot`
   - `read_grouped_hot`
   - `update_grouped_hot`
2. rerank tuples
   - `insert_rerank`
   - `read_rerank`
   - `update_rerank`

Added matching `DataPageChain` helpers for both tuple kinds.

Tests added:

- grouped hot tuple page roundtrip
- rerank tuple page roundtrip
- multi-page extension for grouped hot tuples
- multi-page extension for rerank tuples

The result is that ADR-030 v2 now has concrete page-placement primitives instead of only raw tuple
contracts.

## Measurements

This packet is still a storage-contract slice, so there are no new recall or latency measurements.

Known validation results for this attempt:

- `cargo test grouped_hot_tuple_page_roundtrip --lib`: passed
- `cargo test page_chain_extends_for_multiple_grouped_hot_tuples --lib`: passed
- `cargo clippy --lib --tests -- -D warnings`: passed
- `cargo test`: passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed

## Outcome

This is still not the v2 rebuild path itself, but it removes another blocker:

1. the builder now has a storage abstraction it can target for grouped hot tuples
2. the cold rerank payload also has page-chain storage helpers
3. page placement is tested independently from runtime graph/search logic

## Next Slice

The next narrow slice should be the first real v2 write path:

1. define a minimal builder-side v2 tuple assembly seam
2. write grouped hot tuples plus rerank tuples into the new page-chain helpers
3. keep scan/runtime unchanged until the write path is proven
