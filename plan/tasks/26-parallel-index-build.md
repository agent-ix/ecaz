# Task 26: Parallel Index Build

Status: proposed — **waits on task 18 (parallel scan) to settle the DSM/worker-slot
shape**. ADR TBD; draft alongside Phase 0.

## Scope

Enable multi-worker index build for `ec_hnsw` (and by extension `ec_diskann` and
any future AM sharing the build seam). A single `CREATE INDEX` / `REINDEX`
splits the heap scan, quantizer training sample, graph-construction candidate
sets, and flush work across Postgres parallel workers under a shared
coordinator.

Goal: near-linear build-time reduction on 1M–100M-row corpora for 2/4/8
workers, so that the native-build lane's throughput wins (ADR-042) compound
with parallelism instead of saturating a single core.

Separate from task 18 (parallel *scan*). Shares the DSM/worker-slot machinery
in `src/am/common/parallel.rs`; does not share coordinator semantics (scan
coordinates a top-K heap; build coordinates graph construction).

## Why now

- Native-build lane (ADR-042) is landing single-threaded. The bigger the
  corpus we demonstrate it on, the more a parallel build is the next obvious
  unlock — GPU path (ADR-046) is months out and CPU parallelism is shipped
  first.
- Task 18 is establishing the DSM / worker-slot / claim-aware-drain pattern
  for scan. The same primitives generalize to build; starting now (before
  task 18's coordinator settles) risks two coordinators with divergent
  conventions.
- No current Postgres vector extension parallelizes index build. Pairs with
  task 18's differentiator narrative.
- Build-time is the #1 friction in the user survey for sub-100M corpora;
  latency (task 18) is #1 above that scale.

## Risk: conflict with task 18

Task 18 is actively reshaping `src/am/common/parallel.rs` — DSM layout, worker
slot claiming, coordinator drain, shared heap frontier. Any parallel-build
work that touches the same file before task 18 merges forces sed-pass merges
on both sides. Disposition:

- **Phase 0 (design) and Phase 1 (prototype in a scratch module) can start
  now.** Neither edits `src/am/common/parallel.rs`.
- **Phase 2 onward waits on task 18's coordinator merge.** When it lands,
  rebase, identify the shared primitives (DSM sizing helpers, slot claiming,
  barrier shape), and factor those out *before* adding build-specific
  coordinator logic.

## Design outline

ADR TBD. Working sketch:

- **Heap-scan split.** Leader assigns heap block ranges to workers; each
  worker accumulates its own partial sample for quantizer training and its
  own partial candidate set for graph construction.
- **Quantizer training.** Workers emit per-block training samples to a
  shared reservoir; leader (or last-worker-finished) trains the final
  codebooks. Training is a barrier between the scan phase and the graph
  phase.
- **Graph construction.** The contentious part. Two candidate shapes:
  1. **Partition-and-stitch.** Each worker builds a subgraph over its
     assigned vectors; leader stitches cross-partition edges at the end.
     Simpler, weaker recall if stitching is naive.
  2. **Shared graph with per-worker frontiers.** Workers insert into a
     shared graph guarded by fine-grained neighbor-list locks; each worker
     maintains its own beam-search frontier against the partially-built
     graph. Closer to how task 18's scan coordinator works, stronger
     recall, more contention to tune.
  Pick in Phase 0 based on prototype numbers; do not freeze before
  prototyping both on a 1M seam.
- **Flush.** Per-worker page-building with a leader-side final ID
  assignment, reusing the native-build flush (ADR-042).
- **Correctness invariant.** With `max_parallel_maintenance_workers = 0` the
  parallel path must produce a byte-identical index to today's serial path.
  Enforced via a build-mode test that hashes the output pages.

## Subtasks

### Phase 0 — design and ADR draft

- [ ] **ADR draft.** New ADR (number TBD) with the graph-construction
  shape decision, the relationship to ADR-040 (task 18), and the
  correctness invariant. Block Phase 2 on ADR acceptance.
- [ ] **Benchmark baselines.** Current single-threaded build time at
  100k / 1M / 10M on the real seams, so the parallel win is measured
  against a moving native-build target rather than the pre-ADR-042 number.

### Phase 1 — scratch prototype (no shared file edits)

- [ ] **Scratch module.** `src/am/common/parallel_build_scratch.rs` (or
  similar throwaway name). Implements the chosen graph-construction shape
  against a synthetic driver. Zero dependency on `parallel.rs`; copy any
  primitives needed rather than sharing them.
- [ ] **Partition-and-stitch prototype.**
- [ ] **Shared-graph prototype.** On a 1M seam; measure recall delta vs.
  serial native build and wall-clock speedup at 2/4/8 workers.
- [ ] **Decision.** Record in the ADR draft.

### Phase 2 — shared infrastructure (gated on task 18 merge)

- [ ] **Rebase onto task 18.** Identify DSM-sizing / slot-claiming helpers
  that generalize; factor into a shared module. Do not regress task 18's
  scan path.
- [ ] **Shared build-coordinator primitives.** DSM layout, barrier, per-
  worker slot; distinct from task 18's top-K heap coordinator but sharing
  the claim/drain pattern.

### Phase 3 — AM callback wiring

- [ ] **`amcanbuildparallel`** (or whichever PG name applies) in the
  `IndexAmRoutine` for `ec_hnsw`.
- [ ] **Leader path.** `ambuild` driver that requests workers, assigns
  heap ranges, orchestrates the quantizer-training barrier, and finalizes
  the flush.
- [ ] **Worker entry point.** `_PG_parallel_build_main`-style entry, DSM
  attach, per-worker scan + partial-graph build.
- [ ] **Fallback.** Single-worker path remains intact; degrade cleanly
  when `max_parallel_maintenance_workers = 0` or DSM allocation fails.

### Phase 4 — measurement

- [ ] **Build-time curves.** 1M / 10M / 100M on the real seams at 1/2/4/8
  workers. Target: ≥3× at 4 workers on 10M, ≥5× at 8 workers on 100M
  (adjust targets after Phase 0 baselines).
- [ ] **Recall parity.** Parallel-built index must not regress recall@10
  vs. serial build at the same `m / ef_construction`.
- [ ] **Byte-identical single-worker.** Enforced test.

### Phase 5 — DiskANN propagation

- [ ] **`ec_diskann` adoption.** Once `ec_hnsw` parallel build stabilizes,
  wire the same infrastructure through DiskANN's `ambuild` (task 17).
  Shares the shared-graph coordinator; DiskANN-specific build phases
  (Vamana pruning, SSD layout) plug in on top.

## Owns

- New ADR (number TBD).
- `src/am/ec_hnsw/build*.rs` (parallel path additions).
- Shared build-coordinator primitives in `src/am/common/` (post-task-18
  rebase).

## Dependencies

- **Hard blocker for Phase 2:** task 18 (parallel scan) coordinator
  merged. Starting Phase 2 earlier triggers the brutal-merge surface in
  `src/am/common/parallel.rs`.
- **Hard blocker for Phase 3:** native-build lane (ADR-042) merged.
  Parallelizing the old CPU path is not worth the effort.
- **Soft dependency:** task 17 (DiskANN) stable enough that Phase 5 has
  a target to propagate into.

## Unblocks

- Practical 100M-row build times on commodity hardware without waiting for
  the GPU trainer (ADR-046).
- A second Postgres vector differentiator: no competitor parallelizes
  index build today.
- Symphony AM (ADR-045 Stage 2) builds inherit parallelism for free if
  they ride the `ec_hnsw` build seam.

## Out of scope

- GPU-accelerated build (ADR-046). Orthogonal; CPU parallel and GPU
  offline can coexist.
- Parallel vacuum (ADR-042 touches vacuum but not in parallel form).
- Parallel insert. Different coordination shape (`aminsert` is row-at-a-
  time on the writer connection); not this task.
- Parallel scan — task 18.

## Notes

- **Do not start Phase 2 before task 18 merges.** The cost of a divergent
  DSM/worker-slot convention is higher than the schedule slip.
- **Prototype both graph-construction shapes before the ADR freezes.**
  The partition-and-stitch shape is cheaper to implement and may be
  good enough; the shared-graph shape is the stronger-recall ceiling.
  Data decides, not taste.
- **Byte-identical single-worker is non-negotiable.** It is the only way
  to keep the test suite honest across the transition.
