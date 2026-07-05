---
type: ADR
id: ADR-075
title: "DiskANN Graph Build Worker Stepping Stone"
status: PROPOSED
impact: Affects Task 65b, ec_diskann build, FR-021 parallel build
date: 2026-06-05
---
# ADR-075: DiskANN Graph Build Worker Stepping Stone

## Context

Task 65b targets DiskANN Vamana graph-construction time. Existing peer AMs
(`ec_hnsw`, `ec_ivf`) use PostgreSQL `ParallelContext` for build workers and
read worker count from `IndexInfo::ii_ParallelWorkers`, which PostgreSQL derives
from standard table reloptions and parallel-maintenance GUCs.

DiskANN's first graph-parallel design is different from HNSW/IVF heap-ingest
parallelism. Slice C chooses deterministic epoch proposal plus ordered leader
commit: workers compute proposals, while one reducer owns graph mutation. That
single-writer boundary avoids backlink races during the correctness slice but
does not yet require DSM-resident graph mutation.

## Decision

Use a **rayon graph-core stepping stone** for Task 65b Slice D/E proposal work,
while keeping PostgreSQL as the worker-count authority:

- `ec_diskann` sets `amcanbuildparallel = true`.
- Worker count comes from `IndexInfo::ii_ParallelWorkers`; no custom DiskANN
  worker-count reloption is introduced.
- The AM wires the common parallel-scan callback surface, matching HNSW/IVF.
- Slice D supports `ii_ParallelWorkers = 0` and `1` only.
- `ii_ParallelWorkers = 0` preserves the serial fallback path.
- `ii_ParallelWorkers = 1` enters the rayon scaffold and must remain
  byte-equivalent to serial output.
- Multi-worker proposal fanout, stale-read accounting, and reducer timing land
  in later Task 65b slices before any production default changes.

`parallel_build_batch_size` and `parallel_build_flush_rate` remain DiskANN
graph-build reloptions because they are algorithmic Vamana epoch/cache controls,
not PostgreSQL worker-coordination controls. Until implemented, non-default
values are rejected rather than silently ignored.

## Gate #6 Disposition

Task 65b validation gate #6 applies only to a production `ParallelContext`
coordinator. It is not claimed by the rayon stepping stone. The stepping stone
must not be treated as complete Task 65b acceptance unless performance and
review evidence explicitly accept the missing PostgreSQL worker visibility,
WAL/buffer attribution, and `pg_stat_activity` integration.

## Migration Trigger

Open the `ParallelContext`/DSM coordinator path when either condition is true:

- ordered leader commit is the measured Amdahl bottleneck after Slice E/F, or
- Task 65b acceptance requires PostgreSQL worker accounting/visibility for the
  chosen production path.

The migration should reuse the peer AM `ParallelContext` lifecycle and adapt the
HNSW DSM/LWLock patterns only after the DiskANN reducer/proposal invariants are
covered by tests.
