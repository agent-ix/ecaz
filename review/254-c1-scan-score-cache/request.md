# Review Request: C1 Scan Score Cache

## Context

Packet `253` established that the remaining ordered-scan cost is dominated by
candidate scoring, not tuple reads.

Representative real `10k` hot-path profile for query `id=10000`, `m=8`:

- `ef_search=40`
  - `candidate_score_calls = 1125`
  - `candidate_score_elapsed_us = 76477`
  - graph tuple load time stayed around `4ms`
- `ef_search=200`
  - `candidate_score_calls = 4915`
  - `candidate_score_elapsed_us = 360059`
  - graph tuple load time stayed around `15ms`

The same profile also showed substantial element reuse:

- `graph_element_cache_hits = 804` at `ef_search=40`
- `graph_element_cache_hits = 3926` at `ef_search=200`

So repeated visits during one ordered scan are still paying the same
query/code score cost over and over.

## Problem

The current ordered scan has a scan-local graph tuple cache, but not a
scan-local score cache. Repeated visits to the same element TID still rerun
`score_scan_element_result(...)` against the same query and code bytes.

That is now the highest-signal remaining C1 target.

## Planned work

1. Add a scan-local score cache keyed by element TID for the ordered scan.
2. Route the graph search path through that cache so repeated element visits do
   not rescore identical query/code pairs within one scan.
3. Re-run the hot-path profile from packet `253` plus representative SQL probes
   to confirm:
   - score-call count drops materially
   - scoring time drops materially
   - wall-clock latency moves in the right direction
4. Keep the slice narrow:
   - no planner changes
   - no harness changes
   - no speculative graph algorithm rewrite

## Exit criteria

- a pushed checkpoint materially reduces repeated scoring work on the real C1
  ordered scan path
- validation is green (`cargo test`, `cargo pgrx test pg17`, clippy)
- the packet records before/after hot-path evidence, not just code intent

## Checkpoint

Implemented a scan-local score cache for the ordered scan path:

- `TqScanOpaque` now owns a scan-lifetime score cache keyed by element TID
- `amrescan` resets that cache and `amendscan` frees it
- ordered entry seeding and layer-0 successor expansion now route score
  requests through the cache instead of unconditionally rescoring the same
  query/code pair
- the hot-path debug surface now reports score-cache hit/miss counters so the
  reuse claim is measurable instead of inferred

Also widened the existing `reset_scan_position` cache test so it proves the new
score cache is cleared between scans.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All green on this checkpoint. The first attempt at the checkpoint suite collided
with a concurrent `cargo pgrx test` Postgres harness; rerunning the required
suite sequentially was green.

## Real fixture readout

Scratch helper refreshed against the current installed library so the SQL
surface exposes the new score-cache columns.

Representative probe on the real `10k` fixture, `m=8`, query `id=10000`:

### `ef_search=40`

Before packet `253`:

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

After this checkpoint:

```text
rescan_upper_layer_seed_elapsed_us =  27501
rescan_layer0_seed_elapsed_us      =  15211
graph_element_cache_hits           =    804
graph_element_cache_misses         =    470
graph_element_load_elapsed_us      =   4920
graph_neighbor_cache_hits          =     23
graph_neighbor_cache_misses        =    124
graph_neighbor_load_elapsed_us     =   1106
candidate_score_calls              =    470
candidate_score_elapsed_us         =  32876
score_cache_hits                   =    656
score_cache_misses                 =    470
```

### `ef_search=200`

Before packet `253`:

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

After this checkpoint:

```text
rescan_upper_layer_seed_elapsed_us =  62864
rescan_layer0_seed_elapsed_us      =  73061
graph_element_cache_hits           =   3926
graph_element_cache_misses         =   1574
graph_element_load_elapsed_us      =  10217
graph_neighbor_cache_hits          =     90
graph_neighbor_cache_misses        =    493
graph_neighbor_load_elapsed_us     =   3629
candidate_score_calls              =   1574
candidate_score_elapsed_us         = 108532
score_cache_hits                   =   3342
score_cache_misses                 =   1574
```

The scoring work dropped materially on the exact same real query:

- `ef_search=40`: score calls `1125 -> 470`, score time `76.5ms -> 32.9ms`
- `ef_search=200`: score calls `4915 -> 1574`, score time `360.1ms -> 108.5ms`

### SQL `EXPLAIN (ANALYZE, BUFFERS)` readout

Compared to packet `252`'s representative readout on the same lane:

```text
ef_search=40:
  before: Buffers shared hit=668,  Execution Time=126.200 ms
  after:  Buffers shared hit=668,  Execution Time= 95.759 ms

ef_search=200:
  before: Buffers shared hit=2141, Execution Time=418.902 ms
  after:  Buffers shared hit=2141, Execution Time=186.563 ms
```

So this slice improved wall-clock latency without needing another buffer-hit
reduction; the gain came from avoiding repeated rescoring on already-visited
elements.

## Current conclusion

The score cache is a real C1 win:

- it preserves the same graph traversal breadth on the real fixture
- it turns heavy element reuse into concrete score-cache hits
- it materially reduces scoring time
- it materially improves representative end-to-end SQL latency, especially at
  higher `ef_search`

The next C1 step should be to rerun the verified real-corpus latency surface on
top of this checkpoint and then decide whether another optimization slice is
still needed.
