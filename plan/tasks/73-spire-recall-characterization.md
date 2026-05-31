# Task 73: SPIRE Recall Characterization + Routing-Quality Slices

Status: complete (2026-05-31, closeout `reviews/task-73/002-closeout/`, reviewer acceptance `reviews/task-73/003-completion-audit/feedback/2026-05-31-01-reviewer.md`, follow-up acknowledgements `reviews/task-73/004-reviewer-followup/`)
Owner: coder (to be assigned). One coder, one branch.
Priority: 1 (publishable competitive concern; recall gap vs other AMs is the load-bearing finding)

## Why

Task 68 closeout packet 008 measured SPIRE recall@10 on real-world
fixtures:

| fixture | nprobe | recall@10 | ndcg@10 |
| --- | ---: | ---: | ---: |
| 10k | 16 | **0.9995** | 1.0000 |
| 100k | 16 | **0.8525** | 0.9835 |

The 100k recall floor of 0.8525 is **substantially below** the
recall numbers other ECAZ AMs hit on comparable corpora:

- `ec_ivf` (Task 31 packet 038, real 100K, nprobe=96): **0.9920**
  recall@100 at quality point, `0.9676` at balanced point.
- `ec_diskann` (Task 32, real10K @ list=64): **0.9965** recall@10.
  100K not directly published but consistently > 0.99.
- `ec_hnsw` (Task 33 packet 003, real100K @ ef=400): **0.9775**
  recall@10.

SPIRE drops nearly 15 percentage points from 10k to 100k at the
same nprobe. That points at one (or both) of:

- **Routing aggression**: the recursive routing draft at
  `nprobe=16` / `top_graph_search_list_size=16` visits too few
  leaves on the 100k corpus, missing candidate sets that would
  recover recall.
- **Leaf-scan completeness**: even when the right leaves are
  routed, the per-leaf scan / scoring / candidate-budget logic
  drops candidates that would have ranked.

Task 30 Phase 10 closed the **execution architecture** (bounded
candidate collection, AM scan shape, heap rerank, top-graph
overhead). Phase 13d added **observability**. Neither targeted
**recall as a metric**. This task does.

## Non-Goals

- Do not change SPIRE on-disk format or partition object schema.
- Do not change recursion semantics. SPIRE recursion correctness
  is owned by Task 30 phases — this task is tuning-only and
  measurement-only at first, with optional small slices if the
  measurement reveals tunable defaults.
- Do not pursue routing redesign. If the characterisation shows
  fundamental routing limits, the follow-on is a separate Task 30
  phase, not slices under this task.
- No first-pass AWS / Graviton work — M5 is the mandatory local
  characterisation host. AWS is only a confirmation gate after the M5
  local sweep proves that SPIRE can reach a credible high-recall point.
- Do not pursue cross-AM recall comparison as a publishable claim
  in this task; the cross-AM recall gap belongs in
  `docs/benchmarks.md` after the M5 characterisation is solid.

## Phase 1 — Local Recall/Latency Characterization (gating)

Land one M5-local measurement packet **before any slice**. This packet
may be shared with Task 74's Phase 1 overhead audit. Required contents:

- Recall@10 + latency Pareto sweep on the same fixtures Task 68 used (10k + 100k
  M5 DBpedia), varying:
  - `nprobe` across `8, 16, 32, 64, 96, 128, 256`
  - `top_graph_search_list_size` across `16, 32, 64, 128`
  - `boundary_replica_count` across `0, 1, 2`
  - `recursive_fanout` if Phase 1 evidence suggests it's a knob
- For each `(nprobe, top_graph_search_list_size,
  boundary_replica_count)` combination, capture:
  - recall@10
  - mean query time + p50/p95/p99
  - candidate rows visited, deduped, retained, truncated (the
    Phase 10 diagnostics)
  - top-graph frontier size hit
- Identify the **recall ceiling** at the most permissive local setting
  (e.g. `nprobe=256, top_graph_list=128, boundary=2`) — this
  bounds what routing can recover vs what leaf scan can't.
- Decide which axis closes the most of the 0.85 → 0.99+ gap.
- Explicitly classify AWS readiness:
  - if the local permissive ceiling reaches 0.99+ recall@10 with a
    credible latency point, AWS confirmation may proceed on only the
    selected default/high-recall/control points;
  - if the local permissive ceiling is below the quality bar, do not
    run AWS; hand off to routing/partition/codec diagnosis first.
- Treat cross-AM comparisons as directional unless they use the same
  host, fixture, query count, `k`, and metric. The current IVF citation
  uses recall@100, while the SPIRE/HNSW figures use recall@10; do not
  present those as apples-to-apples.

Phase 1 closes when the measurement packet has reviewer-approved
findings and a ranked Phase 2 slice list (or an honest "no
tunable wins; recall floor is fundamental, escalate to Task 30").

## Phase 2 — Slices (only if Phase 1 surfaces tunable wins)

P0 slices land one at a time. Candidates:

1. **Default `top_graph_search_list_size` change** — if the
   characterisation shows the current default is too aggressive
   for 100k+ corpora, propose a new default backed by Phase 1
   evidence.
2. **Default `nprobe` change** — same shape, different knob.
3. **Boundary-replica-aware routing** — if `boundary_replica_count
   > 0` consistently closes recall without proportional latency
   cost, propose making it the default.
4. **Adaptive routing aggression** — if Phase 1 shows a clean
   `(corpus_size, recall_target) → (nprobe, top_graph_list)`
   curve, propose a row-count-aware default selection.
5. **Diagnostic surface for recall debugging** — if candidate
   accounting (visited, deduped, retained, truncated) doesn't
   show enough to localise drops, expand the diagnostic surface
   under the Phase 13d pattern.

Any slice that changes a default (1, 2, 3, 4) must:
- be backed by Phase 1 evidence,
- be tested on **both** 10k and 100k fixtures (no recall
  regression on 10k while improving 100k), and
- preserve the cited recall floor for the default workload.

## Exit Criteria

- Phase 1 characterisation packet landed with reviewer-approved
  ranking and decision (tunable vs fundamental).
- If tunable: all Phase 2 P0 slices either landed with measured
  recall improvement on the comparator fixtures, or shelved with
  a recorded reason.
- If fundamental: explicit hand-off packet citing Task 30 phase
  responsibility, recommending a routing-redesign track.
- Recall measurement re-runs the comparator at Task 68's
  documented settings to prove the closeout number can be
  reproduced before tuning.
- No `unsafe { ... }` blocks introduced. SPIRE scan code lives
  inside `pg_am_callback!`-bounded surfaces; this task should
  stay scoped to safe Rust tuning.
- `cargo clippy --all-targets --no-default-features --features
  pg18 -- -D warnings` clean.
- Closeout packet citing Phase 1 + (optional) Phase 2 evidence
  flips `plan/tasks/73-…md` status to `complete`.

## Coordination

- **Task 30 phases own SPIRE recursion correctness.** Before
  proposing routing-quality slices, confirm with whichever Task
  30 phase is currently active (last seen: Phase 13d/e) that the
  proposed tuning is in scope. If a slice would change recursion
  semantics, file it as a Task 30 phase instead.
- **Task 68 is closed** — this task picks up the recall gap that
  Task 68 measured but did not address (Task 68's scope was build
  perf, not query recall).
- **Task 74 (SPIRE leaf scan overhead)** is parallel and
  independent — Task 73 targets the routing axis, Task 74 targets
  the leaf-scan axis. They could share a Phase 1 measurement
  packet if convenient.
- Local M5 is the characterisation host. The Gen 10 Intel desktop is an
  optional x86 comparator after the M5 decision, not an exit criterion.
  Cloud confirmation belongs in a future SPIRE-on-Graviton task only if
  the M5 local quality gate passes and the remaining question is
  distributed/remote viability.

## Stop Conditions

- Stop Phase 2 if Phase 1 shows the recall ceiling at the most
  permissive settings is still well below the cross-AM bar (e.g.
  if `nprobe=256, top_graph_list=128, boundary=2` only gets to
  0.92 recall on 100k). At that point the floor is fundamental
  routing or codec quality, and the work belongs in Task 30 or
  a codec-specific task.
- Stop if a proposed default change improves the cited fixture
  but regresses another (cross-corpus tradeoff). Document and
  defer to an adaptive-defaults Phase 2 slice rather than ship
  the regression.
