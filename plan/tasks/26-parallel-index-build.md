# Task 26: Parallel Index Build

Status: **in-progress** — heap ingestion coordinator landed (task 19); graph
assembly architecture decided by ADR-048 (packet 632).

## Scope

Enable multi-worker index build for `ec_hnsw`. A single `CREATE INDEX` /
`REINDEX` splits heap scan, tuple encoding, and HNSW graph construction across
PostgreSQL parallel workers under a shared coordinator.

Goal: material build-time reduction on 50k–10M-row corpora at 2/4/8 workers,
compounding the native-build lane's throughput wins (ADR-042) with parallelism.

Separate from task 18 (parallel scan). Shares worker launch / DSM sizing
infrastructure in `build_parallel.rs`; does not share coordinator semantics.

## Architecture Decision

**ADR-048 is DECIDED: concurrent graph insertion into a DSM node array.**

Workers and leader each insert into one shared `Vec<NativeBuildNode>` in DSM,
protected by one `LWLock` per node. This is the approach pgvector ships in
production, validated independently by hnsw_rs's `Arc<RwLock>` per-point
design.

Key tqvector-specific simplifications over pgvector:

- **No entry point lock during insertion.** Node levels are pre-computed from a
  deterministic seed before workers launch. The entry point (first node at max
  level) is fixed and does not change. pgvector needs its `entryLock +
  entryWaitLock` pair because level assignment happens at insertion time.
  tqvector eliminates this contention entirely.

- **No DSM overflow.** pgvector's shared graph is bounded by
  `maintenance_work_mem` (it stores raw f32 vectors). tqvector encodes all
  tuples before graph assembly; the DSM graph holds only neighbor-slot index
  arrays (~2–4 MB at 50k nodes × m=6). The overflow-to-serial fallback that
  costs pgvector its parallel benefit does not apply.

- **Shorter lock hold per candidate.** Score computation is
  `score(codes[a], codes[b])` via in-process SIMD kernels — no heap fetch, no
  distance dispatch. The neighbor-slot write lock is held for less time per
  candidate than pgvector's equivalent.

**Determinism posture:** concurrent insertion produces nondeterministic neighbor
selection by insertion order. This is accepted, consistent with ADR-042's
tolerance language and with how the live INSERT path already behaves. Level
assignment remains deterministic (pre-computed). The acceptance criterion is
recall quality on the existing gates, not byte-level reproducibility.

**Partitioned graph assembly is rejected** (see ADR-048). Cross-partition
navigability is an uncharacterized recall risk requiring infrastructure
(corpus DSM, patch format, merge algorithm) before speedup can even be
measured. Concurrent insertion starts faster and has stronger recall by
construction.

## Current State (after packets 618–631)

- `amcanbuildparallel = true` for `ec_hnsw`.
- Workers do parallel heap scan + tuple encoding → shm_mq → leader.
- Leader sorts/deduplicates (O(N) after packet 627) and assembles graph
  serially.
- 50k × 64: serial ~30s, parallel ~31s. Graph assembly is ~94% of wall time.
- The shm_mq ingestion path is correct and remains in place. Graph assembly is
  what needs to change.

## Phase Plan

### Phase 1 — DSM graph pre-assembly (NEXT)

Implement the pre-assembly phase that must complete before worker insertion
starts:

1. **Level pre-computation.** Before worker launch, iterate all nodes in
   `heap_tuples` and call `choose_insert_level_for_page_size` for each. Store
   levels in a `Vec<u8>` alongside the existing `heap_tuples`. This is already
   done implicitly per-node during serial graph assembly; make it explicit and
   batch it before DSM setup.

2. **Entry point selection.** Scan pre-computed levels for the first node at the
   maximum level. Store as a fixed `entry_idx: usize`. No lock needed — this
   value does not change during parallel insertion.

3. **DSM node array allocation.** Allocate a flat array of `NativeBuildNode`
   structs in DSM, pre-sized to `heap_tuples.len()`. Each node's
   `neighbor_slots` is initialized to empty. Each node gets one
   `LWLock` initialized via `LWLockInitialize`. Use `BUFFERALIGN` sizing
   matching the existing DSM layout helpers in `build_parallel.rs`.

4. **Gate behind `EcHnswBuildGraphAssembly::ConcurrentDsm`.** Keep
   `SerialLeader` as default. New variant is opt-in for this phase.

Validation target: single-participant (leader only, `requested_workers = 0`)
round-trip that produces a valid index through the existing page-staging path.

### Phase 2 — Worker insertion loop

Wire workers into graph insertion:

1. **Worker callback change.** Instead of encoding to shm_mq, workers encode
   the tuple and immediately call the graph insertion path against the DSM node
   array.

2. **Beam search with shared-read neighbor access.** The existing
   `search_native_layer_result_candidates` loop reads neighbor slots under
   `LWLockAcquire(slot_lock, LW_SHARED)`. Release before scoring candidates.
   This is the read-heavy hot path; shared reads do not block each other.

3. **Forward slot write.** After candidate selection, write the chosen
   neighbor indices into the node's slot under
   `LWLockAcquire(slot_lock, LW_EXCLUSIVE)`.

4. **Backlink write.** For each selected neighbor, acquire that neighbor's slot
   lock exclusive, run backlink pruning, write. This is the contention surface
   — acquire order must be deterministic (sorted by node index) to avoid
   deadlock.

5. **Leader participates.** With the shm_mq drain architecture replaced by
   direct DSM insertion, the leader is free to take a parallel heap scan
   partition. Set `leader_participates = true` in the plan. This recovers the
   ~10–15% heap-ingest time the leader currently spends idle.

6. **Remove shm_mq streams** for the `ConcurrentDsm` path. The per-worker
   queue handles and drain loop are specific to the ingestion coordinator and
   are not needed when workers insert directly.

Validation target: multi-worker PG18 test that verifies all heap TIDs are
present, entry point is valid, and recall meets the existing gate.

### Phase 3 — Recall and speed measurement

Before enabling `ConcurrentDsm` as the default:

1. Run the existing real-corpus recall gate (50k, real embeddings) with 4
   workers. Record the recall delta vs. serial native build. Gate: no regression
   beyond the documented tolerance.

2. Run build-time measurement at 50k and 500k. Target: parallel build
   materially faster than serial on the graph assembly phase, not only heap
   ingestion.

3. Record both results in a measurement packet before switching the default.

### Phase 4 — Default switch and cleanup

1. Set `ConcurrentDsm` as the default graph assembly variant in the build plan.
2. Remove the shm_mq ingestion coordinator code paths that are superseded
   (or keep as fallback for `build_source_column` builds which still use the
   serial path).
3. Update `amcanbuildparallel` docs and any pg_test smoke tests that assert
   the worker count or timing surface.

### Phase 5 — Scale measurement

- Build-time curves at 1M / 10M rows at 2/4/8 workers.
- Target: ≥2× at 4 workers on 1M, ≥4× at 8 workers on 10M (adjust after
  Phase 3 baselines).

## Key Files

- `src/am/ec_hnsw/build_parallel.rs` — coordinator; DSM setup, worker launch,
  graph-assembly new path goes here alongside existing ingestion coordinator.
- `src/am/ec_hnsw/build.rs` — native graph builder; `NativeBuildNode`,
  `NativeBuildLayerSearchScratch`, `NativeBuildVisitedSet` are the structures
  workers will use with their local scratch copies against the shared DSM graph.

## Lock Ordering

To avoid deadlock during backlink writes:

- A worker writing backlinks to nodes M₁, M₂, ..., Mₖ must acquire their
  LWLocks in ascending node-index order.
- This is the same invariant as ADR-026 (live insert backlink lock ordering).
  The build coordinator should document this explicitly even though build has no
  concurrent readers during the assembly phase.

## Owns

- `spec/adr/ADR-048-parallel-hnsw-build-graph-assembly.md` — DECIDED.
- `src/am/ec_hnsw/build_parallel.rs` — DSM graph extension.
- `src/am/ec_hnsw/build.rs` — any shared worker scratch helpers.

## Dependencies

- Task 19 heap-ingestion coordinator: landed. Provides worker launch,
  DSM sizing helpers, WAL/buffer accounting, parallel scan setup.
- ADR-042 native build path: landed. Workers use the same scoring kernels and
  layer-search scratch as the serial builder.
- Task 18 (parallel scan): no longer a hard blocker. The build coordinator
  does not share `src/am/common/parallel.rs`.

## Out of Scope

- GPU-accelerated build (ADR-046).
- Parallel vacuum.
- Parallel scan (task 18).
- `ec_diskann` propagation — after `ec_hnsw` parallel build stabilizes.
