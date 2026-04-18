# Task 18: Parallel Index Scan

Status: proposed — broad-reach latency win, no Postgres vector extension has this today.

Executes ADR-040.

## Scope

Enable `amcanparallel=true` for `tqhnsw` so a single `ORDER BY v <#> q LIMIT k`
query can be split across multiple Postgres workers. Workers run independent
beam searches against a shared top-K coordinator; `ef_search` is budgeted per
worker with a small overlap term so aggregate recall matches (or exceeds) a
single-worker scan at the same total budget.

Goal: linear-ish latency reduction on warm indexes for 2/4/8 workers, and
automatic inheritance of parallelism by DiskANN (task 17) and any future AM
that shares the scan seam.

## Why now

- No other Postgres vector extension (pgvector, pgvectorscale, vectorchord)
  ships a parallel index scan today. Strongest single differentiator per unit
  of effort.
- Broad-reach: every `ORDER BY v <#> q` query benefits once the planner picks
  the parallel plan.
- Compounds with everything downstream. OPQ, AQ/RVQ, DiskANN, SPANN all
  inherit parallelism automatically because the seam is in scan coordination,
  not the scoring kernel.
- Cache-line contention is the only real risk, and the coordinator-only
  shared state (top-K heap) keeps contention bounded.

## Design outline

See ADR-040 for the full shape. Summary:

- **Shared state (DSM):** single top-K min-heap, protected by a lightweight
  lock. Workers push candidates that beat the current kth; coordinator pops
  on scan end.
- **Per-worker state (DSM slots):** independent beam frontier, visited set,
  and scoring scratch. No shared visited set (the coordination cost
  outweighs the redundancy savings at typical `ef_search`).
- **Budget split:** per-worker `ef_search = ceil(ef_search_total / n) *
  (1 + overlap)`, with `overlap` in the 5–15% range. Overlap compensates
  for workers missing neighbors the others already explored.
- **Entry points:** each worker starts from the same Layer-N+ entry point
  but with a distinct RNG seed for beam initialization (prevents all workers
  exploring identical paths).
- **Correctness invariant:** with `n=1` the parallel path must produce
  byte-identical results to today's serial path. Enforced by a scan-mode
  test.

## Subtasks

### Coordinator and DSM

- [ ] **DSM layout.** Define shared top-K heap, lock word, and worker state
  slots in `src/am/parallel.rs`. Size computed by
  `amestimateparallelscan`.
- [ ] **Shared top-K push/pop.** Lock-guarded; hot path is "is candidate
  better than current kth". Fast-reject without taking the lock when
  candidate score is clearly worse than a snapshot of the current kth.
- [ ] **Per-worker state carriers.** Beam frontier, visited bitmap/bloom,
  score scratch. Lives in per-worker DSM slot, never touched by peers.

### AM callback wiring

- [ ] **`amcanparallel = true`** in the `IndexAmRoutine` for `tqhnsw`.
- [ ] **`amestimateparallelscan`.** Returns DSM size = coordinator state +
  `n * per_worker_state`.
- [ ] **`aminitparallelscan`.** Populate coordinator heap, initialize
  per-worker slots.
- [ ] **`amparallelrescan`.** Reset coordinator and per-worker state for
  re-execution (nested loops, param re-bind).
- [ ] **Worker-side scan entry.** Each worker's `ambeginscan` path detects
  the parallel DSM slot and configures its local `TqScanOpaque` against
  it.

### ef_search budget split

- [ ] **Budget math in `resolve_scan_tuning`.** Compute per-worker
  `ef_search` as documented above. GUC `tqhnsw.parallel_ef_overlap`
  (default `0.1`, range `[0.0, 0.5]`) controls the overlap term.
- [ ] **Single-worker equivalence test.** `n=1` parallel scan produces
  byte-identical results to serial scan at the same `ef_search`.

### Planner integration

- [ ] **Parallel cost in `amcostestimate`.** Extend the cost model (D2
  lane in task 11) so the planner prefers the parallel path on
  large indexes when `max_parallel_workers_per_gather > 0`.
- [ ] **EXPLAIN.** Surface per-worker counter rollups (pages read,
  elements scored) via the EXPLAIN hook from task 11.

### Tests and benchmarks

- [ ] **Correctness harness.** Same query across `n = 1, 2, 4, 8`
  workers must produce the same top-K identities (IDs may tie; require
  identity-set equality or a bounded score-delta tolerance).
- [ ] **Recall parity.** At the same *aggregate* `ef_search`, parallel
  recall should match serial within ±1 pp on the 50k warm real seam.
- [ ] **Latency benchmark.** Warm index, `LIMIT 10`, `ef_search=40`, rows
  between 100k and 10M. Report mean / p95 at 1/2/4/8 workers.
- [ ] **Contention stress.** Measure coordinator-lock wait time at 16
  workers to bound the scaling ceiling.

## Owns

- ADR-040
- `src/am/parallel.rs` (new)
- Parallel-scan callbacks in `src/am/mod.rs`

## Dependencies

- Task 15 (PqFastScan first-class). The parallel seam sits above scoring;
  once the two formats share a stable scan loop, adding parallel is
  additive rather than per-format.
- Task 11 D2 planner wiring. Parallel plan selection depends on the
  planner trusting `amcostestimate`, which is gated on the recall gate.

## Unblocks

- DiskANN (task 17) inherits parallel scan the moment it lands, since
  the seam is at the coordination layer not the scoring layer.
- SPANN (ADR-035) likewise.
- Multi-core utilization on any vector workload — the single biggest
  user-visible latency win short of a scoring-kernel change.

## Out of scope

- Parallel build. Build-side parallelism is a separate, larger project
  (coordinator-free, different bottlenecks).
- Cross-query batching on a single worker.
- Parallel vacuum.

## Notes

- **No shared visited set.** Cost analysis in ADR-040 shows the cross-
  worker synchronization cost exceeds the ~5–15% redundant-work savings
  for `ef_search ≤ 200`. Revisit if a workload emerges where `ef_search`
  is routinely above 500.
- **Overlap default.** Start at 10%. Can drop to 0 for throughput-
  sensitive workloads willing to accept a small recall hit.
- **Cache-line contention.** Top-K heap lock is the one hot contention
  point. Mitigation: workers snapshot the current kth score and
  fast-reject locally before taking the lock.
- **PG17 vs PG18.** Parallel index scan callbacks exist in both. No
  PG18 gate needed for this work.
