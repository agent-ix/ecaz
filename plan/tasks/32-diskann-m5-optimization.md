# Task 32: DiskANN M5 Optimization

Status: proposed
Owner: coder1 / runtime-index track
Priority: 2

## Goal

Use the M5 laptop to continue DiskANN performance work after the landed Task 29
lane, with IVF taking priority. DiskANN should focus on closing the remaining
constant-factor gaps without destabilizing the high-recall local profile.

Task 29d already made the initial path landable: build dropped to the accepted
local target, recall stayed near exact, and scan latency beat pgvectorscale at
higher search-list sizes. The remaining work is targeted optimization, not
foundation repair.

## Baseline Rules

- Start from the Task 29d final profile:
  `graph_degree = 32`, `build_list_size = 100`, `alpha = 1.2`.
- Keep pgvectorscale and `ec_hnsw` reference rows isolated from the DiskANN
  table/index unless the packet explicitly measures shared-table planner
  behavior.
- Record release build, extension SHA, M5/macOS shape, PG18 settings, corpus
  manifest, cache state, and one-index-per-table status in every measurement
  packet.

## Phase 1: Refresh The M5 Baseline

- Re-run the Task 29d final sweep on M5 for search-list values
  `64, 128, 200, 400, 800`.
- Capture build time, index size, recall@10, p50/p95/p99 latency, memory HWM,
  and `EXPLAIN (ANALYZE, BUFFERS)` for representative low-L and high-L queries.
- Re-check the L=64 gap against pgvectorscale, since low-L latency was the
  remaining known weakness.

## Phase 2: Profile Before Changing Defaults

Profile the scan path at L=64 and L=200 separately. Attribute time to:

- binary sidecar prefilter and popcount;
- persisted graph page reads and tuple decoding;
- frontier maintenance;
- exact heap rerank;
- result materialization and per-rescan setup.

Do not lower `rerank_budget` or change default search parameters unless recall
stays inside the accepted floor.

## Phase 3: Candidate Slices

Recommended order:

1. **Per-scan graph read cache.** If profiling shows repeated page or tuple
   decoding in one scan, cache decoded graph tuples for the scan lifetime.
2. **Rerank staging.** If exact heap rerank dominates, add a measured
   intermediate ranking stage only when it preserves the recall floor.
3. **Frontier/result scratch reuse.** Reuse scan-local buffers across rescans
   where allocation shows up in profiles.
4. **Build-side follow-up.** Only reopen build work if M5 profiling identifies
   a new dominant contributor after the landed source-distance SIMD win.
5. **Apple Silicon kernel check.** Verify whether the binary sidecar and source
   distance kernels dispatch to the best available arm64 backend. Put any broad
   SIMD backend work in Task 21.

## Validation

- Performance packets need recall and latency together; latency-only wins are
  not accepted for DiskANN.
- Keep build and scan claims separate unless one patch intentionally affects
  both.
- Use focused PG18 tests only when code changes touch PostgreSQL callback
  behavior or correctness-sensitive graph traversal.

## Stop Conditions

- Stop low-L work if the only apparent win is lowering rerank below the recall
  floor.
- Stop build work if the next largest contributor is spread across small
  constant factors without a single ≥30% profile target.
- Defer larger scale curves until IVF has consumed the first M5 optimization
  pass.
