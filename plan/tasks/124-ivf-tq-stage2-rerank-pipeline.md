# Task 124: IVF TurboQuant Stage-2 Rerank Pipeline

Status: **in progress — re-baselined measurement phase** (2026-07-04).
Owner: Codex (measurement re-baseline; branch `task-124-stage2-pareto`,
packet `reviews/task-124/001-stage2-vs-rb1-pareto/`). Original in-engine
implementation ran on the historical `task-124-ivf-tq-stage2` branch; its
validated keep-set landed on `main` via Task 130
(`130-tq-post-task124-cleanup.md`).
Priority: P1 follow-up for the Task 122 TurboQuant keep-experimental outcome.

## 2026-07-04 Re-baseline (supersedes Phase 5 comparators below)

What already landed on `main` (Task 130 keep-set; verify with
`git log -S stage2_final_rerank_width`):

- the in-engine 3-stage pipeline: `storage_format='coarse_rerank'`
  (RaBitQ-1 dense coarse) + `rerank_placement='index'` +
  `rerank_format='turboquant'` (persisted TQ stage-2 payload) +
  `stage2_final_rerank_width=N` (bounded final exact f32 source rerank)
  — `9af6ba83e`;
- stage-2 attribution counters (`tq_stage2_*`) — `fef5e20f6`;
- the recall-broken experimental formats (TQ2/binary/768) were pruned
  per Task 130; 4-bit TQ is the only stage-2 payload format.

Why re-baseline: the comparators in Phase 5 predate two facts.
(a) Task 143 flipped the TQ defaults (dense + int8 scorer) and Task 145
cut topk_collect — the champion moved. (b) Task 147 then showed
**rb1 + heap_f32 rerank width 50 pareto-dominates the promoted TQ
default at every scale** (1m: 6.21 vs 6.66 ms, recall 0.9667 vs 0.9208
@ n32, index −68%). So the product question for the TQ stage-2 pipeline
is no longer "does it beat the RaBitQ + f32 baseline of June" but:

> Does inserting the persisted TQ stage-2 reducer between the rb1
> coarse frontier and the exact rerank beat **rb1 + heap_f32 directly**
> — i.e. does cutting exact heap fetches from 50 to 25 pay for the TQ
> payload scoring, at equal recall?

Re-baselined measurement matrix (packet 001; measurement-only, all on
the landed binary, `ecaz bench suite`, 10k/50k/100k then 1m for the
winner):

- **D (stage2@25)**: `coarse_rerank` + `rerank_placement=index` +
  `rerank_format=turboquant` + `rerank_width=50` +
  `stage2_final_rerank_width=25`.
- **E (TQ apples-to-apples)**: `storage_format=turboquant` +
  `rerank=heap_f32` + `rerank_width=50` — isolates coarse-payload
  density (4-bit vs 1-bit) under the SAME rerank, the control Task 147
  deliberately skipped.
- **F (rb1@w25 control)**: rb1 + `rerank=heap_f32` + `rerank_width=25`
  — if plain rb1 holds recall at width 25, the stage-2 reducer has no
  warm-cache job; its value collapses to the IO-sensitive regime
  (Phase 6).
- Baselines cited, not re-run (same binary lineage): rb1@w50
  (`reviews/task-147/001-density-pareto/`), TQ pure default
  (`reviews/task-146/001-outside-scan-profile/`).

Decision rule: D must beat rb1@w50 AND F at matched recall on warm
latency, or show a credible fetch-count/IO rationale plus a Phase 6
cold-cache win, to stay alive as a product path. Phase 6 (IO-sensitive
validation) remains REQUIRED before any promotion claim regardless —
that is also where the TQ no-rerank default retains its zero-heap-fetch
niche and where the Task 147 verdict could flip.

## Why

TurboQuant is still not product-competitive against the strongest RaBitQ + f32
baseline. The recurring failures are recall and latency, with latency the
larger blocker: direct TQ scoring and rerank shapes have not produced a durable
product win.

Task 122 proved one useful but limited point: TurboQuant should not be promoted
as a standalone final scorer, but a sidecar matrix identified a narrower
product-relevant path worth implementing inside the engine. Keep the existing
strong RaBitQ IVF candidate frontier, use TurboQuant as a compact stage-2
reducer, then run exact f32 rerank only on a bounded survivor set.

The measured sidecar path in `reviews/task-122/009-sidecar-tq-stage2-suite/`
matched f32 recall at the useful points when final exact rerank width was 25,
while touching roughly half the compact stage-2 bytes of RaBitQ8. Width 10 was
too narrow at 50k/100k, so this follow-up starts from the width-25 contract
rather than reopening unconstrained rerank-width exploration.

## Goal

Implement and benchmark an in-engine `ec_ivf` pipeline:

- RaBitQ candidate frontier generation;
- TurboQuant index-side or persisted stage-2 scoring over the frontier;
- exact/source f32 final rerank over a bounded survivor set, initially width 25.

The winning claim is matched recall with lower latency and a clear
fetch/materialization/storage rationale against the current RaBitQ + f32
product baseline. A sidecar-only win is not enough.

This is an optimization task, not a measurement-only task. If the first
in-engine attempt does not improve latency at matched recall, keep drilling into
TurboQuant-specific bottlenecks before closing:

- scalar or tail-heavy TQ scorer surfaces;
- candidate batching width and flush histograms;
- score/top-k/materialization fusion around the TQ stage;
- final f32 rerank width and source-vector fetch count;
- compact payload layout and bytes touched;
- query-prep or per-candidate overhead that hides the nominal compact-code win.

## Focus Guardrails

This task is about making TurboQuant competitive. It is not a general SPIRE,
RaBitQ, materialization, or benchmark-cleanup bucket.

Allowed work:

- TurboQuant scorer dispatch, SIMD/block use, scalar-tail reduction, and
  scorer attribution;
- TurboQuant stage-2 payload placement and scoring inside IVF;
- TurboQuant score/top-k/final-rerank fusion;
- TurboQuant recall diagnostics and rerank-width reduction;
- storage or IO measurements only when they isolate TurboQuant payload behavior.

Out of scope unless it directly blocks the TQ path:

- SPIRE-only optimizations;
- RaBitQ-only optimizations;
- generic materialization pruning not tied to TQ stage-2;
- measurement-only packets that do not lead to a TQ implementation or a
  specific TQ bottleneck diagnosis.

## Scope

### Phase 0 - TurboQuant SIMD and Scalar-Surface Audit

Before implementing the stage-2 pipeline, produce a packet-local audit of every
TurboQuant score surface used or touched by this task:

- no-QJL 4-bit LUT;
- QJL;
- tiled-LUT and int8 exact-mode variants if used;
- IVF single-payload fallbacks;
- IVF exact-dequant rerank;
- HNSW/DiskANN/SPIRE only as reference surfaces, not as the main optimization
  target.

For each surface, record whether it is full block/SIMD, block/SIMD with scalar
tail, or scalar/per-candidate. The implementation plan must prioritize the
surfaces that actually sit on the IVF Task 124 hot path.

### Phase 1 - Engine Path and API Shape

Add a narrow IVF rerank pipeline surface that can express:

- candidate-generation representation;
- stage-2 representation;
- final exact/source rerank width;
- stage-2 survivor width;
- source/index placement of the stage-2 payload.

The first supported path should be RaBitQ candidate generation with
TurboQuant stage-2 scoring and exact/source f32 final rerank. Avoid a broad
cross-AM abstraction until the IVF evidence proves the shape.

Before relying on any latency result, verify the active TQ stage-2 scorer is
actually using the intended block/SIMD path. Record the scorer family, ISA,
flush widths, scalar-tail count, and per-query scorer time. A result dominated
by scalar fallback is not a valid rejection of TurboQuant.

### Phase 2 - Persisted or Index-Side TurboQuant Stage-2 Payload

Wire the TurboQuant stage-2 payload so the scan path does not need to fetch
source f32 vectors before the final rerank boundary.

Required behavior:

- preserve insert, delete, vacuum, and rebuild invariants for the chosen
  payload placement;
- keep malformed or missing payload handling fail-closed or conservative;
- document any durable format change with an ADR before promotion;
- keep source/f32 as the correctness reference and fallback.

### Phase 3 - Scan Counters and Attribution

Expose counters sufficient to prove where the win came from:

- candidates generated by RaBitQ;
- candidates scored by TurboQuant stage-2;
- candidates retained after stage-2;
- exact/source f32 rows fetched or reranked;
- materialized rows;
- compact stage-2 bytes touched;
- source/vector bytes avoided where measurable.

Counters must be available through the existing explain/profile surfaces used
by `ecaz bench suite`.

Carry forward the non-blocking Task 122 reviewer notes:

- keep an off-path unit test for the stage-2 disabled path rather than forcing
  one branch under `#[cfg(test)]`;
- document or preserve the meaning of public diagnostic counters when a
  materialization prune or stage-2 boundary changes what `candidate_row_count`
  represents;
- prefer a dedicated materializations-avoided counter instead of overloading
  truncation counters if the stage-2 implementation needs to distinguish heap
  eviction from pre-materialization discard.

### Phase 4 - Correctness and Recall Gates

Prove the in-engine path preserves final results at the intended recall point.

Required gates:

- focused correctness tests against source/f32 final rerank;
- byte-identical or recall-identical behavior where exact identity is not a
  valid contract;
- explicit rejection of final exact width 10 unless new evidence overturns the
  Task 122 width result.

If recall fails at width 25, diagnose whether the loss is from TQ score quality,
candidate frontier containment, final-rerank width, or implementation overhead.
Do not collapse those into one generic "TQ failed" conclusion.

### Phase 5 - Benchmark Matrix

Run `ecaz bench suite` at 10k, 50k, and 100k before any promotion claim.

Required comparators:

- current RaBitQ + f32 rerank baseline;
- RaBitQ8 stage-2, if the same in-engine stage-2 surface supports it;
- TurboQuant stage-2 with final f32 width 25;
- the current source/f32 reference point for recall and storage context.

Required metrics:

- recall@10 and NDCG@10 where available;
- p50/p95/p99 latency;
- storage per row or per index;
- candidate, stage-2, final-rerank, and materialization counters;
- bytes touched or avoided for compact and source payloads.

### Phase 6 - IO-Sensitive Validation

If the local hot-cache result is close or depends on avoiding source f32 reads,
run one IO-sensitive validation before making a product latency claim:

- cold-cache local;
- remote storage;
- distributed or worker-local rerank; or
- another packet-documented condition where source vector reads dominate.

Do not promote from hot local latency alone if the rationale is IO avoidance.

## Closeout Outcomes

One of:

- **Promote**: matched recall, lower p50/p95/p99 latency, lower final f32 fetch
  or materialization count, and justified storage/IO tradeoff.
- **Iterate**: promising but blocked by payload placement, counters, or one
  benchmark condition.
- **Shelve**: in-engine stage-2 cannot beat the current RaBitQ + f32 path at
  the product-relevant matrix.

## References

- Task 122 closeout: `reviews/task-122/010-closeout-keep-experimental/`
- Task 122 sidecar evidence:
  `reviews/task-122/009-sidecar-tq-stage2-suite/`
- IVF persisted rerank sweep:
  `plan/tasks/111h-ivf-persisted-rerank-format-sweep.md`
- IVF lazy f32 rerank:
  `plan/tasks/112-ivf-lazy-heap-f32-rerank.md`
- IVF bound-aware pruning:
  `plan/tasks/113-ivf-bound-aware-candidate-pruning.md`
