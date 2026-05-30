# Task 71: IVF Parallel Build (`amcanbuildparallel = true`)

Status: proposed
Owner: coder (to be assigned). One coder, one branch.
Priority: 2 (high-value but smaller cross-engine impact than Task 70)

## Why

`ec_ivf` declares (`src/am/ec_ivf/routine.rs:28-29`):

```rust
amroutine.amcanparallel = false;
amroutine.amcanbuildparallel = false;
```

IVF builds are fully single-process today. By contrast:

- `ec_hnsw` has `amcanbuildparallel = true` (Task 26 ADR-048
  ConcurrentDsm landed).
- `ec_diskann` has `amcanbuildparallel = false` but Task 65b is
  drafted to address that.
- IVF has no equivalent task.

Task 69 (Common Training Parallelism) lifted `ec_ivf`'s build path
indirectly — `train_spherical_kmeans` and
`assign_vectors_to_centroids` are now rayon-parallel under the
hood. That captures the training and bulk-assignment phases. What
remains single-process is the **PostgreSQL build callback**: heap
scan via `table_index_build_scan`, per-tuple staging into
`BuildState.heap_tuples`, and the posting-flush walk.

A PG-parallel-build path for `ec_ivf` (mirror HNSW's
`src/am/ec_hnsw/build_parallel.rs`) would let PG split heap scan
across workers, with each worker accumulating into a thread-local
build state, then merge into the centroid + posting flush in the
leader. The trained k-means model and centroid plan are the same
across workers (deterministic at fixed seed); only the
per-vector assignment fan-out and posting accumulation need
parallelisation.

This task does **not** parallelise scan (`amcanparallel`). Only
build (`amcanbuildparallel`). Parallel scan is a separate concern
and has historically been shelved per `plan/tasks/18-parallel-index-scan.md`.

## Non-Goals

- Do not set `amcanparallel = true`. Parallel scan stays out of
  scope.
- Do not change IVF on-disk format. Workers must produce the
  same posting-list / centroid / directory pages as the
  single-process path.
- Do not change deterministic centroid output. At fixed seed and
  fixed sample, the trained model must be byte-identical to the
  current build (Task 69's parallel k-means already guarantees
  this).
- Do not change posting-list assignment semantics. Each heap row
  must land in the same posting list under parallel vs
  single-process build.
- No AWS / Graviton work — M5 is the local optimization host.

## Phase 1 — Design Decision (gating)

Land one design packet that picks the parallel-build shape before
any code change:

- **Option A — Per-worker sample collection, leader-side training
  and assignment.** Workers collect heap tuples into shared memory
  via `shm_mq` (similar to HNSW's pre-DSM shape). Leader trains
  k-means once on the merged sample, then assigns + flushes in the
  leader. Maximises worker utilisation for heap scan, but leaves
  assignment + flush serial.
- **Option B — Per-worker sample + per-worker assignment, leader-side
  centroid merge and flush.** Workers collect samples and call the
  Task 69 parallel `assign_vectors_to_centroids` on their slice
  using a leader-broadcast centroid model. Posting-list staging
  per worker; leader merges + writes posting pages. Higher
  parallel coverage, more synchronisation surface.
- **Option C — ConcurrentDsm pattern (mirror Task 26 ADR-048).**
  Workers write directly to DSM-backed staging; leader publishes.
  Highest engineering cost; matches HNSW's current default but
  unclear if IVF has the same DSM-amenable structure.

The Phase 1 packet picks one based on:

- expected speedup (rough Amdahl bound on heap-scan share),
- engineering cost (per-worker state surface, DSM vs shm_mq),
- consistency with HNSW's `build_parallel.rs` patterns,
- determinism / fallback contract under failure.

## Phase 2 — Implementation Slices

Once Phase 1 picks an option, land in order:

1. **Routine flag flip + skeleton.** Set
   `amcanbuildparallel = true`. Wire `amestimateparallelscan`
   (PG18 callback) to produce a sensible cost-based parallel-build
   decision. Skeleton parallel `BuildState` per worker.
2. **Heap-scan parallelism.** Workers walk heap tuples in
   parallel via PG's standard parallel-scan callback. Per-worker
   thread-local accumulator (matching the chosen Phase-1
   option's shape).
3. **Merge + flush.** Leader merges worker accumulators, runs
   Task 69's parallel k-means + assignment helpers on the merged
   set, and writes posting pages in source order to preserve
   deterministic page layout.
4. **Determinism + recall validation.** At fixed seed, the
   parallel build must produce structurally-equivalent IVF
   directory + posting pages as the single-process build, and
   recall@10 must hold within 0.5 pp on the comparator fixtures
   Task 31 used (real10k/25k/50k/100k).

## Phase 3 — Measurement

Final measurement packet:

- Build wall time at requested workers 1/2/4/8 on real10k, 25k,
  50k, 100k (matching Task 31's fixture surface).
- Recall@10 at each worker count, fixed nprobe/rerank_width.
- Index size invariance (must be byte-identical or
  structurally-equivalent across worker counts).
- Worker launch counter (use the same approach Task 33
  packets 002/003 used — likely
  `pg_stat_get_db_parallel_workers_launched`).
- Memory HWM during build (per the Task 33 packet 002 instrumentation
  gap, this may need a sampler wrapper; record `not_measured` if
  unavailable rather than emitting zero values).

## Exit Criteria

- Phase 1 design packet landed with reviewer-approved option choice.
- Phase 2 slices landed with `amcanbuildparallel = true` and
  end-to-end parallel build working.
- Phase 3 measurement shows multi-× build-time win at workers ≥ 2,
  with regression behaviour documented at workers ≥ N where
  applicable (mirror Task 33's worker-curve analysis).
- Determinism preserved: same-seed builds produce byte-equal or
  structurally-equal index pages across worker counts.
- Recall floor preserved: real10k/25k/50k/100k recall@10 within
  0.5 pp of Task 31 baseline numbers at the same comparator
  points.
- No new `unsafe { ... }` blocks outside the PG callback boundary
  the existing IVF build code already lives in.
- `cargo clippy --all-targets --no-default-features --features pg18
  -- -D warnings` clean.
- Closeout packet citing Phase 1 + Phase 2 + Phase 3 evidence
  flips `plan/tasks/71-…md` status to `complete`.

## Coordination

- **Task 69 is a hard dependency** for any per-worker assignment
  slice (Phase 2 step 3). It is closed — proceed.
- **Task 31 is closed.** This task picks up the build-side
  parallelism IVF still lacks; Task 31's scan/scoring/rerank work
  was the query-side optimization.
- **Task 57 (IVF unsafe burndown)** overlaps the IVF source
  surface. Coordinate so this task's parallel-build commits land
  before Task 57's unsafe lifts to avoid merge conflict on
  `ec_ivf/build.rs`. If Task 57 opens first, this task should
  rebase on top.
- **HNSW build_parallel.rs is the reference**. Read
  `src/am/ec_hnsw/build_parallel.rs` and ADR-048 before Phase 1
  design.
- Honor memory `feedback_dont_defer_safety_fixes` and
  `feedback_anti_pattern_b_unbounded_lifetime` in review.

## Stop Conditions

- Stop if Phase 1 design shows the per-worker overhead is
  comparable to the heap-scan speedup at M5 corpus sizes (i.e.
  parallel build is a wash on local hardware). In that case,
  defer to a Graviton / cloud-class fixture before committing
  implementation cost.
- Stop if recall regresses beyond 0.5 pp on any comparator fixture
  during Phase 2 validation. The deterministic-output rule is
  load-bearing for IVF correctness.
