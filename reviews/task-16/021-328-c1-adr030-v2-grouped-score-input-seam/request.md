# Review Request: C1 ADR-030 V2 Grouped Score-Input Seam

## Context

Packet `327` extended `CachedGraphElement` so grouped-v2 hot tuples keep their grouped search-code
bytes and rerank tuple TID in cache instead of discarding them after tuple decode.

That still left one missing boundary: there was no single typed score-input view over those cached
grouped payloads. Future grouped scoring would still need to manually pull fields out of the cache
entry shape.

## Problem

The grouped scorer path needs a stable input seam that says, in effect:

1. here is the grouped search code
2. here is the cached binary sidecar
3. here is the cold rerank tuple pointer

Without that seam, the first grouped scorer would still be coupled to the cache struct layout.

We need one narrow packet that defines the grouped score-input carrier, but does not enable grouped
runtime yet.

## Planned Slice

Add a typed grouped score-input view over cached grouped hot payloads:

1. grouped cache entries can expose a grouped score input
2. scalar cache entries cannot
3. runtime behavior remains unchanged; grouped scans still stop at the existing unsupported-runtime
   gate

This still excludes:

- no grouped-v2 traversal enablement
- no grouped approximate scorer
- no rerank fetch path
- no change to the grouped-v2 runtime rejection behavior

## Implementation

Updated `src/am/scan.rs`:

1. added `GroupedScoreInput<'a>` with:
   - `reranktid`
   - `search_code`
   - `binary_words`
2. added `CachedGraphElement::grouped_score_input()`:
   - returns `Some(GroupedScoreInput)` only when grouped hot payloads are present
   - returns `None` for scalar cache entries
3. kept all scan/runtime behavior unchanged; this is only a typed seam over already-cached data

New tests:

- `grouped_score_input_uses_cached_grouped_hot_payloads`
- strengthened `cached_graph_element_from_scalar_tuple_ref_has_no_grouped_hot_payloads` to assert
  `grouped_score_input() == None`

## Measurements

This packet is still seam work, so there are no new latency or recall measurements.

Known validation results for this attempt:

- focused validation:
  - `cargo test grouped_score_input_uses_cached_grouped_hot_payloads --lib`: passed
  - `cargo test cached_graph_element_from_scalar_tuple_ref_has_no_grouped_hot_payloads --lib`: passed
- full checkpoint:
  - `cargo test`: passed
  - first `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17` rerun:
    - hit the recurring flaky test `pg_test_tqhnsw_debug_reachable_live_count_matches_admin_snapshot`
  - isolated rerun:
    - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17 test_tqhnsw_debug_reachable_live_count_matches_admin_snapshot`: passed
  - clean full rerun:
    - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

ADR-030 v2 now has a typed grouped score-input seam derived entirely from cached grouped hot
payloads.

What this de-risks:

1. the future grouped scorer can depend on one narrow input type instead of the whole cache entry
   layout
2. the grouped search-code carrier, binary sidecar, and cold rerank pointer now meet at one scan
   boundary
3. the next packet can start introducing grouped scorer plumbing without reshaping the cache again

## Next Slice

The next narrow slice should wire this seam into the candidate-score boundary while still refusing
grouped runtime:

1. add a grouped candidate-score dispatch seam that can accept `GroupedScoreInput`
2. keep grouped scoring behind the existing unsupported-runtime error
3. make the eventual scorer cutover a local change to the dispatch seam rather than a cache-shape
   rewrite
