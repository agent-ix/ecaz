# Review Request: C1 Scan Runtime Profiling

## Context

Prior C1 packets:

- `review/247-c1-real-corpus-latency-10k-verified-run/request.md`
- `review/248-c1-ordered-scan-runtime-fix/request.md`
- `review/249-c1-per-cell-planner-verification/request.md`
- `review/250-c1-ef200-planner-cost-crossover/request.md`

Current verified baseline:

- shared canonical `m=8` surface is complete and valid for
  `ef_search=40,64,100,128,160,200`
- isolated one-index `m=16` surface is complete and valid for the same cells
- `m=16` is not a clear latency win over `m=8`

Representative means from the current C1 surface:

```text
m=8  ef_search=40   mean=140.982ms
m=8  ef_search=64   mean=189.651ms
m=8  ef_search=100  mean=262.439ms
m=8  ef_search=128  mean=320.426ms
m=8  ef_search=160  mean=384.200ms
m=8  ef_search=200  mean=454.021ms

m=16 ef_search=40   mean=148.659ms
m=16 ef_search=64   mean=194.966ms
m=16 ef_search=100  mean=263.784ms
m=16 ef_search=128  mean=317.408ms
m=16 ef_search=160  mean=377.647ms
m=16 ef_search=200  mean=457.512ms
```

So the next C1 gap is not planner routing anymore. It is raw ordered-scan
runtime.

## Problem

The current scan path records `TqExplainCounters`, but on PG17 that surface is
still internal-only. There is no durable per-query profile yet that tells us
whether the current latency is dominated by:

- upper-layer descent / layer-0 seed search work in `amrescan`
- graph tuple reads and decode work
- scoring work
- frontier / beam bookkeeping
- fallback linear scan, if it is still being reached unexpectedly

Without that readout, choosing the first optimization checkpoint would be guessy.

## Static hotspot hypotheses

From the current `src/am/scan.rs` and `src/am/graph.rs` implementation:

1. `graph::read_page_tuple_bytes()` allocates and copies a fresh `Vec<u8>` for
   every graph tuple read.
2. `load_graph_element()` / `load_graph_neighbors()` decode into owned `Vec`
   payloads for every visited candidate and adjacency list.
3. `initialize_scan_entry_candidate()` does real search work during
   `amrescan`, so query latency may be front-loaded before the first tuple is
   emitted.
4. Beam/frontier bookkeeping currently uses `HashSet`, `BinaryHeap`, and
   `Vec::remove()` in the hot path; those may or may not matter compared to the
   page-read / decode cost.

These are hypotheses only. The next slice needs measured counters and timing.

## Planned work

1. Add a PG-test debug helper that runs one ordered tqhnsw scan and returns:
   - existing explain-style counters
   - result count
   - top-level timing buckets for rescan/setup vs tuple emission
   - any additional scan-local timing needed to distinguish graph reads,
     decode, and scoring
2. Run that helper against the real `10k` C1 fixture on the scratch cluster,
   starting with the current operating point:
   - canonical `tqhnsw_real_10k_m8_idx`
   - `ef_search=40`
   - representative real query rows from `tqhnsw_real_10k_queries`
3. Use the measured hotspot to choose the first optimization checkpoint rather
   than changing the runtime path blindly.

## Exit criteria for this slice

- a pushed checkpoint exposes a concrete per-query profile surface for the
  current ordered tqhnsw scan path
- the packet records at least one real `10k` profile capture
- the next optimization target is justified by measured counters/timing, not
  only by static inspection

## Run Update: 2026-04-11

The first profiling checkpoint landed a PG-test debug helper:

- `tests.tqhnsw_debug_scan_profile(index_oid, query real[])`

It runs one ordered tqhnsw scan and returns:

- explicit timing around `amrescan`
- explicit timing around the `amgettuple` emission loop
- the existing explain-style counters after rescan and at full exhaustion
- basic scan-state shape (phase, staged slots, visited/emitted counts)

Checkpoint validation was green:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Profile capture: real `10k`, `m=8`, `ef_search=40`

Warm-cache sample over the first five real query rows from
`tqhnsw_real_10k_queries` against `tqhnsw_real_10k_m8_idx`:

```text
id=10000 rescan=18.350ms emit=0.040ms total=18.396ms
id=10001 rescan= 9.297ms emit=0.033ms total= 9.335ms
id=10002 rescan= 5.129ms emit=0.033ms total= 5.167ms
id=10003 rescan= 4.407ms emit=0.030ms total= 4.442ms
id=10004 rescan= 3.490ms emit=0.030ms total= 3.524ms
```

Common shape across those samples:

- `rescan_phase = graph_traversal`
- `rescan_current_result = true`
- `final_phase = exhausted`
- `total_linear_pages_read = 0`
- `total_bootstrap_expansions = 40`
- `total_bootstrap_pages_read = 40`
- `total_elements_scored = 40`
- `total_heap_tids_returned = 40`

So the ordered tqhnsw runtime itself is strongly front-loaded in `amrescan`.
Tuple emission is negligible once the staged result window exists.

## Paired SQL plan readout

To compare the AM-local profile against actual SQL execution, the same real
query shape was checked with `EXPLAIN (ANALYZE, BUFFERS)`:

### `ef_search=40`

```text
Index Scan using tqhnsw_real_10k_m8_idx on tqhnsw_real_10k_corpus
  actual time=5.855..6.710 rows=10
  Buffers: shared hit=1505
Execution Time: 6.804 ms
```

### `ef_search=200`

```text
Index Scan using tqhnsw_real_10k_m8_idx on tqhnsw_real_10k_corpus
  actual time=13.170..13.651 rows=10
  Buffers: shared hit=6167
Execution Time: 13.749 ms
```

## Current read

Two things are clear now:

1. Warm-cache SQL latency is already single-digit milliseconds at
   `ef_search=40`, so the earlier `~141ms` C1 benchmark result is primarily a
   cold-cache surface, not a warm executor hot path.
2. The dominant remaining issue is page-touch volume during graph search.
   The helper's current counters only see emitted-candidate materialization, but
   `EXPLAIN (BUFFERS)` shows the actual index scan touches far more shared
   buffers than those counters report:
   - about `1505` shared-buffer hits at `ef_search=40`
   - about `6167` shared-buffer hits at `ef_search=200`

That means the next optimization target should not be tuple emission. It should
reduce repeated graph page access during rescan/search, most likely through
scan-local graph element / adjacency reuse or another mechanism that lowers the
number of element and neighbor page touches per query.
