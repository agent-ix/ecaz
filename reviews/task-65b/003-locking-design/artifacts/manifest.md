# Task 65b Locking Design Artifact Manifest

- packet type: design-only review request
- code commit under review: `37052b81693d3b5693d95fcaa0f29f7ffc748063`
- packet commit: pending at artifact creation
- task bucket: `reviews/task-65b/003-locking-design/`
- timestamp: `2026-06-04T20:48:33Z`
- host lane: local M5 context from packets 001 and 002
- fixture/storage context: real10k and real100k, `ec_diskann`, `pq_fastscan`
- build reloptions context: `graph_degree=32`, `build_list_size=100`, `alpha=1.2`

## Inputs

`reviews/task-65b/001-measurement-floor/request.md`

- real10k SQL index build `6.72s`
- real100k SQL index build `243.29s`
- real10k Recall@10 L64/L128/L200 `0.9965 / 0.9970 / 0.9975`
- real100k Recall@10 L64/L128/L200 `0.9190 / 0.9640 / 0.9755`
- real10k in-degree p95/p99/max `52/79/2881`
- real10k backlinks `142105`, reprunes `61593`

`reviews/task-65b/002-neighbor-cache/request.md`

- code checkpoint `37052b81693d3b5693d95fcaa0f29f7ffc748063`
- serial `BuilderNeighborCache` validation passed
- real10k SQL index build `6.86s`
- real10k build-probe `62.168s`
- real10k Recall@10 L64/L128/L200 unchanged at
  `0.9965 / 0.9970 / 0.9975`
- real10k in-degree p95/p99/max unchanged at `52/79/2881`

`plan/tasks/65b-diskann-build-parallel-graph-construction.md`

- Slice C asks for a locking strategy with a contention model based on the hub
  degree distribution from Slice A.
- Determinism default should be called out in the first design packet.

`src/am/ec_diskann/routine.rs`

- `ec_diskann` currently has `amcanbuildparallel = false`.

`src/am/ec_hnsw/build_parallel.rs`

- Existing PG `ParallelContext` and DSM/LWLock precedent for build workers.

`src/am/ec_ivf/build_parallel.rs`

- Existing PG `ParallelContext` precedent for parallel heap ingestion and
  build-worker instrumentation.

## Decision Summary

The chosen first implementation path is deterministic epoch/batch proposal with
ordered leader commit:

- rayon graph-core worker fanout for the stepping-stone implementation
- immutable epoch snapshot for proposal reads
- leader-owned `BuilderNeighborCache` commit in fixed pivot order
- sharded `RwLock`/LWLock stripes reserved for a future live shared-cache path
  if ordered commit blocks the Task 65b performance target

## Validation

No new validation commands were run for this design-only packet. Packets 001 and
002 provide the measurement and cache-behavior evidence cited by this request.
