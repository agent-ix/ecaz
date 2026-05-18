# Review Request: C1 Scan Graph Read Cache

## Context

Packet `251` established the first real C1 runtime profile:

- warm-cache SQL execution at `m=8, ef_search=40` is already about `6.8ms`
- the ordered scan remains front-loaded in `amrescan`
- tuple emission is negligible compared to rescan/setup
- the real query shape still touches far more shared buffers than the current
  element-only counters report:
  - about `1505` shared-buffer hits at `ef_search=40`
  - about `6167` shared-buffer hits at `ef_search=200`

So the next optimization target is page-touch volume during graph search, not
the visible tuple-emission path.

## Problem

The current graph read surface repeatedly calls:

- `graph::load_graph_element(...)`
- `graph::load_graph_neighbors(...)`
- `graph::load_graph_adjacency(...)`

Those helpers read and decode tuples afresh each time. During one ordered scan,
the same element and neighbor tuples can be revisited multiple times across:

- upper-layer descent
- layer-0 seed search
- later frontier/result materialization

That repeated tuple reread / redecode pattern is the leading C1 suspect after
packet `251`.

## Planned work

1. Add a scan-local cache for graph reads in `TqScanOpaque`.
2. Route ordered scan search/materialization through cached graph-element /
   adjacency access instead of unconditional rereads.
3. Keep the slice narrow:
   - no planner changes
   - no benchmark harness changes
   - no speculative executor changes outside the graph read path
4. Re-run the profile helper plus representative `EXPLAIN (ANALYZE, BUFFERS)`
   probes to verify buffer-hit reduction before claiming improvement.

## Exit criteria

- a pushed checkpoint reduces repeated graph page access on the current real
  `10k` C1 path
- validation is green (`cargo test`, `cargo pgrx test pg17`, clippy)
- this packet records before/after profile evidence, not just code intent

## Checkpoint

Implemented the scan-local graph read cache in the ordered scan path:

- `TqScanOpaque` now owns scan-lifetime caches for decoded graph elements and
  neighbor tuples
- `amrescan` resets those caches and `amendscan` frees them
- ordered entry seeding now reuses cached graph reads across:
  - upper-layer seed search
  - layer-0 result search
  - later result materialization
- `materialize_graph_result_candidate` now reads the selected element from the
  scan-local cache instead of unconditionally rereading it from the index
- `graph.rs` only widened two helper seams so `scan.rs` could reuse the
  existing result-window search logic instead of forking another traversal loop

Added a unit test in `scan.rs` that proves `reset_scan_position` clears the
new scan-local graph caches before the next rescan.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All green on this checkpoint.

## Profile rerun

Representative rerun against the existing real `10k` scratch fixture using the
same `tqhnsw_real_10k_m8_idx` path and query `id=10000`:

### Direct AM helper

`tests.tqhnsw_debug_scan_profile(...)` after setting the correct runtime GUC
(`SET tqhnsw.ef_search = ...`):

```text
ef_search=40  rescan=128.581ms total=128.847ms total_bootstrap_expansions=40  total_bootstrap_pages_read=40  total_heap_tids_returned=40
ef_search=200 rescan=407.032ms total=409.877ms total_bootstrap_expansions=200 total_bootstrap_pages_read=200 total_heap_tids_returned=200
```

That confirms the cache slice preserved the real runtime breadth surface; it
did not collapse `ef_search`.

### SQL `EXPLAIN (ANALYZE, BUFFERS)` readout

Current scratch rerun:

```text
ef_search=40:
  Index Scan using tqhnsw_real_10k_m8_idx
  Buffers: shared hit=668
  Execution Time: 126.200 ms

ef_search=200:
  Index Scan using tqhnsw_real_10k_m8_idx
  Buffers: shared hit=2141
  Execution Time: 418.902 ms
```

Compared to packet `251`'s earlier representative readout on the same lane:

- `ef_search=40`: `shared hit` dropped from about `1505` to `668`
- `ef_search=200`: `shared hit` dropped from about `6167` to `2141`

So this slice did materially reduce repeated graph-page access.

## Current conclusion

The cache checkpoint achieved the narrow page-touch goal, but it did **not**
translate into a proportional wall-clock improvement on the current scratch
state. In the current environment, ordered SQL remains about `126ms` at
`ef_search=40`, which is still in the same general band as the real C1
benchmark surface.

That means page rereads were only part of the cost. The next bottleneck is
likely CPU-side graph traversal / decode / scoring overhead rather than raw
shared-buffer churn alone.
