# Review Request: C1 Graph Tuple Copy Boundary

## Context

Packet `261` corrected the warm-cache benchmark seam:

- verified warm runs now support `--warmup-passes`
- warm steady-state measurement now uses `--session-mode per-cell`
- honest warm `10K` latency improved materially versus the old one-backend-per-
  query harness read, but still lands around `p50=14.3ms` at
  `m=8, ef_search=40`

So C1 remains open on real warm steady-state latency even after the launcher
fix.

## Problem

The current hot-path evidence no longer points to the benchmark harness first.
The strongest remaining code-level suspects are owned-copy boundaries during
graph tuple fetch/decode and result materialization.

Current evidence:

- backend `perf` still shows scoring as the hottest single symbol, but the next
  non-scoring seams are graph tuple read/decode and scan bookkeeping
- line/code search points at:
  - `src/am/graph.rs` `read_page_tuple_bytes(...)`
  - `src/am/page.rs` tuple decode paths that allocate/copied owned `Vec`s
  - `src/am/scan.rs` graph-result materialization that clones heap tids into
    scan state

## Planned work

1. Inspect the graph tuple read/decode boundary and remove the most obvious
   owned-copy path that is still on the warm scan hot path.
2. Keep the change narrow enough that the warm verified seam can judge it
   honestly.
3. Re-measure the verified warm per-cell `10K` `m=8`, `ef_search=40` surface.

## Exit criteria

- one concrete graph tuple copy/materialization seam is narrowed or removed
- the change is validated through the normal checkpoint gate
- the verified warm per-cell surface is rerun and compared to the current
  `p50=14.315ms` baseline
