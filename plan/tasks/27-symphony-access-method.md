# Task 27: SymphonyQG Access Method — ADR-045 Stages 2 and 3

Status: proposed — **unblocked by task 25**. Start now.

The original 1pp-of-exact absolute-encoding gate has been superseded;
see `plan/tasks/25-rabitq-quantizer.md` under "Decision gate" for the
reasoning. In short: Symphony does not use absolute-encoded RaBitQ, so
the absolute-path recall verdict is not the right blocker. The actual
Stage-2 prerequisite from the quantizer side was the centered-encode +
score API, which landed in task 25 slice 15
(`review/20014-task25-centered-api/`). Symphony's own recall gate lives
at the end of Phase 2 of this task — an end-to-end test, not a
quantizer-module test.

Executes **ADR-045 Stages 2 and 3**. Sibling to task 25 (which executes Stage 1).

## Scope

Introduce `symphony` as a third access-method variant alongside `ec_hnsw`
(ADR-032) and `ec_diskann` (ADR-034), housed under `src/am/symphony/` per
ADR-041. Reuse the `ec_hnsw` page, build, and insert skeletons; swap in two
structural changes that co-design the graph layout with the RaBitQ scoring
path.

**Stage 2** — quantized-graph build, rerank still on:

1. **Out-degree padding.** Each node's neighbor list is padded to a multiple
   of the FastScan SIMD batch size by selecting *additional real edges*
   (not dummies). Storage grows modestly; traversal issues only full-width
   kernels with no tail path.
2. **Quantization-aware edge selection.** The RNG / α-pruning rule
   evaluates candidate edges using the RaBitQ distance, not fp32. The
   built graph is self-consistent with the scoring path.

Stage 2's query path still reranks with exact fp32 as a safety net. This
isolates graph-layout risk from quantizer-accuracy risk.

**Stage 3** — no-rerank query path:

Flip off rerank. Top-k returned directly from RaBitQ estimates, using the
error bound (from task 25) to size the candidate pool conservatively. Gated
by recall@10 holding at the Stage 1 baseline on full benches.

## Why now (after task 25)

- SymphonyQG is the largest latency-per-recall win on the roadmap:
  2–4× QPS over `ec_hnsw` at equal recall per the paper, and a simpler
  query pipeline (the graph *is* the filter — no three-stage
  RaBitQ→FastScan→exact pipeline to maintain).
- Storage is a side win: Stage-2 indexes are *smaller* than `ec_hnsw`
  despite padding, because the RaBitQ code shrinks more than the
  adjacency grows.
- The coexisting-formats posture (ADR-032, ADR-033) already absorbs a
  third AM variant cleanly. No migration story to design from scratch.

## Design outline

See ADR-045 §"Stage 2" and §"Stage 3." Summary:

- **Module layout.** `src/am/symphony/` per ADR-041; mirrors the
  `src/am/ec_hnsw/` file split (build, insert, scan, page, graph,
  vacuum).
- **Reuse.** Page layout, WAL, graph traversal skeleton, vacuum
  primitives — all copied/adapted from `ec_hnsw`. Divergence is
  isolated to (a) adjacency list padding and (b) the distance
  function in build-time edge selection and runtime scoring.
- **RaBitQ consumption.** Depends on the frozen RaBitQ scorer API
  handed off from task 25 Phase 3. This AM does not re-implement
  the quantizer; it consumes it.
- **Out-degree padding.** At build time, after the standard
  neighbor-selection step, pad each list to the next multiple of the
  FastScan SIMD batch width by taking the next-best *real* candidates
  from the evaluated pool. Track the padding factor in index metadata
  so vacuum can maintain it.
- **Quantization-aware pruning.** The α-pruning / RNG rule in the
  graph builder evaluates candidates with the RaBitQ scorer. This is
  the "Stage 2" structural change that makes the graph self-consistent
  with the scan path.
- **Wire format.** New `INDEX_FORMAT_V5_SYMPHONY` (or the ADR-032
  reloption equivalent). Not auto-migratable from `ec_hnsw`; REINDEX
  only.
- **Stage-2 query path.** RaBitQ scorer in the graph beam search,
  fp32 rerank on the top-k candidate pool. Reuses `ec_hnsw`'s rerank
  seam without modification.
- **Stage-3 query path.** Drop rerank. Use the RaBitQ error bound
  (from task 25 Phase 2 calibration) to size the candidate pool
  `k' > k` such that the top-k from the estimator contains the true
  top-k with the target confidence. Gate behind a reloption until
  recall@10 holds on the full bench matrix.

## Subtasks

### Phase 0 — handoff and design freeze

- [ ] **Consume RaBitQ API.** Verify the frozen scorer surface from
  task 25 Phase 3 covers everything this task needs: rotation,
  encode, scalar norm, scorer, error bound. Any gap goes back to
  task 25 as a scope addition, not patched in-tree here.
- [ ] **Page-layout delta.** Document what diverges from `ec_hnsw`'s
  page layout to support padded adjacency. Ideally: same tuple
  format, different neighbor-count semantics.
- [ ] **Correctness invariant.** Define the reference oracle: a
  `symphony` index built at `padding = 1` (no padding) must produce
  identical top-k to an `ec_hnsw` index built with a RaBitQ-scored
  pruning rule but fp32 scoring at query time — isolates the two
  structural changes for unit-level validation.

### Phase 1 — `src/am/symphony/` skeleton (Stage 2)

- [ ] **Module scaffold.** `src/am/symphony/{build,insert,scan,page,graph,vacuum}.rs`
  following ADR-041. Registered as a third AM in pgrx alongside
  `ec_hnsw` and `ec_diskann`.
- [ ] **Wire format.** `INDEX_FORMAT_V5_SYMPHONY` (or reloption
  equivalent). Metadata page carries the padding factor and the
  RaBitQ parameters.
- [ ] **Page layout.** Padded-adjacency tuples. Reuse `ec_hnsw`'s
  WAL helpers wherever possible.
- [ ] **Scan path (Stage 2).** Beam search with RaBitQ scoring,
  fp32 rerank on the top-k tail. Reuses `ec_hnsw`'s rerank.
- [ ] **Build path (Stage 2).**
  - [ ] Heap scan + RaBitQ encode (uses task 25's encoder).
  - [ ] Graph construction with quantization-aware α-pruning.
  - [ ] Out-degree padding pass.
  - [ ] Flush.
- [ ] **Insert path (Stage 2).** `aminsert` mirroring the build
  structure: encode → quantization-aware neighbor selection →
  padding → write.
- [ ] **Vacuum path.** Maintain padding invariant on delete. Likely
  reuses `ec_diskann`'s vacuum primitives (task 17) — shared graph
  lifecycle under ADR-033.

### Phase 2 — Stage 2 validation

- [ ] **Recall parity.** On the 50k and 1M real seams, `symphony`
  Stage 2 recall@10 at least matches `ec_hnsw` + RaBitQ-prefilter
  at equal `ef_search`. Expect it to exceed, per the paper.
- [ ] **Latency win.** Measure end-to-end p50/p95 at equal recall.
  Target: ≥1.5× QPS over `ec_hnsw` (Stage 2 still reranks, so the
  headline 2–4× is not realized until Stage 3).
- [ ] **Build-time cost.** Expect 1.3–2× slower build than
  `ec_hnsw`, offset by a reported 8× build speedup vs NGT-QG —
  measure and record, do not hide the regression if it exists.
- [ ] **Storage.** Confirm Stage-2 indexes come in smaller than
  `ec_hnsw` despite padding.
- [ ] **Review packet.** Publish Stage 2 numbers before Stage 3.

### Phase 3 — Stage 3: no-rerank query path

- [ ] **Error-bound sizing.** Consume task 25's calibrated error
  bound to size the candidate pool `k'`. `k' = k` when the tail of
  the error distribution is tight; widen when the tail is heavy.
- [ ] **Reloption gate.** `symphony.no_rerank = off | on | auto`.
  Default `off` at first ship; flip to `auto` (model-driven) once
  recall@10 holds on the full bench matrix.
- [ ] **Scan path (Stage 3).** Skip the rerank stage. Return top-k
  directly from the RaBitQ estimator output.
- [ ] **Recall gate.** Stage 3 recall@10 within 1pp of Stage 2 on
  the 50k, 1M, and full bench seams. If any seam fails, leave the
  reloption at `off` by default and document.
- [ ] **Review packet.** Stage 3 headline: 2–4× QPS over `ec_hnsw`
  at equal recall, with the rerank pipeline collapsed.

### Phase 4 — planner integration

- [ ] **Cost model.** Update the ADR-011 / task-11 cost model so
  the planner picks `symphony` when appropriate. Stage 3's
  no-rerank path changes the cost shape — shorter scan, no rerank
  tail — so the existing HNSW cost formula is wrong for symphony.
- [ ] **EXPLAIN surface.** Show the scoring mode (rerank on/off),
  the padding factor, and the RaBitQ error-bound pool size.

## Owns

- ADR-045 Stages 2 and 3.
- `src/am/symphony/` (new module per ADR-041).
- Wire format `INDEX_FORMAT_V5_SYMPHONY` (or reloption equivalent
  under ADR-032).
- Planner cost model for the `symphony` AM (coordinates with task 11).

## Dependencies

- **Hard blocker (met):** task 25 shipped the centered-encode + score
  API that Symphony §3.1.1 / eq. (5)–(6) require
  (`review/20014-task25-centered-api/`). The absolute-encoding 1pp
  recall gate has been retired for the reasons recorded in task 25
  ("Decision gate" section); it was never the right gate for Symphony.
- **Hard blocker (met):** task 25 Phase 3 API freeze
  (`review/20005-task25-task27-handoff-contract/`, superseded by the
  amended contract recorded in task 25 slice 16 after reviewer
  feedback on the centered-API packet). Covers rotation seam,
  absolute encoder + scorer, error-bound estimator, centered encode
  + score, and seeded SRHT.
- **Soft dependency:** task 26 (parallel index build). If it lands
  first, symphony builds inherit parallelism at no cost; if not,
  the symphony build ships single-threaded and gets parallelism on
  the task-26 rebase.
- **Soft dependency:** task 20 (OPQ). OPQ's learned rotation drops
  into the RaBitQ front-end without changing this task's design;
  compose later.
- **Read-only:** `ec_hnsw` and `ec_diskann` modules as skeletons.
  Do not edit them from this task.
- **Inherits prior ADR-031 work via task 25.** The persisted-sidecar,
  cached-runtime, and grouped-binary-traversal work already on `main`
  (commits `d662a72` / `e1b0912` / `552f4d1`; packets 279, 281, 285, 286,
  359, 360) is absorbed into task 25's graduated quantizer module. This
  task consumes task 25 through the frozen trait API; it does not reach
  past into the legacy in-scan prefilter.

## Unblocks

- The headline end-to-end Symphony win: 2–4× QPS over `ec_hnsw` at
  equal recall with a simpler query pipeline.
- Supersedes ADR-031's prefilter pipeline: the Symphony query path
  *is* the filter, no three-stage composition.
- Template for future quantized-graph variants — e.g., DiskANN's
  in-memory tier adopting Symphony-style pruning against RaBitQ
  becomes a plausible follow-up.

## Out of scope

- RaBitQ quantizer itself — task 25.
- Learned rotation (OPQ) — task 20; composes in later.
- GPU-accelerated offline build — ADR-046; orthogonal.
- Migrating `ec_hnsw` indexes to `symphony` — REINDEX only, per
  ADR-032 and ADR-045.
- Parallel build — task 26; this task ships serial and rebases onto
  parallel when task 26 lands.

## Notes

- **Stage 2 and Stage 3 are separate ship-gates.** Do not try to
  land them in one packet. Stage 2 isolates graph-layout risk;
  Stage 3 isolates quantizer-accuracy risk. A combined landing
  makes a regression impossible to attribute.
- **Stage 3's reloption default is `off` on first ship.** Flip to
  `auto` only after the full bench matrix holds. A no-rerank path
  that fails recall on one corpus is a correctness bug, not a
  tuning miss.
- **Do not reach back into `ec_hnsw` to share code.** If a
  primitive wants to be shared (padding, RaBitQ-aware pruning), it
  migrates to `src/am/common/` under ADR-041's seam discipline.
- **Build-time regression is acceptable in Stage 2.** Per the
  paper and ADR-045, the structural cost is baked into the
  approach. Measure and disclose; do not optimize away a design
  property.
