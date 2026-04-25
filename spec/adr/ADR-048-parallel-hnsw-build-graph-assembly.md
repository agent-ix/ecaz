---
id: ADR-048
title: "Parallel HNSW Build Graph Assembly"
status: PROPOSED
impact: Affects FR-008 (HNSW build), FR-021 (parallel build), ADR-042
date: 2026-04-25
---
# ADR-048: Parallel HNSW Build Graph Assembly

## Context

ADR-042 moved production `ec_hnsw` builds to Ecaz-owned native HNSW
construction. Task 19 added a first executable PostgreSQL parallel build path:
workers scan heap pages and encode tuples, while the leader drains encoded
tuples, sorts them by heap TID, and runs the existing native graph/page assembly.

Measurements from packets `626` through `631` show the current limit clearly:

- Parallel heap ingestion launches workers and reduces heap scan/encoding time.
- The tuple merge and `BuildState::push` path is now small after packet `627`.
- Native graph hot-path cleanups in packets `628`, `629`, `630`, and `631`
  reduced serial graph cost, especially for indexed `tqvector` code scoring.
- On the 50k x64 `ecvector` source-scored fixture, graph assembly still takes
  about 27 seconds and dominates wall-clock build time.

The current `build_parallel` coordinator is intentionally shaped like PostgreSQL
parallel build ingestion:

- worker processes own heap scan and tuple encoding
- encoded tuples flow through per-worker `shm_mq` streams
- the leader owns the final `BuildState`
- graph assembly is `SerialLeader`

That boundary should not be reused unchanged for graph assembly. It gives
workers no durable shared view of all vectors and no writable graph surface once
tuple ingestion finishes.

## Decision

Adopt a separate **partitioned graph assembly** design for the next parallel
build phase. Keep the existing coordinator for heap ingestion, but add a second
graph-assembly phase with a different data contract.

The proposed model is:

1. **Shared immutable build corpus.** After tuple ingestion, materialize encoded
   tuples into a deterministic, read-only build corpus that graph workers can
   address by node id. For small/medium builds this can be a DSM-backed flat
   corpus; for larger builds it can spill to a temp-file-backed format. The
   first implementation should stay DSM-only and enforce a memory cap.
2. **Partitioned local graphs.** Split node ids into deterministic contiguous
   partitions. Each worker builds a local HNSW subgraph for its partition using
   the native graph builder over a partition-local node range and the same
   scoring workspace rules as serial build.
3. **Boundary candidate discovery.** For each partition, compute a small,
   deterministic set of cross-partition boundary candidates. The first spike
   should use partition entry points plus a fixed per-partition representative
   sample, then validate recall before adding more elaborate routing.
4. **Leader deterministic merge.** Workers emit graph patches: forward links
   and backlink proposals expressed in global node ids. The leader applies
   patches in `(partition_id, node_id, layer, target_id)` order using the same
   backlink pruning and tie-break rules as native serial build.
5. **Same page format.** The merged graph must still produce the existing
   `Vec<HnswBuildNode>` shape consumed by page staging. This ADR does not
   introduce a new on-disk format.

The initial implementation must be gated behind the parallel build plan and
must keep serial graph assembly as the default fallback until measurement and
recall packets prove the partitioned graph is good enough.

## Why Not Shared Concurrent HNSW Insert?

Classic multi-threaded HNSW construction inserts nodes concurrently into one
mutable graph using node locks. That is not the preferred first Ecaz path.

Reasons:

- PostgreSQL parallel workers are processes, not threads. A shared mutable graph
  would need DSM-resident slot arrays, per-node locks, and explicit lifetime
  management instead of ordinary Rust `Vec` ownership.
- Concurrent insertion order is naturally nondeterministic. ADR-042 made
  deterministic build topology a first-class acceptance criterion.
- HNSW insertion touches visited sets, candidate heaps, forward slots, and
  backlinks. Adding cross-process locks around each mutation risks spending the
  speedup on synchronization.
- The current page staging code already wants a complete `Vec<HnswBuildNode>`.
  A patch/merge phase preserves that contract while isolating parallelism from
  storage emission.

Shared concurrent insertion can be revisited later if partitioned graph
assembly fails recall or scaling gates, but it is not the first implementation
target.

## Why Not Reuse The Parallel Scan Coordinator?

The scan coordinator owns query-time work: scan descriptor attachment, rescan
epochs, worker slot ownership, and runtime snapshots. Build graph assembly needs
a different contract:

- immutable corpus addressing by node id
- worker-owned graph patches
- deterministic leader merge
- build-only WAL/buffer accounting
- page staging after graph merge

The dedicated `src/am/ec_hnsw/build_parallel.rs` boundary remains the right
home for build orchestration.

## First Implementation Spike

The first code spike should be intentionally narrow:

1. Add planning surface for `EcHnswBuildGraphAssembly::PartitionedWorkerPatches`
   without enabling it by default.
2. Add a pure-Rust partition planner that maps `(node_count, workers)` to
   contiguous node ranges and representative samples. Unit-test determinism and
   edge cases.
3. Add graph-patch data structures in leader-local Rust memory first. Do not
   introduce DSM graph storage until the merge contract is proven.
4. Implement a single-process simulation of partitioned local graph build plus
   deterministic merge, behind a test-only or debug-only entry point.
5. Measure graph quality against the current serial native build on small and
   50k fixtures before wiring real worker execution.

This keeps the highest-risk question front and center: whether partitioned HNSW
plus deterministic boundary merge preserves enough recall to justify making it
parallel in PostgreSQL workers.

## Acceptance Criteria

The partitioned graph path may become executable only after packets show:

- **Correctness:** same heap/index tuple counts, valid page staging, no
  malformed neighbor slots, deterministic output for fixed seed and worker
  count.
- **Recall:** no unacceptable recall regression against serial native build on
  the existing synthetic and real-corpus gates. The threshold should be recorded
  in the implementation packet before enabling the path.
- **Speed:** at least one 50k+ fixture where wall-clock build time improves
  materially after graph assembly, not only heap ingestion, is included.
- **Fallback:** serial leader graph assembly remains available and is selected
  whenever partitioned graph planning cannot satisfy memory or worker
  constraints.

## Consequences

### Positive

- Preserves the existing page format and native graph staging contract.
- Avoids cross-process fine-grained graph mutation in the first parallel graph
  implementation.
- Creates a reviewable quality gate before changing default graph topology.
- Lets the current heap-ingestion coordinator remain useful without forcing it
  to own graph mutation semantics.

### Negative

- The partitioned graph will not be byte-for-byte identical to serial native
  build. It needs recall validation, not only unit tests.
- Boundary merge quality is the central risk. Too few cross-partition edges will
  produce disconnected or weakly navigable local neighborhoods.
- DSM corpus storage is additional infrastructure after the single-process
  simulation proves the graph strategy.

## Alternatives Considered

### Continue Serial Hot-Path Cleanup Only

Rejected as the main strategy. Packets `628` through `631` removed several real
hot-path costs, but the 50k source-scored fixture still spends roughly 27
seconds in graph assembly. More cleanup can help, but it will not make parallel
index build scale.

### Worker Score Offload Per Search Step

Rejected. Each HNSW expansion scores a small neighbor slice. Cross-process
round trips would dominate the work unless the search loop were redesigned into
large batches, which is not how HNSW traversal behaves.

### Full Shared Mutable Graph In DSM

Deferred. It may eventually be necessary, but it creates lock-heavy,
nondeterministic, cross-process graph mutation before proving the simpler
partition/merge topology can meet recall gates.

## References

- ADR-042: Native HNSW build path
- Packets `626` through `631`: task 19 build measurements and graph hot-path
  cleanup packets
- `src/am/ec_hnsw/build_parallel.rs`: current heap-ingestion coordinator
- `src/am/ec_hnsw/build.rs`: native serial graph builder and page staging
