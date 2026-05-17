# Review Request: C1 AM Startup Boundary Reconciliation

## Context

Packet `259` resolved the executor-vs-AM ambiguity for the representative real
`10k` probe:

- tqhnsw SQL query on `tqhnsw_real_10k_m8_idx` at `ef_search=40`
  - `Index Scan` startup: `46.187ms`
  - `Index Scan` total: `52.091ms`
- direct heap fetch of the exact top-10 TIDs chosen by tqhnsw
  - `Tid Scan` startup: `0.046ms`
  - `Tid Scan` total: `0.084ms`
  - execution time: `0.123ms`

So the missing `~40ms` is not heap/executor row fetch after tqhnsw has already
chosen rows. It remains on the tqhnsw startup side.

Packet `258` already showed that the current startup counters do not explain
that wall time:

- initialize entry: `5.630ms`
- candidate scoring: `3.476ms`
- graph element load: `0.696ms`
- graph neighbor load: `0.250ms`

Those buckets are real, but incomplete.

## Problem

The current C1 instrumentation still under-accounts for the AM startup surface.
That makes the next optimization target fuzzy even though we now know it is
inside tqhnsw.

The next slice needs to reconcile:

- real SQL `Index Scan` startup time
- total tqhnsw startup work inside `amrescan`
- the currently exposed sub-buckets

without conflating AM work with outer SQL/executor behavior.

## Revised C1 Read

This packet changed the C1 interpretation materially.

- Warm-cache representative latency on the current `10k` real-corpus build is
  now at the NFR-001 `p50 < 5ms` target surface for `m=8, ef_search=40`.
- The old packet-259 `~46ms` representative startup reading is not
  reproducible on the current warm-cache build and should be treated as a
  cold-cache or otherwise non-representative artifact.
- The remaining large gap on the verified cold-cache surface is therefore not
  a hidden tqhnsw CPU path; it is overwhelmingly the cost of buffer misses /
  page reads during graph traversal.

So the main C1 CPU optimization arc is functionally complete for the warm
surface. The next C1 question is not “where is the missing warm CPU time?” but
“how should warm and cold latency be reported separately, and is there a
worthwhile I/O-side mitigation for the cold surface?”

## Planned work

1. Add a total tqhnsw startup boundary probe around the AM startup path.
2. Split that total against the already exposed sub-buckets to locate the still
   missing internal cost.
3. Use that result to choose the next concrete C1 optimization seam instead of
   continuing with partial counters.

## In-Progress Findings

- The new debug boundary counters are now wired into the local build and scratch
  wrapper for `tests.tqhnsw_debug_scan_hot_path_profile`.
- On the representative real `10k` probe (`id=10000`, `m=8`):
  - `ef_search=40`
    - `rescan_amrescan_total_elapsed_us=3289`
    - `initialize_entry_elapsed_us=1334`
    - `layer0_seed_elapsed_us=987`
    - `graph_element_load_elapsed_us=742`
    - `candidate_score_elapsed_us=241`
  - `ef_search=200`
    - `rescan_amrescan_total_elapsed_us=17645`
    - `initialize_entry_elapsed_us=14679`
    - `layer0_seed_elapsed_us=14373`
    - `graph_element_load_elapsed_us=11633`
    - `candidate_score_elapsed_us=1006`
- A fresh direct SQL rerun on the same representative query is materially lower
  than the older packet-259 reading and is now close to the AM boundary probe:
  - `EXPLAIN (ANALYZE, FORMAT JSON)` at `ef_search=40`
    - `Index Scan` startup: `4.194ms`
    - `Index Scan` total: `4.816ms`
    - execution time: `4.947ms`
  - `EXPLAIN (ANALYZE, BUFFERS)` at `ef_search=40`
    - `Index Scan` startup: `4.081ms`
    - `Index Scan` total: `4.556ms`
    - execution time: `4.750ms`
- On the current build and warmed representative query, the gap between direct
  SQL `Index Scan` startup (`~4.1ms`) and tqhnsw `amrescan` total (`3.289ms`) is
  now only about `0.8ms` to `0.9ms`, not `~40ms`.
- A repeated plain-query wall-time sanity check stays below the direct `EXPLAIN`
  numbers:
  - 5,000 repeated plain scans of the representative query completed in
    `5.655s` inside the live `psql` session
  - that is about `1.131ms/query`
- Backend-side `perf` is now available on this machine and the first real server
  capture against repeated plain SQL scans still shows tqvector cycles dominated
  by scoring:
  - `31.78%` `tqvector::quant::prod::ProdQuantizer::score_ip_from_split_parts`
  - `4.11%` `Vec<T>::extend_from_slice`
  - `1.20%` `tqvector::am::graph::read_page_tuple_bytes`

## Current Read

On the current local build, the earlier packet-259 `~46ms` representative probe
is not reproducible. The direct SQL startup time for that same query is now
around `4.1ms`, which is already close to the new tqhnsw `amrescan` boundary
counter at `3.289ms`.

That changes the packet-260 question from “where is the missing 40ms inside
tqhnsw?” to “what accounts for the remaining sub-millisecond gap between the AM
boundary and the full SQL `Index Scan` startup on the representative query, and
was the older 46ms reading stale, cold-cache, or otherwise non-representative?”

The next step is to lock down that current measurement with a small multi-query
sample and then decide whether C1 should keep using the existing `EXPLAIN`
surface as-is or add a complementary plain-query timing harness.

## Checkpoint

- Code checkpoint: `04098c2` `debug: add AM rescan boundary timing probes`
- Validation:
  - `cargo test`
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- Packet status: open

This checkpoint resolved the packet’s main instrumentation gap: we now have an
explicit tqhnsw `amrescan` boundary counter and can compare it directly against
the representative SQL `Index Scan` startup time on the current build.

## Exit criteria

- this packet explains where the missing AM startup time lives relative to the
  current counters
- the result is based on the representative real-`10k` query, not synthetic
  microbenchmarks alone
- the next optimization target is a narrower internal seam than the current
  “somewhere inside tqhnsw startup” state
