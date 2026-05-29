# Task 65b: DiskANN Build — Parallel Vamana Graph Construction

Status: **proposed** 2026-05-28. Direct follow-up to Task 65
(`plan/tasks/65-diskann-build-perf-vamana-core.md`) and the
performance assessment in
`plan/design/diskann-build-performance.md` (P0 issue #6 option 2/3).

Owner: coder (to be assigned). Single coder, single branch.
Depends on Task 65 closing first — this task assumes the
algorithmic core (single-pass growing-α, `SearchScratch`,
bounded-heap frontier, rayon-encode) is already landed and
stable.

## Why

Task 65 brought single-process DiskANN build on real-10k from
70.678 s to 8.44 s (8.4×). Per-phase breakdown post-Task-65:

| phase | time | parallelised? |
|---|---:|---|
| heap scan | ~1.3 s | no (Postgres) |
| training | ~0.13 s | no |
| codec encode | ~0.1 s | yes (rayon) |
| **Vamana graph construction** | **~7 s** | **no** |
| page writes | ~0.04 s | no |

Graph construction is now ~83% of build wall time and remains
fully single-threaded. The rayon dependency added in Task 65 is
underused: it only parallelises the trivial encode step.

pgvectorscale's reference DiskANN build on similar 10k×1536d
corpora typically lands in 1–3 s with parallel workers enabled.
Our single-process post-Task-65 number is ~3–8× off that floor;
closing most of that gap requires parallel graph construction.

## Why this is hard

The Vamana paper's serial insertion loop has a hard dependency:
each pivot insertion reads the current graph state (existing
neighbours of the candidate set) and writes its own out-edges +
backlinks. Naïve parallelisation breaks two invariants:

1. **Greedy search visits stale neighbours.** Pivot B's greedy
   search may traverse pivot A's stale out-edges while A is still
   writing them. The recall floor of the final graph depends on
   how stale the reads are.
2. **Backlink writes race.** Two pivots writing to a shared
   neighbour's adjacency list at the same time corrupt the list
   without locking. Naïve `Mutex<Vec<u32>>` per node serialises
   the hot nodes (the medoid and high-degree hubs) — that's
   exactly where contention is worst.

pgvectorscale's solution (`pgvectorscale/src/access_method/build.rs:1146-1151`):
- Per-worker batches of pivot insertions.
- Shared `BuilderNeighborCache` (LRU; `graph/neighbor_store.rs:50`)
  caches neighbour reads/writes.
- Periodic flush of the cache to the paged graph store at a
  configurable interval (`flush_rate`).
- Postgres `ParallelContext` workers (`build.rs:352`) with shared
  DSM state and a `ConditionVariableBroadcast` barrier for
  initialisation.

Our HNSW parallel build (`src/am/ec_hnsw/build_parallel.rs`, 4204
LOC) uses a different but related pattern: Postgres
`ParallelContext` + DSM-resident node insert state + per-node
condition-variable handoff via `EcHnswConcurrentDsmInsertStateCell`.
That infrastructure cannot be reused wholesale for Vamana because
HNSW's level structure and entry-point semantics differ, but the
DSM scaffolding, the `ParallelContext` lifecycle, the
`HeapRelationGuard`/`IndexRelationGuard` wrappers, and the WAL/
buffer usage accounting are all directly applicable.

## Goal

Cut real-10k DiskANN build wall time on a 4-core machine to
≤ 3 s while:

- holding real-10k Recall@10 within 0.5 percentage points of the
  Task 65 post-fix baseline,
- maintaining deterministic-or-explicitly-documented build output
  for a fixed seed + worker count,
- not regressing single-process build time when parallel build is
  disabled (worker count = 0 falls back to Task 65's path).

## Scope

### In scope

1. Parallel graph construction loop.
2. Shared neighbour cache (read-coherent enough for the recall
   gate; need not be strictly serialisable).
3. Worker coordination — choose ONE of:
   - **(Recommended) Postgres `ParallelContext` workers**, matching
     HNSW's `build_parallel.rs` pattern. Reuse `EcHnswConcurrentDsmInsertStateCell`-style
     scaffolding where applicable. This keeps the parallel mechanism
     consistent across AMs.
   - **Rayon thread pool**, simpler but lives outside Postgres's
     parallel-worker accounting (no `pg_stat_progress_create_index`
     integration, no WAL/buffer attribution per worker, no
     `pg_stat_activity` visibility). Acceptable as a stepping
     stone if `ParallelContext` integration proves too large.
4. Per-pivot batch sizing and flush cadence as a tuned reloption
   (default: TBD by measurement; pgvectorscale uses `flush_rate`).
5. Recall gate enforcement and determinism handling (see below).
6. New reloption(s) for parallel-build control (worker count,
   batch size, flush rate) on `ec_diskann`. Existing `ec_hnsw`
   reloption naming is the precedent.
7. Build-side logging: per-worker timing, contention counters,
   stale-read fraction.
8. Validation packets matching the Task 65 shape.

### Out of scope

- Persist layer changes (Task 65 deferred this; deferred again
  here).
- SIMD distance kernel changes (Task 29d already did AVX2+FMA;
  NEON is wired).
- Changes to runtime insert / scan path.
- Cross-AM codec changes.
- Distributed / GPU build (ADR-046 deferred follow-up).

## Determinism decision

This is the load-bearing design call for this task. Three options:

1. **Accept nondeterminism.** Build output depends on worker
   scheduling. Same seed + same worker count + same corpus on
   the same machine may produce different graphs across runs.
   Recall@k is statistically equivalent across runs but the
   `bo_007_deterministic_for_fixed_seed`-style golden tests are
   abandoned. Lowest implementation cost.
2. **Deterministic-by-construction.** Each worker owns a fixed
   pivot range; reduction order is fixed. Backlink races are
   resolved by a deterministic tiebreaker (e.g. lowest node id
   wins). Higher complexity, slightly lower throughput, but
   golden tests survive.
3. **Deterministic reduction after parallel proposal.** Workers
   propose edges; a sequential reduction phase commits them in
   pivot order. Cheapest determinism but adds a sequential
   bottleneck that limits scaling.

**Pick a default and call it out in the first packet.** Reviewer
recommendation: option 2 unless measurement shows it's >20%
slower than option 1, then renegotiate.

## Slice plan

Narrow, testable slices. Each its own commit + packet.

- **Slice A — measurement floor.** Per-phase timing split on
  current Task 65 head for real-10k, real-100k, synth-10k. This
  is the reference number every later slice compares against.
  No code change.
- **Slice B — shared neighbour cache.** Single-threaded
  introduction of the cache abstraction (read-through to the
  in-memory graph, write-back to the same). Confirms no recall
  regression vs Task 65 head; isolates the abstraction cost.
- **Slice C — locking design.** RFC-style packet documenting the
  chosen locking strategy (per-node `RwLock<Vec<u32>>`, sharded
  lock array, lock-free CAS on small adjacency Vecs, or other),
  with a contention model on hub-node degree distribution from
  Slice A's data. No code change.
- **Slice D — parallel worker scaffolding.** ParallelContext (or
  rayon) wired in with worker count = 1; output identical to
  Task 65 head, no perf change. This is the boundary slice that
  proves the scaffolding works before correctness gets stressed.
- **Slice E — multi-worker correctness.** Worker count = 4 with
  the chosen determinism strategy. Recall gate enforced on every
  fixture. This is where the design call from "Determinism
  decision" gets cashed.
- **Slice F — flush cadence + batch tuning.** Measurement-driven
  sweep over `flush_rate` × `batch_size` × `worker_count` on
  real-10k and real-100k. Land defaults.
- **Slice G — fallback path.** Worker count = 0 reloption falls
  back to Task 65's single-process path. Smoke check on the
  matrix.
- **Slice H — measurement packet.** Release-mode timing split,
  recall delta, per-worker scaling curve up to host core count.

## Validation gate

1. **Recall.** Real-10k Recall@10 within 0.5pp of Task 65 head.
   Synth-10k held within the explicitly documented Task 65
   envelope (the regression noted in Task 65 packet 002
   feedback must already be addressed before this task starts).
2. **Performance.** Real-10k build ≤ 3 s on a 4-core host.
   Real-100k build ≤ 30 s. Per-worker scaling curve published.
3. **Determinism.** Per the option chosen in the design call:
   either documented nondeterminism with statistical recall
   bounds, or bit-equal output for a fixed seed + worker count.
4. **Functional.** All existing `ec_diskann` tests pass; new
   tests cover the parallel path; concurrency tests cover the
   neighbour-cache + locking surface.
5. **Fallback.** Worker count = 0 produces byte-equal output and
   timing within 5% of Task 65 head.
6. **Postgres integration** (if option ParallelContext): WAL +
   buffer usage attributed to workers; `pg_stat_progress_create_index`
   shows progress; `pg_stat_activity` lists the workers.

## Coder workflow notes

- **Branch off the Task 65 close commit.** Do not start this
  task until Task 65's B1 (`tuple_is_alive` revert) and B2
  (synth-10k recall fix) are merged.
- **Push every slice.** Slices D and E in particular will
  interact with reviewer comments more than usual; keep the
  remote head in sync.
- **Concurrency tests are mandatory at slice E.** Borrow the
  `loom` or `shuttle`-style model checking already used by
  HNSW (`src/am/ec_hnsw/concurrent_dsm_state.rs`) if applicable.
  Per memory `feedback_dont_defer_safety_fixes`, any data race
  is a blocker — do not label it "optional follow-up".
- **No new `unsafe` outside the ParallelContext scaffolding.**
  The neighbour-cache and locking design must be safe-Rust
  unless the FFI surface requires it. Per memory
  `feedback_anti_pattern_b_unbounded_lifetime`, no
  `fn(*mut T) -> &'a T` wrappers; use typed views or inline
  `NonNull::as_ref()` at call sites.
- **macOS pgrx-test blocker.** Same situation as Task 65 — the
  `_BufferBlocks` dyld issue and now also the `Operation not
  permitted` install failure mean validation falls back to
  Linux CI or the ecaz CLI corpus path for slices that touch
  the Postgres callback surface.

## References

- Task 65 (predecessor): `plan/tasks/65-diskann-build-perf-vamana-core.md`
- Investigation: `plan/design/diskann-build-performance.md`
- HNSW parallel-build (reference implementation):
  `src/am/ec_hnsw/build_parallel.rs` (4204 LOC),
  `src/am/ec_hnsw/concurrent_dsm_state.rs`
- HNSW parallel-build task: `plan/tasks/26-parallel-index-build.md`,
  `plan/tasks/58-hnsw-build-parallel-p8-consumer-migration.md`
- pgvectorscale (read-only): clone the upstream
  `timescale/pgvectorscale`; key files are
  `pgvectorscale/src/access_method/build.rs:352-1151`,
  `pgvectorscale/src/access_method/graph/neighbor_store.rs`,
  `pgvectorscale/src/access_method/build/parallel.rs`.

## Acceptance criteria

1. Parallel Vamana build lands behind a reloption.
2. All gates above pass.
3. Measurement packet documents real-10k, real-100k, and a
   per-worker scaling curve up to host cores.
4. Determinism decision is documented; if nondeterministic,
   the recall confidence interval is published.
5. No regression in DiskANN insert or scan tests.
6. Fallback path (worker count = 0) matches Task 65 head
   byte-for-byte.

## Estimated size

Large. The HNSW analogue (`build_parallel.rs`) is 4200 LOC and
Vamana's parallel semantics are harder than HNSW's level-by-level
structure. Expect 3–6 weeks for a single coder including
measurement and concurrency-test work.
