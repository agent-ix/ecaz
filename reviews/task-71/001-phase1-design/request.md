# Review Request: Task 71 Phase 1 IVF parallel build design

Task: `plan/tasks/71-ivf-parallel-build.md`

## Summary

This Phase 1 design packet selects **Option A: per-worker heap tuple collection, leader-side training and assignment** for the first `ec_ivf` parallel-build implementation.

The selected shape mirrors the proven HNSW parallel heap-ingestion coordinator, but deliberately does not import ADR-048's concurrent DSM graph assembly. IVF does not have an order-dependent mutable graph structure during build. Its current build path is:

1. scan heap tuples into `BuildState.heap_tuples`;
2. train the coarse k-means model from a deterministic sample;
3. optionally train the `pq_fastscan` grouped-PQ model from the same deterministic sample;
4. assign every tuple to a centroid;
5. stage and flush centroid, posting, directory, and metadata pages in list order.

The load-bearing deterministic boundary is the ordered merged tuple corpus. Once the leader has that corpus, Task 69 already parallelizes the expensive training and assignment helpers with deterministic output, so the PostgreSQL worker phase should stay focused on splitting heap scan and tuple decoding.

## Decision

Implement Phase 2 as:

- set `ec_ivf.amcanbuildparallel = true` only after a working parallel build path is wired;
- add an IVF-local `build_parallel.rs` coordinator following HNSW's heap-scan worker model:
  - leader opens `ParallelContext`;
  - leader initializes a shared parallel table scan descriptor;
  - each worker opens heap/index relations, builds the same tuple representation as `ec_ivf_build_callback`, and sends encoded `BuildTuple` messages through a per-worker `shm_mq`;
  - workers send a terminal done message and aggregate heap/index tuple counts;
  - leader drains messages, sorts them by heap TID, validates them through `BuildState::try_push`, and falls back to the serial path if no workers launch;
- run the existing leader-owned `train_model`, optional `train_pq_fastscan_model`, `stage_build_plan`, and `flush_build_plan` sequence on the merged state.

The implementation should preserve the current page-staging contract. Posting pages remain written by the leader in centroid/list order, and tuple order within each list remains the same as the deterministic merged corpus plus `assign_vectors_to_centroids` output.

## Why Option A

**Expected speedup.** Option A captures the PostgreSQL-visible heap scan and tuple decoding work immediately. The remaining training and assignment phases are not purely serial anymore because Task 69 made `train_spherical_kmeans`, `train_grouped_pq4_model`, and `assign_vectors_to_centroids` rayon-parallel with deterministic output.

**Engineering cost.** IVF `BuildTuple` has a compact, direct wire shape: heap TID, dimensions, gamma, payload bytes, and source `f32` vector. That is smaller and lower risk than introducing DSM posting-list staging or cross-process centroid broadcasts before measurement proves the heap-scan split is insufficient.

**Consistency with HNSW.** HNSW already has a battle-tested PG parallel heap-scan pattern: shared `table_parallelscan_initialize`, per-worker `shm_mq`, worker relation reopen, WAL/buffer accounting, and leader fan-in. Reusing that boundary for IVF keeps the first code slice close to known-good infrastructure while avoiding HNSW-specific graph DSM complexity.

**Determinism.** Option A preserves IVF's stronger deterministic contract. Workers may scan heap pages in nondeterministic order, but the leader sorts merged `BuildTuple`s by `(block_number, offset_number)` before training and staging. The deterministic sample indices, k-means seed, centroid assignment, posting-list layout, directory entries, and metadata should therefore match the single-process path structurally.

**Fallback.** If `CreateParallelContext`, DSM setup, or worker launch yields no workers, the build should use the existing serial path. That fallback must happen before any index pages are flushed.

## Rejected Options

**Option B: per-worker assignment after leader-broadcast centroid model.**

This needs at least two phases or a broader DSM surface. Workers cannot assign tuples until the model is trained, and the model cannot be trained until the sampled source vectors are collected. A second worker pass or a retained cross-process tuple corpus would add complexity before we know whether Option A's heap-scan fan-in plus Task 69's rayon assignment is enough.

**Option C: ConcurrentDsm pattern from ADR-048.**

ADR-048 is the right answer for HNSW because graph insertion mutates a shared neighbor structure and recall depends on global navigability during insertion. IVF build has no analogous concurrent graph mutation. The mutable output is page staging, which should remain leader-owned for deterministic layout and WAL simplicity. DSM posting staging can be reconsidered only if Phase 3 measurements show leader-side posting flush dominates after Option A.

## Phase 2 Checklist

- Add an IVF parallel-build coordinator module with HNSW-style `ParallelContext`, shared table scan, per-worker queues, and worker entrypoint.
- Add `BuildTuple` message encode/decode coverage, including source-vector float bit round-trip and malformed/truncated message rejection.
- Sort drained worker tuples by heap TID before pushing into `BuildState`.
- Keep `amcanparallel = false`.
- Flip `amcanbuildparallel = true` only in the slice that proves end-to-end PG18 parallel build works.
- Add worker launch instrumentation matching the HNSW debug counter pattern or a Task 33-compatible `pg_stat_get_db_parallel_workers_launched` measurement in the validation packet.

## Validation

No tests were run for this design-only packet.

Code validation belongs to Phase 2. The first implementation packet should run at least a focused PG18 build smoke that proves workers launch and the generated IVF index scans correctly, plus a focused Rust unit test for message serialization once the coordinator module exists.

## Review Questions

1. Is Option A the right first implementation shape for IVF, given Task 69's deterministic rayon training and assignment helpers?
2. Is the deterministic merge boundary sufficiently explicit for preserving same-seed page structure?
3. Should the first code slice include only heap-scan parallelism, or should it also introduce a diagnostic counter/GUC before the flag flip?
