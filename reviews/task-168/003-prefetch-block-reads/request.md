# Review request: Task 168 Phase 3 — prefetch + block-grouped reads (negative result, shelved)

- Branch: `task-168-diskann-batched-beam`
- Commits: `9e0a09617` (slice) → `685e81f0c` (shelve; `src/` restored
  byte-identical to the packet-002 state `5ac7e00b0`).
- Evidence: `artifacts/manifest.md` — three-arm A/B (baseline / prefetch+
  grouping / grouping-only) at the W=4 default, release backend,
  packet-001 rabitq fixture.

## Summary

Phase 3 (graph-page prefetch via the shared read-stream helper +
block-grouped batch reads) is a measured loss on this task's envelope and
is shelved per the task's land-only-≥5%-wins rule:

- prefetch+grouping: warm latency regressed at every scale, worst +29%
  (50k L=400). Root cause: the pg18 read-stream prefetch helper pins and
  releases each block synchronously, then the grouped read re-pins —
  double buffer traffic with no I/O to hide on a warm index.
- grouping-only: neutral at 10k/50k, negative at 100k; no cell reaches 5%.
- recall bit-identical in all arms (read-order-only change).
- The task file's scan-lifetime node cache was not built: frontier dedup
  already guarantees single-read-per-node-per-scan (packet 001 counters),
  so an intra-scan cache cannot hit. If reviewers want a cross-rescan
  cache investigated it needs a nested-loop-join workload, which none of
  the bench lanes exercise.

## Asks

1. Concur with shelving Phase 3 (evidence over prediction — the task file
   projected a win here; the A/B says otherwise on 10/50/100k).
2. Note the caveat: cold-tail prefetch may still matter for
   larger-than-memory indexes (out of this task's envelope). If that
   regime becomes a target, reopen with an eviction-per-query harness.
3. One measurement hygiene note for future packets: suite-level OS-evict
   steps poison *subsequent* suite runs in the same session — packet 003's
   first grouping-only pass had to be re-run warm (both passes are in the
   packet, the manifest says which to trust).
