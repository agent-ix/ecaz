# Task 72: SPIRE Parallel Build (Post-Task-68 Follow-On)

Status: proposed
Owner: coder (to be assigned). One coder, one branch.
Priority: 3 (largest engineering surface of the three parallel-build tasks)

## Why

After Task 68 closed, the SPIRE 100K build wall-time split looks
like (from `reviews/task-68/005-zero-replica-fast-path-measurement/`
and the closeout packet 008):

| phase | 100k ms | % of 3418 ms |
| --- | ---: | ---: |
| heap_scan | 1307 | 38 % |
| top_graph | 946 | 28 % |
| assignment | 574 | 17 % (Task 69 parallel) |
| kmeans | 490 | 14 % (Task 69 parallel) |
| draft (post-fast-path) | 92 | 3 % |
| publish | 8 | 0.2 % |

After Tasks 68 + 69:

- `assignment` and `kmeans` are already rayon-parallel inside the
  shared training surface.
- `draft_leaf_rows` collapsed from 19.2 s to 25 ms via Task 68's
  zero-replica fast path.
- `top_graph` was shelved at packet 007 (sub-5 % gain measured for
  the distance-cache slice; further Vamana work has recall risk).

`ec_spire` still declares
(`src/am/ec_spire/routine.rs:28-29`):

```rust
amroutine.amcanparallel = false;
amroutine.amcanbuildparallel = false;
```

So the **single-process** path still owns:

- `heap_scan` (38 % of 100k wall time) — PG-side
  `table_index_build_scan` callback that materialises each row
  into the SPIRE build state.
- The recursive routing pipeline (drafts, top-graph, publish) at
  the leader. Some sub-phases were parallelised internally by
  Task 68 (via Task 69 calls), but the orchestration itself is
  serial.

This task picks up SPIRE parallel build with the goal of letting
PG split heap scan across workers and parallelising the
remaining serial orchestration phases where determinism allows.

## Non-Goals

- Do not change SPIRE on-disk format. Workers must produce the
  same partition objects, draft layout, and publish artifacts as
  the single-process path.
- Do not change recursion semantics. SPIRE recursion correctness
  is owned by Task 30 phases — this task is timing-only.
- Do not pursue top-graph parallelism. Task 68 packet 007 already
  measured and shelved that lane.
- Do not parallelise the publish / object-store flush in Phase 1.
  The crash-safety contract there is non-trivial and deserves its
  own phase (or own task).
- No AWS / Graviton work — M5 is the local optimization host.
- Do not redesign the SPIRE coordinator. Distributed orchestration
  is owned by Task 30 phase 11/12/13 series.

## Phase 1 — Design Decision (gating)

This task is materially larger than Task 71 (IVF parallel build)
because SPIRE has more orchestration layers and more crash-safety
surface. Land one design packet before code:

- Audit each remaining serial phase and decide which are
  parallelisation-eligible vs not:
  - **heap_scan**: PG parallel-scan callback territory. Mirrors
    HNSW / IVF parallel build pattern.
  - **draft assembly (post-fast-path)**: now 92 ms at 100k, may
    not be worth parallelising. Phase 1 should explicitly
    confirm or reject.
  - **recursive routing**: single iteration at fanout=8/nlists=128
    per Task 68 packet 003. Likely not parallel-eligible at
    measured shapes.
  - **publish / object_store**: explicitly out of scope for
    Phase 1 (crash safety surface).
- Pick the worker model: per-worker `SpireBuildState` with leader
  merge, or DSM-backed staging matching HNSW's ConcurrentDsm.
- Specify the determinism contract: same-seed builds must produce
  structurally-equivalent SPIRE partition objects, drafts, and
  publish artifacts. Byte-equality is preferable; structural
  equality with documented worker-order observable differences is
  the fallback.
- Specify the fallback contract: under failure, parallel build
  must either complete successfully or roll back to a state where
  re-trying the single-process path works.

## Phase 2 — Implementation Slices

Once Phase 1 picks the model:

1. **Routine flag flip + skeleton.** Set
   `amcanbuildparallel = true` on `ec_spire`. Wire
   `amestimateparallelscan`. Skeleton per-worker
   `SpireBuildState`.
2. **Heap-scan parallelism.** Workers walk heap tuples in
   parallel; per-worker thread-local accumulator. Leader merges
   into the existing centroid plan + draft pipeline.
3. **Leader merge + determinism gate.** Leader runs the existing
   Task 68 + Task 69 parallel paths (k-means, batch assignment,
   zero-replica fast path) on the merged sample. Same-seed
   structural-hash check on output artifacts.
4. **Optional: parallel draft assembly.** Only if Phase 1
   identifies a draft sub-phase worth parallelising **and**
   measured 100k wall-time share justifies it (≥ 5 % of total
   post-Task-69-and-Task-72-step-3 build time).

## Phase 3 — Measurement

Final measurement packet repeating Task 68's Phase-1 disjoint
split, plus:

- Build wall time at workers 1/2/4/8 on the same fixture surfaces
  Task 68 measured (10k, 100k).
- Recall@10 at each worker count, holding within 0.5 pp of
  Task 68's closeout numbers (10k: 0.9995; 100k: 0.8525 at
  nprobe=16).
- Structural hash equality across same-seed builds at every
  worker count (hierarchy, root routing, routing centroids, leaf
  summary, leaf assignments — same as Task 68 closeout).
- Worker launch counter via the same instrumentation Tasks 33 +
  71 use.
- Memory HWM (sampler wrapper or `not_measured`).

## Exit Criteria

- Phase 1 design packet landed with reviewer-approved model choice.
- Phase 2 slices landed; `amcanbuildparallel = true` on `ec_spire`.
- Phase 3 measurement shows multi-× build-time win at workers ≥ 2
  on the 100k fixture.
- Determinism preserved: same-seed structural hashes match across
  worker counts.
- Recall floor preserved on Task 68 comparator fixtures.
- No new `unsafe { ... }` blocks outside the PG callback boundary
  the existing SPIRE build code already lives in.
- `cargo clippy --all-targets --no-default-features --features pg18
  -- -D warnings` clean.
- Closeout packet citing Phase 1 + Phase 2 + Phase 3 evidence
  flips `plan/tasks/72-…md` status to `complete`.

## Coordination

- **Tasks 68 + 69 are hard dependencies.** Both closed — proceed.
- **Task 30 phases own SPIRE recursion + distributed semantics.**
  This task must coordinate with whichever Task 30 phase is
  active (last seen: phase 13d/e). Do not start before checking
  for active Task 30 conflicts on `ec_spire/build/**`.
- **Task 71 (IVF parallel build) is the reference for the lighter
  shape.** Read it before Phase 1 design to decide whether SPIRE
  can adopt the same model or needs more orchestration surface.
- **HNSW `build_parallel.rs` and ADR-048 are the deeper reference**
  for the ConcurrentDsm option.
- M5 is the local optimization host. Cloud confirmation belongs
  in a SPIRE-specific Graviton task if/when one is opened.
- Honor memory `feedback_dont_defer_safety_fixes`,
  `feedback_anti_pattern_b_unbounded_lifetime`, and
  `feedback_view_operations_not_accessors` in review.

## Stop Conditions

- Stop if Phase 1 design surfaces irreducible orchestration
  serialisation (e.g. draft coordinator state that cannot be
  partitioned by row, or a publish step that requires single-writer
  semantics across all phases). In that case, document the finding
  and shelve — the heap-scan-only parallelism may still be worth
  landing, or the whole task may shelve.
- Stop if recall regresses beyond 0.5 pp on any comparator fixture
  during Phase 2 validation. Deterministic output is load-bearing
  for SPIRE correctness given its distributed read semantics.
- Stop if measured 100k speedup at workers=4 is below 1.5× — at
  that point parallel build is a wash and the engineering cost
  isn't justified. Mirror Task 33's stop condition.
