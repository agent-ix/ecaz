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
- Those totals are far smaller than the earlier `EXPLAIN (ANALYZE, FORMAT JSON)`
  `Index Scan` startup number (`46.187ms` at `ef_search=40`).
- Backend-side `perf` is now available on this machine and the first real server
  capture against repeated plain SQL scans shows tqvector cycles still dominated
  by scoring:
  - `31.78%` `tqvector::quant::prod::ProdQuantizer::score_ip_from_split_parts`
  - `4.11%` `Vec<T>::extend_from_slice`
  - `1.20%` `tqvector::am::graph::read_page_tuple_bytes`
- A direct wall-time sanity check on the plain query path is materially lower
  than the `EXPLAIN` numbers:
  - 5,000 repeated plain scans of the representative query completed in `4.42s`
  - that is about `0.884ms/query` end-to-end through `psql`

## Current Read

The missing `~40ms` does not appear to be hidden inside tqhnsw `amrescan`
itself. The stronger emerging hypothesis is that the old C1 `EXPLAIN ANALYZE`
surface is dominated by `EXPLAIN`/instrumentation overhead rather than by the
actual plain ordered index scan.

The next step is to confirm that interpretation on a small multi-query sample
and then decide whether C1 should pivot from `EXPLAIN`-reported execution time
to a plain-query timing harness for the real latency requirement.

## Exit criteria

- this packet explains where the missing AM startup time lives relative to the
  current counters
- the result is based on the representative real-`10k` query, not synthetic
  microbenchmarks alone
- the next optimization target is a narrower internal seam than the current
  “somewhere inside tqhnsw startup” state
