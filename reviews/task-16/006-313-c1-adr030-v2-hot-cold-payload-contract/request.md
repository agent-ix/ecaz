# Review Request: C1 ADR-030 V2 Hot/Cold Payload Contract

## Context

Packet `312` established versioned metadata for ADR-030 v2:

- explicit `v1 scalar` versus `v2 grouped` format versioning
- explicit transform/search/rerank descriptors
- backward-compatible metadata-page decode for legacy indexes

The next narrow slice is to define the tuple-level payload contract for the intended ADR-030 query
pipeline:

1. binary prefilter
2. grouped FastScan scorer
3. tiny rerank

## Problem

The metadata page can now describe a grouped-format index, but the page codec still only has the
current scalar element tuple layout.

That leaves the v2 format underspecified at the tuple boundary:

1. there is no explicit hot tuple shape for `binary + grouped search code`
2. there is no explicit cold rerank payload shape
3. page-codec tests cannot lock the intended hot/cold split before builder and scan work starts

## Planned Slice

Define page-codec contracts for a future grouped v2 layout without wiring them into runtime code:

1. a hot tuple carrying:
   - duplicate heap tids
   - graph neighbor tuple ref
   - cold rerank tuple ref
   - binary sidecar words
   - grouped search code
2. a cold rerank tuple carrying:
   - gamma
   - rerank payload bytes
3. roundtrip tests that lock the tuple shapes

This slice intentionally excludes:

- no builder writes for v2 tuples yet
- no scan reads for v2 tuples yet
- no page allocator changes yet

## Implementation

Added explicit page-codec tuple contracts for the intended ADR-030 v2 hot/cold payload split.

New tuple tags:

- `TQ_GROUPED_HOT_TAG`
- `TQ_RERANK_TAG`

New tuple shapes:

1. `TqGroupedHotTuple`
   - `level`
   - `deleted`
   - inline duplicate heap tids
   - `neighbortid`
   - `reranktid`
   - binary sidecar words
   - grouped search-code bytes
2. `TqRerankTuple`
   - `gamma`
   - rerank payload bytes

Both tuples now have:

- owned encode/decode types
- borrowed decode views
- explicit encoded-length helpers

The hot tuple is intentionally shaped around the expected runtime pipeline:

1. graph traversal and duplicate draining stay in the hot tuple
2. binary prefilter inputs stay hot
3. grouped search-code bytes stay hot
4. higher-fidelity rerank payload stays cold behind `reranktid`

Tests added:

- grouped hot tuple roundtrip
- grouped hot tuple borrowed-view coverage
- rerank tuple roundtrip
- miri grouped hot tuple roundtrip
- miri rerank tuple roundtrip

## Measurements

This packet is a page-codec contract slice, so there are no new recall or latency measurements.

Known validation results for this attempt:

- `cargo test grouped_hot_tuple_roundtrip --lib`: passed
- `cargo test rerank_tuple_roundtrip --lib`: passed
- `cargo clippy --lib --tests -- -D warnings`: passed
- `cargo test`: passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17` run 1: one-off failure in
  `pg_test_tqhnsw_frontier_head_refills_from_consumed_neighbors`
- isolated rerun of `tests::pg_test_tqhnsw_frontier_head_refills_from_consumed_neighbors`: passed
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17` run 2: passed

## Outcome

This packet does not build the v2 index yet, but it does lock the tuple contract that an efficient
rebuild can target.

What this de-risks:

1. the hot path now has an explicit storage shape instead of only prose in ADR-030
2. the rerank payload is explicitly separated from the hot scan payload
3. page-codec tests will catch accidental drift before builder and scan wiring begins

What remains:

1. actual v2 page placement and allocation rules
2. builder support for writing grouped hot tuples and rerank tuples
3. scan support for reading the new hot tuple and optional rerank fetches

## Next Slice

The next narrow slice should move from tuple contract to page-placement contract:

1. define how hot tuples and rerank tuples are placed and linked on data pages
2. add page-chain helpers for inserting and reading the new tuple kinds
3. keep the slice codec/allocation-only without introducing a new builder or scan path yet
