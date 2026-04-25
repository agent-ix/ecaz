---
id: ADR-048
title: "Parallel HNSW Build Graph Assembly"
status: DECIDED
impact: Affects FR-008 (HNSW build), FR-021 (parallel build), ADR-042
date: 2026-04-25
revised: 2026-04-25
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

The current `build_parallel` coordinator is shaped like PostgreSQL parallel
build ingestion:

- worker processes own heap scan and tuple encoding
- encoded tuples flow through per-worker `shm_mq` streams
- the leader owns the final `BuildState`
- graph assembly is `SerialLeader`

That boundary is not reusable for graph assembly. It gives workers no writable
graph surface after tuple ingestion finishes.

An initial PROPOSED version of this ADR described a partitioned graph assembly
strategy (workers build local subgraphs, leader merges boundary patches).
Packet 632 reviewed that proposal against pgvector's shipped implementation and
the hnsw_rs crate used in tqvector's prior build path. That review reversed the
decision. This ADR records the revised decision.

## Decision

Adopt **concurrent graph insertion into a DSM-resident node array** as the
parallel graph assembly strategy. Workers insert into the same shared graph
concurrently, protected by per-node LWLocks. The existing page-staging contract
is unchanged.

### Why concurrent insertion over partitioned graph assembly

Three reference implementations were examined to settle this question:

**pgvector** ships concurrent parallel HNSW build in production. Workers and
leader all call the same `BuildCallback` → `HnswFindElementNeighbors` path
against a single shared graph in DSM. The graph is protected by per-element
LWLocks, an entry point LWLock pair, and a shared memory allocator. This is the
direct PostgreSQL AM precedent.

**hnsw_rs 0.3.4** (tqvector's prior build dependency) uses the same per-node
RwLock granularity via `Arc<RwLock<>>` on each point's neighbor list and a
single `Arc<RwLock<>>` on the entry point. Its `parallel_insert` is
`datas.par_iter().for_each(insert)` — concurrent insertion with the same lock
structure. Two independent implementations converged on the same lock shape.

**Partitioned graph assembly** (the prior proposal) has one unresolved central
risk: cross-partition navigability. HNSW's long-range graph structure is
established during insertion by following paths through globally-visible nodes.
A worker that only sees its own partition produces locally dense but globally
disconnected neighborhoods. The "boundary candidate discovery" step is the
quality gate, and its behavior under different corpus distributions is not
characterized. Partitioned assembly requires more infrastructure (DSM corpus,
patch wire format, deterministic merge, recall gate) before any speedup can be
measured. Concurrent insertion has none of these prerequisites.

### tqvector-specific advantages over pgvector

**No raw-vector DSM overflow.** pgvector's shared graph arena is bounded by
`maintenance_work_mem` because it stores raw f32 vectors in DSM. When the
graph overflows, pgvector falls back to serial on-disk insertion per tuple,
losing the parallel benefit entirely. tqvector encodes all tuples before graph
assembly. The DSM build surface must expose compact code bytes so worker
processes can score candidates without reading the leader's Rust `BuildState`,
but it does not need to store raw source vectors. At 50k nodes × m=6 the
neighbor-slot footprint is 2–4 MB, and the encoded-code corpus remains compact
relative to raw f32 vectors. The pgvector raw-vector overflow fallback does not
apply.

**No entry point lock during insertion.** tqvector assigns node levels from a
deterministic seed. All levels are knowable before workers launch. The entry
point (first node at the maximum level) is fixed before any worker starts.
pgvector needs its `entryLock + entryWaitLock` two-lock dance because level
assignment happens at insertion time and any inserting worker might produce a
new entry point. tqvector eliminates this contention entirely.

**Shorter lock hold per candidate.** tqvector scores candidates as
`score(codes[a], codes[b])` using in-process SIMD kernels — no heap fetch, no
distance function dispatch, no raw vector normalization. The neighbor-slot write
lock is held for shorter than pgvector's equivalent, reducing contention.

### Determinism

ADR-042 made determinism a first-class criterion: "determinism given a fixed
(seed, dimensions, bits, options) tuple." Concurrent insertion produces
nondeterministic neighbor selection when multiple workers process nearby nodes
simultaneously.

The resolution: treat determinism as a **recall quality gate**, not a
byte-for-byte topology requirement. This is consistent with ADR-042's own
language — "up to the documented tolerance of the backlink pruning heuristic"
— and with how determinism is already applied to the live INSERT path (which
processes rows in arbitrary transaction order). Level assignment remains
deterministic because levels are pre-computed from the seed before any worker
starts. The graph topology varies by insertion order; the graph quality is
bounded by the same ef_construction-width search every insertion performs.

Parallel build must meet the same recall thresholds on the existing gates as
serial build. That is the acceptance criterion, not byte-level reproducibility.

## Model

### Pre-assembly phase (leader, before workers launch)

1. Heap ingestion completes via the existing shm_mq coordinator (unchanged).
2. Leader owns `BuildState` with all encoded tuples in `heap_tuples`.
3. Leader pre-computes all node levels from the deterministic seed.
4. Leader identifies the entry point: the first node at the maximum level.
5. Leader allocates the DSM build surface:
   - compact encoded code bytes for candidate scoring across worker processes
   - a flat node array, with each node holding one `LWLock`, level, and
     neighbor-slot offset/count
   - one flat neighbor-slot array addressed by node offsets
   The graph arrays are pre-sized to `heap_tuples.len()` and the pre-computed
   slot counts. No dynamic allocation occurs during insertion.

### Insertion phase (all participants)

Workers and leader each take a partition of the node index range via the
existing parallel table scan mechanism. For each node in their partition:

1. Read the node's pre-computed level.
2. Search the shared graph from the (fixed, lockless) entry point down to
   layer 0 using the existing beam search logic with `LWLock` shared reads on
   neighbor slots.
3. Select forward neighbors and write them into the node's slot under exclusive
   `LWLock`.
4. Add backlinks to each selected neighbor under exclusive `LWLock` on the
   neighbor's slot.

Visited sets, candidate heaps, and scoring workspaces are worker-local (not
shared). The same reusable scratch structures from the serial native builder
apply per-worker.

### Post-assembly phase (leader)

After all workers finish:

1. Leader reads the completed DSM node array.
2. Passes it to the existing `build_native_hnsw_graph` flush path unchanged.
3. Page staging, WAL accounting, and index write are unchanged.

## Why Not Reuse The Parallel Scan Coordinator

The scan coordinator owns query-time work: scan descriptor attachment, rescan
epochs, worker slot ownership, and runtime snapshots. Build graph assembly needs
a different contract. `src/am/ec_hnsw/build_parallel.rs` remains the right home
for build orchestration. The existing heap-ingestion coordinator in that file is
retained; graph assembly adds a second code path alongside it.

The shm_mq streams become unnecessary for graph assembly (workers insert
directly into the DSM graph instead of sending encoded tuples to the leader).
The parallel table scan setup, worker launch, and WAL/buffer accounting from the
existing coordinator are reused.

## First Implementation Spike

The spike should be intentionally narrow to keep the highest-risk question
front and center: does concurrent insertion preserve enough recall?

1. Add `EcHnswBuildGraphAssembly::ConcurrentDsm` to the plan enum without
   enabling it by default.
2. Add the pre-assembly phase: level pre-computation, entry point selection,
   DSM node array allocation with per-node LWLock initialization.
3. Implement worker-local insertion: beam search with shared-read neighbor
   access, forward neighbor write under exclusive lock, backlink write under
   exclusive lock on the target node.
4. Wire one worker in a single-participant test (leader only, no workers
   launched) to prove the DSM graph round-trips through page staging correctly.
5. Enable multi-worker execution and run the existing recall gate on a 50k
   fixture. Record the recall delta against serial native build before enabling
   as the default path.

## Acceptance Criteria

The concurrent DSM path may become the default only after packets show:

- **Correctness:** same heap/index tuple counts, valid page staging, no
  malformed neighbor slots, valid entry point.
- **Recall:** parallel build meets the existing recall gates (same thresholds
  as serial native build). The measured delta must be recorded in the
  implementation packet.
- **Speed:** at least one 50k+ fixture where wall-clock build time improves
  materially (graph assembly phase, not only heap ingestion).
- **Fallback:** serial leader graph assembly remains available and is selected
  when `requested_workers == 0` or DSM allocation fails.

## Consequences

### Positive

- Matches the approach proven by pgvector in production.
- Eliminates the cross-partition navigability risk of the partitioned approach.
- Removes the boundary merge infrastructure requirement.
- Leader participates in graph insertion (no dedicated queue drainer).
- No raw-vector DSM overflow fallback needed due to compact code and graph
  representation.
- No entry point lock contention due to pre-determined levels.
- Existing page staging contract unchanged.

### Negative

- Graph topology is nondeterministic by insertion order. Recall must be
  measured and recorded; byte-level reproducibility is not guaranteed.
- Per-node LWLock adds 16 bytes per node in DSM (800 KB at 50k nodes —
  acceptable).
- Backlink write contention is the hot path. Mitigation: short lock hold time
  due to SIMD scoring without heap fetch.

## Alternatives Considered

### Partitioned Local Graphs with Deterministic Leader Merge

Described in the initial PROPOSED version of this ADR. Rejected after packet
632 review because:

- Cross-partition navigability is the central uncharacterized risk.
- Requires more infrastructure (corpus DSM, patch format, deterministic merge)
  before any speedup can be measured.
- Concurrent insertion is already proven in production by pgvector.

### Continue Serial Hot-Path Cleanup Only

Rejected as the main strategy. Packets `628` through `631` removed several real
hot-path costs, but 50k graph assembly still takes ~27 seconds. Further cleanup
can help incrementally but will not make parallel index build scale.

### Worker Score Offload Per Search Step

Rejected. Each HNSW expansion scores a small neighbor slice. Cross-process
round trips would dominate unless the search loop were redesigned into large
batches, which is not how beam search behaves.

## References

- ADR-042: Native HNSW build path
- Packets `626` through `632`: task 19 build measurements, graph hot-path
  cleanup, and this ADR review
- pgvector `src/hnswbuild.c`: concurrent parallel build reference
- hnsw_rs 0.3.4 `src/hnsw.rs`: per-node RwLock + rayon parallel insert
- `src/am/ec_hnsw/build_parallel.rs`: current heap-ingestion coordinator
- `src/am/ec_hnsw/build.rs`: native serial graph builder and page staging
