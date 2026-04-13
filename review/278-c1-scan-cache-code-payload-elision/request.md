# Review Request: C1 Scan Cache Code Payload Elision

## Context

The smaller zero-copy and allocation probes are mostly exhausted:

- packet `262` kept a narrow graph tuple copy-boundary fix for a small real win
- packets `263`, `276`, and `277` were all valid follow-up probes and were
  discarded

The remaining structural seam is larger than another small-`Vec` tweak. Warm
C1 still reloads graph elements into a scan-local cache that stores full owned
encoded `code` payloads, even though steady-state ordered scan mostly needs:

- element liveness / heap-tid presence
- neighbor-link metadata
- the element score, which is already cached separately

That means each graph-element cache miss still copies the full compressed code
payload into `GraphElement.code`, even when the code bytes are only needed once
to compute the first score.

## Problem

`cached_graph_element(...)` in `src/am/scan.rs` currently loads
`graph::GraphElement`, which owns:

- `heaptids: Vec<ItemPointer>`
- `code: Vec<u8>`

The code payload is materially larger than the heap-tid list on the real
`1536`-dim, `4-bit` corpus. After the first score is computed, steady-state
scan-local traversal does not need to keep that payload around in the graph
cache. Keeping it means extra per-element allocation/copy work on every graph
cache miss.

## Implementation

Completed work:

1. Added a borrowed `TqElementTupleRef` decode view in `src/am/page.rs`.
2. Added a graph helper in `src/am/graph.rs` so scan can operate on that
   borrowed tuple while the page buffer is pinned.
3. Reworked the scan-local graph-element cache in `src/am/scan.rs` so it keeps
   only the metadata needed after first score computation:
   - `tid`
   - `level`
   - `deleted`
   - `heaptids`
   - `neighbortid`
4. Populated the scan-local score cache from the borrowed decode path on graph
   cache miss, so the encoded `code` payload is scored once without being copied
   into the cached scan-local element.

This kept the broader `graph::GraphElement` surface intact for build / vacuum /
debug code and limited the zero-copy change to the steady-state ordered-scan
cache path.

## Result

Kept.

Relative to the standing packet `270` warm verified baseline on real `10K`,
`m=8`, `ef_search=40`, `warm-after-prime3`, `per-cell`, `cached-plan`:

```text
packet 270 baseline:
  p50=10.753ms p95=12.784ms p99=14.034ms mean=10.720ms

probe run 1:
  p50=10.479ms p95=12.162ms p99=14.205ms mean=10.418ms

probe run 2:
  p50=10.590ms p95=12.535ms p99=14.752ms mean=10.603ms
```

So this slice is another small but real warm-path keep:

- both confirmation runs improved mean latency versus the standing packet `270`
  baseline
- the average of the two confirmation runs is `10.511ms`, about `1.95%` below
  the packet `270` baseline mean
- the gain is not large enough to change the overall C1 picture by itself

This likely exhausts the remaining clearly-defensible zero-copy / payload-copy
reduction seam for the current scan-local cache shape. The next priority should
shift to the new quantized-data search/scoring lane (`ADR-031`, `ADR-030`)
rather than returning to `ADR-029`.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- two verified warm real-corpus reads on:
  - `scripts/bench_sql_latency_verified_scratch.sh`
  - `--prefix tqhnsw_real_10k`
  - `--m 8`
  - `--ef-search 40`
  - `--cache-state warm-after-prime3`
  - `--warmup-passes 3`
  - `--session-mode per-cell`
  - `--timing-mode cached-plan`

## Exit Criteria

- the packet records whether removing scan-local cached code payload ownership
  improves the verified warm real-corpus `10K`, `m=8`, `ef_search=40`,
  `warm-after-prime3`, `per-cell`, `cached-plan` seam
- the packet records whether the code checkpoint was kept or discarded
