# Review Request: C1 Scan CPU Hot-Path Breakdown

## Context

Packet `252` landed a scan-local graph read cache in the ordered scan path and
reduced shared-buffer hits on the real `10k` scratch probe:

- `ef_search=40`: about `1505 -> 668` shared hits
- `ef_search=200`: about `6167 -> 2141` shared hits

That change was validated and pushed. However, the same scratch rerun still
shows large wall-clock cost:

- `ef_search=40`: about `126ms`
- `ef_search=200`: about `419ms`

So page rereads were not the whole C1 bottleneck.

## Problem

The remaining ordered-scan cost is now more likely CPU-side work in the scan
runtime itself:

- repeated graph traversal bookkeeping
- tuple decode / neighbor slicing
- scoring over candidate codes
- candidate frontier maintenance

The current explain counters do not break those costs down tightly enough to
justify the next optimization.

## Planned work

1. Add a narrow profiling seam for the ordered scan hot path, focused on CPU
   work rather than shared-buffer churn.
2. Measure where rescan time is going across:
   - candidate expansion
   - tuple decode / adjacency extraction
   - scoring
   - frontier/result maintenance
3. Use that evidence to pick the next optimization slice instead of guessing.
4. Keep the slice measurement-first:
   - no planner changes
   - no benchmark harness changes
   - no speculative algorithm rewrite without a measured target

## Exit criteria

- a pushed checkpoint records a concrete CPU-side breakdown for the real C1
  ordered scan path
- validation is green (`cargo test`, `cargo pgrx test pg17`, clippy) if code
  changes are introduced
- the packet names the next optimization target using measured evidence

## Checkpoint

Added a new ordered-scan hot-path profiling seam:

- scan-local timing buckets on the ordered rescan path for:
  - upper-layer seed search
  - layer-0 result search
  - graph element cache hit/miss/load time
  - graph neighbor cache hit/miss/load time
  - candidate scoring count/time
- a new debug SQL surface:
  `tests.tqhnsw_debug_scan_hot_path_profile(index_oid, query real[])`
- coverage updates so the existing profile regression proves the new counters
  are populated on a non-empty fixture

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All green on this checkpoint.

## Real fixture readout

Scratch helper registered against the current installed library:

```text
CREATE OR REPLACE FUNCTION tests.tqhnsw_debug_scan_hot_path_profile(...)
```

Representative probe on the real `10k` fixture, `m=8`, query `id=10000`:

### `ef_search=40`

```text
rescan_upper_layer_seed_elapsed_us =  57287
rescan_layer0_seed_elapsed_us      =  26526
graph_element_cache_hits           =    804
graph_element_cache_misses         =    470
graph_element_load_elapsed_us      =   3052
graph_neighbor_cache_hits          =     23
graph_neighbor_cache_misses        =    124
graph_neighbor_load_elapsed_us     =    995
candidate_score_calls              =   1125
candidate_score_elapsed_us         =  76477
```

### `ef_search=200`

```text
rescan_upper_layer_seed_elapsed_us = 242862
rescan_layer0_seed_elapsed_us      = 147571
graph_element_cache_hits           =   3926
graph_element_cache_misses         =   1574
graph_element_load_elapsed_us      =  10438
graph_neighbor_cache_hits          =     90
graph_neighbor_cache_misses        =    493
graph_neighbor_load_elapsed_us     =   4588
candidate_score_calls              =   4915
candidate_score_elapsed_us         = 360059
```

## Conclusion

The remaining C1 runtime cost is not dominated by tuple reads anymore.

The profile shows:

- graph tuple load time is tiny relative to total rescan time
  - about `4.0ms` total load time at `ef_search=40`
  - about `15.0ms` total load time at `ef_search=200`
- candidate scoring dominates the hot path
  - about `76.5ms` at `ef_search=40`
  - about `360.1ms` at `ef_search=200`
- there is heavy reuse opportunity
  - `804` graph element cache hits at `ef_search=40`
  - `3926` graph element cache hits at `ef_search=200`

So the next optimization target should be **scan-local score reuse**, not more
graph tuple caching. The obvious next slice is a score cache keyed by element
TID (or equivalent reuse seam) so repeated element visits stop recomputing the
same query/code score during one ordered scan.
