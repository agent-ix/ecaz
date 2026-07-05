# Review Request: C1 ADR-030 V2 Grouped Hot Payload Cache Seam

## Context

Packet `326` made scan loaded-state bookkeeping distinguish `ExactUnavailable` from `None`, so a
live grouped-v2 tuple no longer looks like “no score state loaded yet.”

The next narrow scan-side seam is to preserve grouped hot payloads explicitly in the cache entry
itself. Until now, the cache kept grouped tuple headers and binary sidecars, but it discarded the
other hot-v2 payloads:

1. grouped search code bytes
2. rerank tuple TID

That means future grouped score or rerank work would still need to rediscover those payloads from
disk instead of carrying them through the cache boundary.

## Problem

Even after packets `324`-`326`, the scan cache still treated grouped-v2 tuples as if only their
header and optional binary sidecar mattered.

That leaves two structural gaps:

1. there is no explicit cache-level carrier for grouped search-code bytes
2. there is no cache-level carrier for the cold rerank tuple pointer

Without that seam, the first grouped scorer would have to punch back through tuple decoding instead
of reusing the cached graph element shape.

## Planned Slice

Extend `CachedGraphElement` so it preserves the full grouped hot payload boundary:

1. scalar tuples still cache no grouped payloads
2. grouped tuples cache:
   - grouped search code bytes
   - rerank tuple TID
3. runtime behavior stays unchanged; this is a cache-shape seam only

This still excludes:

- no grouped-v2 traversal enablement
- no grouped approximate scorer
- no rerank fetch/scoring path
- no change to the grouped-v2 runtime rejection

## Implementation

Updated `src/am/graph.rs`:

1. added `GraphTupleRef::reranktid()`
2. added `GraphTupleRef::grouped_search_code()`

Updated `src/am/scan.rs`:

1. extended `CachedGraphElement` with:
   - `reranktid: Option<ItemPointer>`
   - `grouped_search_code: CachedGroupedSearchCode`
2. added `CachedGroupedSearchCode` as an explicit cache carrier for grouped hot search-code bytes
3. changed `CachedGraphElement::from_graph_tuple_ref(...)` to preserve grouped-v2 hot payloads from
   the typed tuple ref
4. left all runtime scoring behavior unchanged; the new fields are purely a cache-boundary seam

New tests:

- `cached_graph_element_from_grouped_tuple_ref_keeps_grouped_hot_payloads`
- `cached_graph_element_from_scalar_tuple_ref_has_no_grouped_hot_payloads`

## Measurements

This packet is still storage/runtime seam work, so there are no new latency or recall measurements.

Known validation results for this attempt:

- focused validation:
  - `cargo test cached_graph_element_from_grouped_tuple_ref_keeps_grouped_hot_payloads --lib`: passed
  - `cargo test cached_graph_element_from_scalar_tuple_ref_has_no_grouped_hot_payloads --lib`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- full checkpoint:
  - `cargo test`: passed
  - first `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17` run:
    - hit one failure in `pg_test_tqhnsw_debug_reachable_live_count_matches_admin_snapshot`
    - isolated rerun of that test passed
    - treated as suite flake, not a slice-local regression
  - clean reruns:
    - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17 test_tqhnsw_debug_reachable_live_count_matches_admin_snapshot`: passed
    - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
  - final `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

ADR-030 v2 scan cache entries now preserve the full grouped hot payload boundary instead of only the
header and binary sidecar.

What this de-risks:

1. the first grouped approximate scorer can read grouped search-code bytes from cached graph
   elements instead of reopening tuple decode paths
2. future rerank wiring already has the cold rerank tuple pointer available in the cache shape
3. scan-side grouped work can keep building on one typed cache representation instead of layering
   more grouped special-cases onto scalar assumptions

## Next Slice

The next narrow slice should introduce the first grouped score input seam without enabling grouped
runtime yet:

1. add a grouped score-input carrier that can be derived from cached grouped hot payloads
2. gate grouped score requests behind the existing unsupported-runtime path
3. make the scorer entry boundary explicit enough that a later packet can plug in grouped LUT
   scoring without reshaping the cache again
