# Review Request: C1 Task16 TurboQuant V3 Page Layout Groundwork

Current head at execution: `0ab91db`

## Context

Packet `426` established the task-16 decision point:

- quantized deferred rerank closes turboquant latency but misses the recall
  target on the isolated source-backed `50k, m=16, ef=128` lane
- heap-f32 rerank preserves recall but stays too slow

That leaves lever `3` as the justified next step: a turboquant hot/cold payload
split that can preserve the recall-restoring rerank path while shrinking the
graph-hot working set.

This packet does not wire lever `3` into build, scan, insert, or vacuum yet.
It lands the dormant page-layout substrate first so the on-disk format work is
reviewable on its own.

## What Landed

### `src/am/page.rs`

- added `INDEX_FORMAT_V3_TURBO_HOT_COLD`
- taught metadata decode and graph-storage classification to accept V3 as a
  turboquant format
- added `MetadataPage::current_v3_turbo_hot_cold(...)` for the eventual format
  bump
- added `TQ_TURBO_HOT_TAG`
- added `TqTurboHotTuple` / `TqTurboHotTupleRef`:
  - inline heap TIDs
  - neighbor TID
  - cold rerank TID
  - optional binary-sign sidecar words
- added `DataPage` and `DataPageChain` helpers to insert, read, and update
  turbo-hot tuples

## Test Coverage

Low-level coverage now includes:

- V3 metadata graph-storage classification and roundtrip
- turbo-hot tuple encode/decode roundtrip
- borrowed turbo-hot tuple ref accessors
- single-page turbo-hot tuple page roundtrip
- multi-page chain extension for turbo-hot tuples
- Miri-style turbo-hot tuple roundtrip coverage

## Validation

Green on this head:

- `cargo test`
- `bash scripts/run_pgrx_pg17_test.sh`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Readout

This packet is intentionally dormant.

- no build path writes V3 yet
- no runtime path reads turboquant through the new hot/cold tuple
- no measurement claims are attached to this packet

The purpose is to land the smallest defensible on-disk substrate for lever `3`
before touching runtime behavior. The next slice should wire this layout into
the turboquant format adapters and only then re-run the isolated serious-lane
measurement.
