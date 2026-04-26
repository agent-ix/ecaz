# Task 26: Parallel Index Build

Status: **in-progress** — concurrent DSM graph assembly is implemented and
default-on for eligible PG18 parallel builds; Phase 5 scale curves remain.

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

- **No raw-vector DSM overflow.** pgvector's shared graph is bounded by
  `maintenance_work_mem` (it stores raw f32 vectors). tqvector encodes all
  tuples before graph assembly; workers still need compact encoded code bytes
  in DSM for candidate scoring, but not raw f32 source vectors. The graph
  surface holds neighbor-slot index arrays (~2–4 MB at 50k nodes × m=6) plus
  the compact code corpus. The overflow-to-serial fallback that costs pgvector
  its parallel benefit does not apply.

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

## Current State (after packets 618–666)

- `amcanbuildparallel = true` for `ec_hnsw`.
- Workers do parallel heap scan + tuple encoding → shm_mq → leader for the
  original ingestion coordinator path.
- The leader sorts/deduplicates (O(N) after packet 627), then eligible PG18
  parallel builds default to concurrent DSM graph assembly.
- Concurrent DSM graph assembly precomputes levels, fixes the entry point,
  allocates a DSM graph/corpus surface, and has workers insert node partitions
  into the shared graph behind per-node LWLocks.
- The diagnostic GUC `ec_hnsw.enable_parallel_build_concurrent_dsm` can disable
  the concurrent DSM graph path and force the old serial-leader graph assembly.
- Real 50k source-scored summary (packet 666): serial build `30:15.962`,
  best concurrent DSM build `03:17.371`, recall@10 `0.91` / `0.91`.
- The remaining shm_mq ingestion path is not yet superseded for every build
  shape. Removing it requires direct worker-to-DSM tuple ingestion or an
  explicit fallback split; do not delete it as a cleanup-only change.

## Phase Plan

### Phase 1 — DSM graph pre-assembly (DONE)

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

3. **DSM graph/corpus allocation.** Allocate the compact encoded code corpus,
   flat graph node array, and flat neighbor-slot array in DSM. Each node is
   pre-sized from the level plan, starts with empty neighbor slots, and gets one
   `LWLock` initialized via `LWLockInitialize`. Use `BUFFERALIGN` sizing
   matching the existing DSM layout helpers in `build_parallel.rs`.

4. **Gate behind `EcHnswBuildGraphAssembly::ConcurrentDsm`.** Keep
   `SerialLeader` as default. New variant is opt-in for this phase.

Validation target met: single-participant and attachment/page-staging coverage
landed in the concurrent DSM packet chain.

### Phase 2 — Worker insertion loop (DONE)

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

6. **Remove shm_mq streams** for a future direct-ingestion path. The current
   default still uses the existing heap-ingestion coordinator to collect encoded
   tuples before concurrent DSM graph assembly. Direct worker-to-DSM ingestion
   is a separate follow-up because source-scored builds and fallback behavior
   still need an encoded corpus boundary.

Validation target: multi-worker PG18 test that verifies all heap TIDs are
present, entry point is valid, and recall meets the existing gate.

### Phase 3 — Recall and speed measurement (DONE for real 50k)

1. Run the existing real-corpus recall gate (50k, real embeddings) with 4
   workers. Record the recall delta vs. serial native build. Gate: no regression
   beyond the documented tolerance. Packet 666 records recall@10 parity
   (`0.91` / `0.91`).

2. Run build-time measurement at 50k and 500k. Target: parallel build
   materially faster than serial on the graph assembly phase, not only heap
   ingestion. Packet 666 records the 50k result: wall-clock speedup about
   `9.20x`, graph-phase speedup about `10.77x`. The 500k curve remains part of
   Phase 5 scale measurement.

3. Record results in a measurement packet before switching the default. Packet
   666 is the Phase 3 real-50k source of truth.

### Phase 4 — Default switch and cleanup (PARTIAL)

1. Set `ConcurrentDsm` as the default graph assembly variant in the build plan.
   Done in packet 665; the GUC remains as a diagnostic fallback.
2. Remove only shm_mq ingestion coordinator code paths that are truly
   superseded. Not done: the queue/drain path still owns parallel heap
   ingestion for the non-direct DSM ingestion path, and is still the safe
   fallback boundary.
3. Update `amcanbuildparallel` docs and any pg_test smoke tests that assert
   the worker count or timing surface. PG18 smoke tests were updated with the
   default switch; requirements/docs are being aligned after packet 666.

### Phase 5 — Scale measurement

- Real 50k worker sweep is recorded in packet 668:
  - 1 worker: `07:12.017`, `graph_us = 395621949`
  - 2 workers: `04:59.790`, `graph_us = 268137745`
  - 4 workers: `03:24.964`, `graph_us = 173200231`
  - 8 requested workers launched 7 and regressed: `04:08.671`,
    `graph_us = 216938590`
- Packet 672 reruns the 8-worker point after raising PG18 worker-process
  headroom to `max_worker_processes = 16`, `max_parallel_workers = 16`, and
  `max_parallel_maintenance_workers = 8`:
  - 8 workers launched 8 graph workers and finished in `02:27.948`,
    `graph_us = 116850823`
- Current best real-50k point is 8 workers when the PG18 cluster has enough
  worker-process headroom. Packet 668's regressed 8-worker point should be
  treated as a cluster-limit diagnostic, not a scaling conclusion.
- Next build-time curve target is the DBPedia 990k/10k profile
  (`ec_hnsw_real_ann_benchmarks_anchor`) once chunked prepare/load support from
  Task 10066 is available, or after a one-shot non-resumable load if the
  operator accepts the restart risk.
- Longer-horizon target: 10M rows at 2/4/8 workers.
- Original target remains directionally useful but should be recalibrated after
  the 990k run: ≥2× at 4 workers on 1M, ≥4× at 8 workers on 10M.

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
