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

## Exit criteria

- this packet explains where the missing AM startup time lives relative to the
  current counters
- the result is based on the representative real-`10k` query, not synthetic
  microbenchmarks alone
- the next optimization target is a narrower internal seam than the current
  “somewhere inside tqhnsw startup” state
