# Task 68: SPIRE Build Performance — Characterization + Targeted Slices

Status: complete (2026-05-30). Closeout packet:
`reviews/task-68/008-closeout/`.
Owner: coder (to be assigned). One coder, one branch.
Priority: 1 (highest leverage for SPIRE build wall time)

## Why

SPIRE build wall time has never been formally split by component. The
build pipeline lives across `src/am/ec_spire/build/{drafts,training,
recursive,publish,top_graph,routing_plan,object_store,tuples,types}.rs`
plus shared training under `crate::am::common::training`. Without a
characterization packet we cannot rank fixes; the DiskANN Task 29c /
Task 65 precedent (`plan/design/diskann-build-performance.md`) showed
that profiling-first is the cheapest way to avoid speculative slices.

Suspected dominant costs, **unranked**, to be validated by Phase 1
profiling:

- **Shared k-means** (`common_training::train_spherical_kmeans` at
  `src/am/common/training.rs:71`), called per recursion level from
  `ec_spire/build/recursive.rs:30` and from the single-level pipeline
  at `ec_spire/build/training.rs:12,142`. Recursion compounds the cost.
- **Grouped PQ4 codebook training**
  (`common_training::train_grouped_pq4_model` at
  `src/am/common/training.rs:278`), called from SPIRE materialization
  (`ec_spire/update/materialization.rs:392`).
- **Draft / publish / top-graph assembly** under
  `ec_spire/build/{drafts,publish,top_graph}.rs`.
- **Object-store I/O** under `ec_spire/build/object_store.rs`.
- **Per-vector assignment loops**
  (`common_training::assign_vector_to_centroid` at
  `src/am/common/training.rs:144`) called from training, recursive,
  and routing paths.

The shared-training pieces are split out into Task 69 (Common Training
Parallelism). This task owns SPIRE-specific build pipeline cost and
sequences the shared-training lift through Task 69 once Phase 1
identifies it as a dominant cost.

## Non-Goals

- Do not edit `src/am/common/training.rs` here; that surface is owned
  by Task 69. This task may **consume** Task 69's parallel APIs once
  they exist, but does not redesign them.
- Do not change SPIRE on-disk format or object-store wire format as
  part of a perf slice without an ADR. P0 slices are constant-factor
  wins inside the existing format.
- No AWS or Graviton work in this task. M5 is the local optimization
  host (mirror Tasks 31/65 baseline rules). Cloud-class confirmation
  is a separate follow-on.

## Phase 1 — Characterization (gating)

Land one measurement packet **before any code change**. Required
contents:

- A SPIRE build wall-time split at two corpus sizes (10k and 100k on
  M5, fixture-loader release build, one-index-per-table), broken into
  at minimum:
  - heap scan + sample collect,
  - shared k-means (sum across all recursion levels),
  - shared PQ4 codebook training,
  - draft assembly,
  - top-graph construction,
  - publish / object-store flush.
- Per-level k-means count (how many calls, mean and total time per
  level) — confirm or refute the "recursion compounds k-means"
  hypothesis.
- A static call audit listing every site that calls
  `common_training::*` from `ec_spire/build/` so Phase 2 has a clean
  consumer surface to point at.
- Recommend Phase 2 P0 slices in priority order, each with a measured
  wall-time share and an estimated cap (best-case speedup).

Profiling tools: `dhat`, `samply`, or `cargo flamegraph` are all
acceptable. Whatever is used, store the raw output under
`reviews/task-68/001-characterization/artifacts/` with a manifest per
CLAUDE.md `manifest.md` rules.

Phase 1 closes when the measurement packet has reviewer-approved
findings and a ranked P0 list.

## Phase 2 — P0 Slices

P0 slices are landed one at a time, each with:

- A code packet under `reviews/task-68/{NNN}-{slug}/` with the
  source diff and a Phase-1 backreference for the cost share it
  targets.
- A measurement packet under the same task bucket repeating the
  Phase-1 wall-time split, with the slice applied, on the same two
  fixture sizes.
- A per-slice cap: skip the slice if the projected speedup is below
  ~5 % of total build wall time at 100k, unless it's a prerequisite
  for another slice.

Candidate slices (only those Phase 1 ranks P0 are landed):

1. **Cooperative consumption of Task 69's parallel k-means** —
   replace the `train_spherical_kmeans` call sites under
   `ec_spire/build/{training,recursive}.rs` with the parallel entry
   point once Task 69 lands it. Determinism must be preserved
   (same seed → bit-identical centroids vs scalar baseline).
2. **Parallel `assign_vector_to_centroid` fan-out across recursion
   levels.** SPIRE recursion assigns each vector once per level; if
   profiling shows assignment dominates a level, rayon over the
   sample slice with thread-local scratch.
3. **Draft assembly hot-path reductions** — only after Phase 1 shows
   draft cost is non-trivial. Scope strictly limited to
   `ec_spire/build/drafts.rs` and `ec_spire/build/tuples.rs`.
4. **Top-graph construction** — only if Phase 1 shows it is a P0.
   Scope strictly limited to `ec_spire/build/top_graph.rs` and
   `ec_spire/build/routing_plan.rs`.
5. **Object-store flush batching** — only if Phase 1 shows
   per-flush overhead is a P0. Scope strictly limited to
   `ec_spire/build/object_store.rs` and `ec_spire/build/publish.rs`.

Any slice outside the above list requires Phase-1 evidence and a
short addendum to this task file.

## Exit Criteria

- Phase 1 characterization packet landed with reviewer-approved
  ranking.
- All Phase 1 P0 slices either landed with a measured win on the
  same fixture, or explicitly shelved with a recorded reason.
- Final measurement packet repeating the Phase-1 split, showing the
  build-time delta vs baseline at both 10k and 100k.
- Recall floor preserved: SPIRE recall@10 within 0.5 pp of baseline
  on whatever fixtures the Phase-1 packet pins as the comparators.
- Build determinism preserved: identical seed → identical SPIRE
  posting / draft / publish artifacts (byte-equal where the format
  is byte-equal; structurally-equal where rayon ordering is
  observable but semantically equivalent — call this out in the
  slice packet).
- No new `unsafe { ... }` blocks introduced. Reuse existing safety
  patterns. Honor memory `feedback_dont_defer_safety_fixes` and
  `feedback_anti_pattern_b_unbounded_lifetime`.

## Coordination

- **Task 69 is a hard dependency** for slice 1. Phase 1 may land
  before Task 69 closes, but slice 1 only lands after Task 69's
  parallel k-means is on `main`.
- This task does **not** open work on `src/am/ec_ivf/` even where
  the same shared-training improvements are visible there. IVF
  consumption is tracked under whatever IVF lane the team picks
  next (currently no dedicated IVF build task; see Task 57 deferred
  unsafe burndown).
- SPIRE recursion correctness is owned by Task 30 phases. This
  task must not change recursion semantics — only timing.
