# Task 75: SPIRE Latency — Routing Candidate Envelope

Status: complete (2026-05-31, closeout `reviews/task-75/003-closeout/`)
Owner: coder (to be assigned). One coder, one branch.
Priority: 1 (closes the load-bearing SPIRE/IVF latency gap from Tasks 73/74)

## Why

Tasks 73 and 74 jointly established the SPIRE latency problem:

- At matched recall (0.9975-1.0000 recall@10 on real-100k), SPIRE
  p50 is `3.65x-7x` IVF depending on host (Intel 3.65x, AWS
  Graviton 4.5-4.7x, M5 7x).
- Task 74's Intel profiler placed `75 %` of SPIRE CPU inside
  `ProdQuantizer::score_ip_from_split_parts` and only `~4.9 %` in
  identifiable SPIRE-specific orchestration. Task 74 correctly
  shelved scan-orchestration optimization slices on that basis.

The load-bearing observation Task 74 *did not* act on lives in the
Task 73 packet `pipeline-100k-tg128-b0.log`: at
`top_graph_search_list_size=128`, the SPIRE candidate envelope is
**identical at nprobe 64, 96, and 128** —
`candidate_sum=2784952` and `object_bytes_sum=2237295832` for all
three. At matched recall on this fixture, SPIRE scores roughly
`2.78 M` candidates per query against IVF's `~75 k` (96 lists ×
~780 vectors), a `~37x` scored-candidate ratio. That explains the
latency gap: SPIRE is doing more scoring work, not slower scoring
work.

This task answers one question: **can routing deliver matched
recall with a materially smaller candidate envelope?** Either
answer is useful:

- If yes, a routing-tightening slice (fewer leaves, tighter
  recursive draft, smarter top-graph traversal) closes the gap.
- If no, the matched-recall latency is structural to SPIRE's
  routing/storage shape and the dev line should accept it.

## Non-Goals

- Do not change scoring kernels. The scoring path is shared with
  IVF and is out of scope for a SPIRE-specific task.
- Do not reopen Task 74's scan-orchestration slices. Their
  rationale (`~4.9 %` orchestration overhead) still holds.
- Do not change SPIRE on-disk format, partition-object schema, or
  recursion correctness contracts.
- Do not redesign routing as a P0 deliverable. This task is
  measurement-first; a routing slice only lands if Phase 1
  surfaces a clean win without touching recursion semantics.
- Do not pursue defaults policy here. That lives in
  `plan/design/spire-quality-defaults-followup.md`.

## Phase 1 — Candidate Envelope Attribution (gating)

Land one local Intel-desktop measurement packet **before any
slice**. Required contents:

- At the Task 73 high-recall point (`tg128 b0 nprobe=96`) on
  real-100k, instrument per-query:
  - `leaf_route_sum` (leaves selected by routing)
  - `candidate_sum` (raw candidates entering scoring)
  - retained-after-rerank count (those that made the heap)
  - returned-to-k count (those that made the top-k)
  - per-leaf candidate distribution (mean / p95 / max)
- Sweep `top_graph_search_list_size` across `16, 32, 64, 96, 128`
  at the highest nprobe each setting permits, holding
  `boundary_replica_count=0`, and record:
  - recall@10 at each setting
  - candidate_sum at each setting
  - retained/returned counts at each setting
- Compute the **candidate→retained→returned funnel** at each
  setting. The diagnostic question is: at the matched-recall
  point, how many of the `2.78 M` candidates ever make the heap?
- Compare to IVF control at matched recall: at nprobe 96, what is
  IVF's candidate_sum and retained-after-rerank count? The
  IVF/SPIRE candidate ratio is the *upper bound* on what routing
  could close.

Phase 1 closes when the packet has:

- a clean funnel table at the matched-recall point, and
- a ranked Phase 2 P0 slice list, or "the funnel is tight, the
  envelope is structural, accept and close".

## Phase 2 — Slices (only if Phase 1 surfaces a routing win)

P0 slices land one at a time. Candidates depend on Phase 1, but
reasonable hypotheses:

1. **Tighter recursive draft at high `top_graph_search_list_size`**
   — if Phase 1 shows the draft picks leaves whose candidates
   never reach the heap, prune them earlier.
2. **Score-bound early termination** — if the heap's worst score
   bounds further candidate work, push the bound into the
   per-leaf scoring loop.
3. **Adaptive `nprobe` collapse** — if Phase 1 shows nprobe
   doesn't change the envelope at high `top_graph_search_list_size`,
   treat the extra nprobe budget as routing-unused and document or
   reject it as a knob in that regime.

Any slice must:

- preserve Task 73's recall@10 floor at the tested fixture
  (within 0.5 pp),
- preserve recursion-correctness semantics (no Task 30 phase
  question reopened),
- be measured against the Phase 1 funnel to show ≥ 10 % p50
  latency win at the matched-recall point (else skip per the
  Task 74 small-slice cap),
- include the same Intel-desktop perf/flamegraph evidence Task 74
  established as the gate for SPIRE scan-side claims.

## Exit Criteria

- Phase 1 packet landed with reviewer-approved funnel and
  ranking.
- All Phase 2 P0 slices either landed with measured p50 win at
  matched recall, or shelved with a recorded reason.
- Recall floor preserved per Task 73 measured points (10k and
  100k).
- No new `unsafe { ... }` blocks. SPIRE scan code lives inside
  `pg_am_callback!`-bounded surfaces; this task should stay safe.
- `cargo clippy --all-targets --no-default-features --features
  pg18 -- -D warnings` clean.
- Closeout packet flips `plan/tasks/75-…md` status to `complete`.

## Coordination

- **Task 73** is closed; this task picks up the candidate-envelope
  thread Task 73 measured but did not address.
- **Task 74** is closed; this task does not touch the
  scan-orchestration code Task 74 cleared.
- **Task 30 phase tree** is closed (Phase 13e accepted). If
  Phase 1 surfaces a question that requires recursion-semantics
  changes, file as a Task 30 successor rather than slicing here.
- **Task 72 (SPIRE parallel build)** is independent — build-time
  vs query-time.
- **Defaults follow-on** (`plan/design/spire-quality-defaults-
  followup.md`) is unblocked by this task's evidence but is a
  separate decision track.

## Stop Conditions

- Stop Phase 2 if Phase 1 funnel shows ≥ 50 % of candidates reach
  the heap at the matched-recall point. That would mean the
  routing draft is already tight and the envelope is structural
  to SPIRE's recall path; the remaining gap belongs to a
  scoring-kernel or storage-format track, not SPIRE.
- Stop Phase 2 if a candidate slice would change recursion
  semantics. File the question elsewhere and pause.
- Stop if a slice improves the matched-recall point but regresses
  the current default (cross-setting tradeoff). Document and
  defer rather than ship.
