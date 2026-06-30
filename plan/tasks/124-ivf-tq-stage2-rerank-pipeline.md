# Task 124: IVF TurboQuant Stage-2 Rerank Pipeline

Status: **COMPLETION CANDIDATE — all seven TQ scorer levers implemented/measured; reduced-dimension workload validation failed the recall contract** (2026-06-30).
Reopened by the user after the `027` shelve. The shelve is REJECTED: it claimed
TQ speed levers were "exhausted," but **the TQ scoring kernel itself was never
touched at the time of that closeout.** The pre-reopen code changes were IVF
scan/rerank/options plumbing in `src/am/ec_ivf/`; nothing in the actual
TurboQuant scorer at `src/am/common/candidate_batch/` or `src/quant/`. See
`reviews/task-124/027-speed-closeout-shelve/feedback/2026-06-30-02-reviewer.md`.
Packet `reviews/task-124/035-post-scorer-product-suite/` is retained only as
post-scorer 4-bit TQ validation evidence. Its earlier product-promotion
closeout framing was rejected by reviewer feedback and is superseded. Packet
`reviews/task-124/036-tq2-real-index-validation/` validates TQ2 in a real IVF
index: the SIMD scorer row now appears in the workload, but recall remains
unchanged from packet 008 and still broken at 50k/100k, so TQ2 SIMD must not be
claimed as a usable TQ speedup. Packet
`reviews/task-124/037-tq2-dim768-real-index/` validates a real reduced-dimension
TQ2 format (`rerank_format=turboquant2_768`) in real IVF indexes at 10k / 50k /
100k. It cuts in-engine TQ2 scorer elapsed by about 49-53% versus full-dim TQ2,
but recall remains broken at 50k/100k, so reduced-dimension TQ2 also must not be
claimed as a usable stage-2 speedup. Task 130 removes the validation-only
`turboquant2_768` reloption from the production-facing code surface while
preserving packet 037 as negative evidence.
Owner: coder (to be assigned). One coder, one branch.
Priority: P1.

## PRIMARY INTENT — READ FIRST (non-negotiable)

The single goal of this task is to **make TurboQuant itself faster** — the TQ
scoring/compute path. This intent has now been restated 4-5 times and keeps not
happening; the work keeps drifting into plumbing, baseline comparisons, and
storage/promotion verdicts. To remove all ambiguity:

Every slice MUST report a **TQ-internal before/after speed delta measured on the
TQ scorer** (e.g. ns/candidate or TQ scorer elapsed µs), not end-to-end query
latency that is dominated by the shared coarse frontier.

**Out of scope — do NOT bring these up as the answer:**

- comparisons against the f32/source baseline — this task is not a bake-off;
- storage size, "is it worth it," promotion, or product-competitiveness
  judgments — not the question here, and not the reviewer's or coder's call;
- IVF / RaBitQ frontier (`nprobe`) tuning — shared work, not TurboQuant;
- further scan-path materialization/allocation micro-tweaks — that envelope is
  already at its floor (see State At Reopen) and is NOT where remaining TQ speed
  lives.

A slice that returns an f32 comparison, a storage/promotion verdict, or an
nprobe-frontier result instead of a TQ-scorer speed delta does **not** satisfy
this task.

## State At Reopen — done vs never attempted

TQ speed changes that worked (kept) — all memory-traffic reductions **around**
the scorer:

- selected-payload loader (003): TQ decode `1202 µs → 514 µs`, segment bytes
  `2.84 MB → 1.75 MB` at 100k/nprobe64;
- contiguous slab (011); final exact width `25 → 15` (005); group-width locality
  (004): group-16 cut segment reads `1.75 MB → 147 KB`.

TQ speed changes that failed (reverted) — micro-allocation/addressing tweaks;
this materialization micro-overhead is genuinely at its floor:

- top-k fusion (012), compact group headers (013), direct-slot rerank (014),
  vector-index lookup (018), borrowed score buffer (020), slab-vector lookup
  (025).

The `027` closeout claim that TQ speed levers were "exhausted" is INCORRECT and
is superseded by this reopen.

## Required Next Phase — TQ speed levers never attempted (ALL REQUIRED, NO DEFERMENTS)

Every one of the following is **required** work for this task. None may be
deferred, descoped, or closed as "not worth it." Each is a TQ-scorer/compute-path
optimization and each must land with a measured TQ-internal before/after speed
delta on the TQ scorer:

1. **The TQ scoring kernel itself** — register/accumulator blocking, LUT layout,
   prefetch, batch-accumulation width, dequant/FMA fusion. Never profiled. This
   is the biggest untouched lever.
2. **Per-query LUT / query-prep cost** — never measured (on the task's lever
   list).
3. **Batch/flush width** — confirmed at 100, never swept as a throughput lever.
4. **Dimension/subspace reduction** — only bit-depth (4/2/1) was tried, never
   fewer dims.
5. **TQ2 with a real SIMD kernel** — TQ2 was rejected as "slow," but its packet
   shows it ran scalar (no kernel was written); its SIMD speed was never
   measured.
6. **QJL scoring speed** — all work was no-QJL 4-bit; QJL never speed-tested.
7. **Prefetch / pipelining payload reads ahead of scoring.**

This task is not complete until all seven are implemented and measured with TQ
scorer before/after deltas in the relevant workload where a production path
exists. Packets 028-034 provide scorer-level evidence; packet 035 provides
4-bit TQ in-engine validation; packet 036 shows TQ2 now has in-engine SIMD
attribution but remains recall-broken in the real index; packet 037 adds and
validates a real reduced-dimension TQ2 format and shows the scorer win is real
but recall-broken. Task 130 is the cleanup lane that removes that failed
validation-only format from the callable reloption surface.

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

Correction 2026-06-29: packet `015-tq-phase6-local-cache-evict` attempted this
locally, but the macOS `F_NOCACHE` helper was per-fd and did not evict
PostgreSQL's separate relation reads. Treat packet 015 as an attempted local run,
not controlled cold-cache evidence.

If the local hot-cache result is close or depends on avoiding source f32 reads,
run one IO-sensitive validation before making a product latency claim:

- cold-cache local;
- remote storage;
- distributed or worker-local rerank; or
- another packet-documented condition where source vector reads dominate.

Do not promote from hot local latency alone if the rationale is IO avoidance.

## Closeout Outcomes

Correction 2026-06-30: packet `035-post-scorer-product-suite` is **not** a
Task 124 closeout. It reran the 10k / 50k / 100k 4-bit TQ matrix on current
HEAD and validated one in-engine scorer result: the TQ scorer component
improved versus packet 026 (`1.811008 ms -> 1.779246 ms` at 10k,
`1.851708 ms -> 1.788211 ms` at 50k, `1.907458 ms -> 1.804748 ms` at 100k)
while remaining on the NEON/SIMD path. Reviewer feedback rejected the product
promotion/shelve framing because TQ2 and reduced-dimension wins were still only
microbenchmark results. Packet `036-tq2-real-index-validation` resolves the TQ2
workload-validation gap and shows TQ2 recall is unchanged from packet 008 and
still broken at 50k/100k. Packet `037-tq2-dim768-real-index` resolves the
remaining reduced-dimension workload-validation gap with a real index-side
format and 10k / 50k / 100k recall + latency + TQ scorer attribution. The
reduced-dimension scorer delta is real (`~49-53%` lower TQ2 scorer elapsed than
full-dim TQ2), but recall is also broken at 50k/100k, so the correct closeout
candidate is **Shelve**, not promotion. Task 130 records the follow-up cleanup
that keeps packet 037 as evidence but removes the failed validation-only
`turboquant2_768` reloption from the production-facing code surface.

~~Closeout 2026-06-30: **Shelve**.~~ **SUPERSEDED / REOPENED 2026-06-30.** The
prior shelve was rejected by the user: it asserted TQ speed levers were
"exhausted," but the TQ scoring kernel and six other TQ-compute-path levers were
never attempted (see Required Next Phase). The in-engine TQ stage-2 path was
implemented, instrumented, and benchmarked at 10k / 50k / 100k, and the
materialization envelope around the scorer was optimized — but the scorer compute
path itself was never touched. At that point, the task had to remain **open**
until the seven required TQ-scorer speed levers were implemented and measured.
No shelve/closeout was permitted while any remained unattempted.

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
