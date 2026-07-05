# Review Request: C1 Scan Fast Hash State

## Context

Packet `254` landed the scan-local score cache and materially improved the real
`10k` latency surface. Packet `247` now records the repaired score-cache rerun:

- canonical `m=8, ef_search=40`: mean `89.089ms`
- canonical `m=8, ef_search=200`: mean `173.680ms`

That is a real win, but it still misses `NFR-001` badly.

The post-score-cache hot-path probe on the same representative real query
(`id=10000`) now shows a different bottleneck shape:

- `ef_search=40`
  - upper-layer seed elapsed: `24.560ms`
  - layer-0 seed elapsed: `14.301ms`
  - candidate scoring elapsed: `31.711ms`
- `ef_search=200`
  - upper-layer seed elapsed: `65.931ms`
  - layer-0 seed elapsed: `69.229ms`
  - candidate scoring elapsed: `110.260ms`

So traversal bookkeeping is now in the same band as scoring, and the hot path
is still heavily keyed by `ItemPointer` lookups:

- scan-local visited / expanded / emitted sets
- scan-local graph and score caches
- beam-search visited sets during seed search

## Problem

The ordered scan hot path still uses `std` `HashMap` / `HashSet` with their
default hasher across the main `ItemPointer` bookkeeping surfaces. After the
score-cache win, those structures are more likely to matter:

- `search_layer0_result_candidates_with_successors(...)` constructs a fresh
  visited set during rescan seeding
- `BeamSearch` tracks visited nodes during traversal and refill work
- scan-local caches and state sets are consulted repeatedly during the same
  query

This is a narrow, plausible next optimization target that does not change the
search algorithm or planner behavior.

## Planned work

1. Switch the scan/search hot-path `ItemPointer` map/set structures to a faster
   hash implementation.
2. Keep the slice narrow:
   - no algorithm rewrite
   - no planner changes
   - no harness changes
3. Re-run the hot-path probe and representative real-fixture latency surface to
   verify whether the bookkeeping cost drops materially.

## Exit criteria

- a pushed checkpoint narrows the hash-heavy scan/search bookkeeping cost on
  the real C1 path
- validation is green (`cargo test`, `cargo pgrx test pg17`, clippy)
- this packet records measured before/after evidence, not just the code change

## Checkpoint

Switched the hot scan/search bookkeeping state to `hashbrown`:

- scan-local graph and score caches in `scan.rs`
- scan-local visited / expanded / emitted sets in `scan.rs`
- beam-search visited sets in `search.rs`
- layer-search visited sets in `graph.rs`

This keeps the slice narrow:

- no planner changes
- no traversal algorithm rewrite
- no harness changes

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All green on this checkpoint.

## Real fixture readout

Representative hot-path probe on the same real `10k` query (`id=10000`,
`tqhnsw_real_10k_m8_idx`):

### `ef_search=40`

Before this slice:

```text
rescan_upper_layer_seed_elapsed_us = 24560
rescan_layer0_seed_elapsed_us      = 14301
graph_element_load_elapsed_us      =  2791
graph_neighbor_load_elapsed_us     =   912
candidate_score_elapsed_us         = 31711
```

After this slice:

```text
rescan_upper_layer_seed_elapsed_us = 24271
rescan_layer0_seed_elapsed_us      = 14207
graph_element_load_elapsed_us      =  2837
graph_neighbor_load_elapsed_us     =   932
candidate_score_elapsed_us         = 31530
```

### `ef_search=200`

Before this slice:

```text
rescan_upper_layer_seed_elapsed_us = 65931
rescan_layer0_seed_elapsed_us      = 69229
graph_element_load_elapsed_us      =  8198
graph_neighbor_load_elapsed_us     =  3782
candidate_score_elapsed_us         = 110260
```

After this slice:

```text
rescan_upper_layer_seed_elapsed_us = 61339
rescan_layer0_seed_elapsed_us      = 66907
graph_element_load_elapsed_us      =  7318
graph_neighbor_load_elapsed_us     =  3464
candidate_score_elapsed_us         = 105936
```

That is a real but modest improvement:

- `ef_search=40`: essentially flat within noise
- `ef_search=200`: upper-layer seed time improved by about `7%`, layer-0 seed
  time by about `3%`, and scoring time by about `4%`

### Representative SQL `EXPLAIN (ANALYZE, BUFFERS)` readout

On the same representative ordered query:

```text
ef_search=40:
  before: Buffers shared hit=668,  Execution Time=95.759 ms
  after:  Buffers shared hit=668,  Execution Time=80.683 ms

ef_search=200:
  before: Buffers shared hit=2141, Execution Time=186.563 ms
  after:  Buffers shared hit=2141, Execution Time=172.001 ms
```

The buffer footprint did not change; the gain came from cheaper in-memory
bookkeeping.

### Canonical `m=8` verified sweep

Command used:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
    --prefix tqhnsw_real_10k \
    --m 8 \
    --ef-search 40,64,100,128,160,200 \
    --cache-state cold \
    --output /tmp/nfr1_real_10k_m8_fasthash.summary
```

Completed cells:

```text
m=8   ef_search=40   n=200   p50=87.770ms p95=100.300ms p99=107.971ms mean=88.360ms min=76.414ms max=110.384ms server_qps=11.32 wall=19.81s
m=8   ef_search=64   n=200   p50=104.465ms p95=115.149ms p99=118.887ms mean=103.883ms min=88.734ms max=120.511ms server_qps=9.63  wall=21.52s
m=8   ef_search=100  n=200   p50=125.603ms p95=139.825ms p99=148.885ms mean=125.027ms min=104.615ms max=155.138ms server_qps=8.00  wall=25.75s
m=8   ef_search=128  n=200   p50=140.254ms p95=157.450ms p99=167.119ms mean=139.747ms min=115.337ms max=172.427ms server_qps=7.16  wall=28.70s
m=8   ef_search=160  n=200   p50=154.170ms p95=175.365ms p99=187.590ms mean=153.566ms min=127.538ms max=194.312ms server_qps=6.51  wall=31.45s
m=8   ef_search=200  n=200   p50=174.103ms p95=204.769ms p99=220.001ms mean=174.147ms min=138.656ms max=225.509ms server_qps=5.74  wall=35.62s
```

Compared to packet `247`'s post-score-cache `m=8` surface:

- `ef_search=40`: mean `89.089ms -> 88.360ms`
- `ef_search=64`: mean `106.258ms -> 103.883ms`
- `ef_search=100`: mean `125.833ms -> 125.027ms`
- `ef_search=128`: mean `141.723ms -> 139.747ms`
- `ef_search=160`: mean `156.208ms -> 153.566ms`
- `ef_search=200`: mean `173.680ms -> 174.147ms`

## Conclusion

This slice is worth keeping, but it is not the next big unlock.

- The change is low-risk and modestly improves the canonical `m=8` surface for
  most cells.
- The gain is much smaller than the score-cache win.
- The remaining C1 gap is still dominated by traversal/search work, not by the
  default hasher alone.

So the next slice should target the traversal algorithm or a finer-grained
breakdown inside upper-layer / layer-0 seed search, rather than more generic
map/set tuning.
