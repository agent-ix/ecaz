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

1. Start with the narrowest scan-result materialization copy on the hot path
   so the verified warm seam can judge it honestly before a larger graph read
   rewrite.
2. If that slice is flat, drop it instead of committing complexity and move to
   the actual graph tuple read/decode boundary.
3. Re-measure the verified warm per-cell `10K` `m=8`, `ef_search=40` surface
   after each narrow attempt.

## Current draft

Probe 1, discarded:

- replaced `SelectedScanResult.heap_tids: Vec<ItemPointer>` with inline fixed
  storage so graph/linear materialization no longer allocated a temporary
  heap-tid `Vec` before copying into `ScanResultState`
- warm read came back effectively flat at `p50=14.176ms`, `p95=16.748ms`,
  `p99=17.768ms`, `mean=14.218ms` against the packet `261` baseline
  (`p50=14.315ms`, `p95=16.350ms`, `p99=17.613ms`, `mean=14.194ms`)
- result: dropped instead of committing no-signal complexity

Checkpoint kept in this packet:

- `src/am/graph.rs` now decodes element and neighbor tuples directly from the
  locked page slice
- the old `read_page_tuple_bytes(...).to_vec()` boundary is removed
- graph caches still own decoded `Vec` payloads, but the temporary full-tuple
  byte copy no longer exists on the hot path

Validation read:

- `cargo test`: green on rerun
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: green
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`:
  green

Warm measurement against the packet `261` baseline:

- prior baseline: `p50=14.315ms`, `p95=16.350ms`, `p99=17.613ms`,
  `mean=14.194ms`
- run 1 after the graph decode change: `p50=13.914ms`, `p95=16.240ms`,
  `p99=20.743ms`, `mean=14.032ms`
- run 2 after the graph decode change: `p50=13.997ms`, `p95=16.147ms`,
  `p99=17.652ms`, `mean=13.958ms`

Current read:

- this is a small but directionally consistent warm improvement, roughly
  `1-2%` on `p50`/mean
- it is not remotely enough to close C1, but it is cleaner and more defensible
  than the discarded scan-result materialization tweak
- the next meaningful work should still target the larger owned decode/result
  boundaries or the scoring path, not stop here

## Exit criteria

- one concrete graph tuple copy/materialization seam is narrowed or removed
- the change is validated through the normal checkpoint gate
- the verified warm per-cell surface is rerun and compared to the current
  `p50=14.315ms` baseline
