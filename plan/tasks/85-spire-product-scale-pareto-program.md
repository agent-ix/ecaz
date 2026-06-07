# Task 85: SPIRE Product-Scale Pareto Program

Status: in progress - reopened for comprehensive same-recall latency plan (2026-06-07; correction after premature closeout `reviews/task-85/007-product-scale-closeout/`)
Owner: coder (to be assigned). One coder, one branch.
Priority: 0 (large SPIRE continuation after Task 84)

## Why

Tasks 79-84 changed the SPIRE optimization problem from "find any high-recall
configuration" into a product-scale Pareto problem.

The retained AWS 1M/q500 surface after Tasks 79/81 is:

- `global1152`: `recall@10=0.9832`, `candidate_sum=9,213,846`
- `heap_rerank_sum=12,500`
- selected-leaf misses dominate the gap: `81` selected-leaf block misses vs
  `3` routing misses

Tasks 82-84 then established important negative evidence:

- wider top-graph routing recovers recall by exploding candidates;
- blanket global block caps recover recall only by growing the candidate
  surface;
- k=3 block-summary representatives did not improve the retained 1M recall
  point at matched cap/candidates.

That is enough to stop treating each knob as a separate small task. The next
SPIRE project should be a larger engineering lane that turns the retained
candidate-surface work into a complete product profile: recall, latency,
candidate control, storage, defaults, and operator evidence all measured
together.

## Objective

Deliver or reject a product-scale SPIRE Pareto point for 1M-row workloads that
is materially better than the retained Task 79/81 point and defensible against
single-node IVF/HNSW/DiskANN comparators.

The task should contain multiple implementation and measurement checkpoints. A
single narrow tuning slice is not enough to complete it.

## Scope

### 1. Task 84 Handoff Gate

Start only after Task 84 has either:

- landed an accepted recall-recovery policy; or
- closed with evidence that no bounded selected-block recovery policy is
  justified.

Task 85 must inherit Task 84's accepted/rejected policy evidence rather than
rerunning the same scoring, k-summary, or blanket-cap experiments as standalone
slices.

### 2. Product Pareto Baseline

Establish a current AWS 1M/q500 SPIRE baseline packet against:

- the retained Task 79/81 surface;
- the accepted Task 84 recovery point, if one lands;
- Task 83 blanket-cap controls;
- current single-node IVF, HNSW, and DiskANN comparator rows where existing
  evidence is stale or incomplete.

Report recall@10, p50/p95/p99 latency, `candidate_sum`,
`heap_rerank_sum`, route totals, storage bytes/row, build time, and AWS host
profile.

### 3. Matched-Recall Latency Work

Pursue latency only at matched or improved recall. Candidate reduction remains
the first-class lever, but the task may also include:

- selected-block scoring or rescue policy refinement after Task 84;
- candidate scoring CPU reductions that preserve the selected candidate set;
- object-read and heap-rerank locality improvements;
- RaBitQ-specific SPIRE scoring/layout work;
- route/cache reuse only where it preserves correctness and does not hide
  candidate growth.

Do not accept a latency win that is just lower recall, lower rerank width, or a
renamed blanket cap.

### 4. Comprehensive Optimization Program

Task 85 is not complete after identifying the first failed candidate option.
The negative evidence from Tasks 80-84 and Task 85 packets 001-007 must be
converted into a larger implementation plan and then worked as part of this
task unless a packet-local stop condition proves the whole program is no longer
worth pursuing.

Any direction that would otherwise be described as "future research" is task
scope once it is identified as the next plausible same-recall latency lever.
It must be tracked as a required workstream, not as a vague follow-up. Each
workstream below must end in one of:

- an implementation checkpoint with packet-local local/AWS evidence;
- a measurement checkpoint proving it is not the current bottleneck; or
- a stop-condition checkpoint explaining why the design is infeasible or not
worth pursuing before implementation.

Task 85 cannot close by naming a future direction that has not passed through
one of those exits. Closeout may name follow-up work only after the required
workstreams below have been attempted or explicitly rejected with evidence.

The remaining optimization program is:

#### 4.1 Object-Read And Physical Layout

Goal: reduce the object-read component of the retained block16/global1152
surface without changing the selected candidate set or recall semantics.

Evidence to preserve:

- retained Task 79/81 bar: recall@10 `0.9832`,
  `candidate_sum=9,213,846`;
- packet 006 warm block16 repeat: object read p50 `183.712 ms`, score p50
  `56.872 ms`;
- packet 006 block32/global1152: object read p50 `167.198 ms`, score p50
  `44.204 ms`, but only by doubling candidates to `18,413,851`.

Required work:

- design a SPIRE V5 or equivalent layout that supports partial segment reads
  for selected blocks;
- evaluate a hot/cold split where routing summaries, block summaries, and
  candidate payload needed for pruning are separated from full rerank payload;
- add row/block locators if the current object layout forces reads of
  unrelated block payload;
- measure read bytes, read calls, block payload touched, and per-query object
  read latency in packet-local AWS 1M/q500 suites.

Acceptance bar:

- recall@10 >= retained Task 79/81 recall;
- no increase in `heap_rerank_sum`;
- `candidate_sum` no higher than retained unless paired with a separately
  justified recall gain;
- material p50/p95 improvement versus retained block16/global1152 on the same
  AWS profile and q500 suite.

#### 4.2 Summary Scoring CPU

Goal: reduce summary-scoring CPU for the retained selected-block policy rather
than lowering latency by selecting fewer or different blocks that lose recall.

Required work:

- profile block-summary scoring on the retained block16 surface;
- optimize the RaBitQ summary score path, decode path, and memory layout;
- evaluate SIMD/cache-friendly summary batches only when they preserve score
  ordering or have packet-local recall proof;
- report score p50/p95, total summary candidates, and recall deltas.

Acceptance bar:

- same retained candidate set or packet-local proof that any score-ordering
  change preserves recall;
- score CPU improvement visible in AWS 1M/q500 funnel output;
- end-to-end p50/p95 improvement without lower recall.

#### 4.3 Candidate-Set-Preserving Rerank Locality

Goal: improve heap/rerank locality and tuple access after candidate selection,
without lowering rerank width or hiding work in cache warmup.

Required work:

- measure heap rerank access locality and tuple fetch/read amplification for
  the retained point;
- evaluate candidate ordering, TID grouping, block-local rerank batches, or
  prefetch scheduling that preserves the exact candidate set;
- keep cold/warm results separated in all reports.

Acceptance bar:

- `heap_rerank_sum` unchanged unless recall improves;
- recall@10 >= retained Task 79/81 recall;
- latency improvement reported with the same warm/cold policy as the retained
  baseline.

#### 4.4 Candidate-Surface Redesign Only With Recall Preservation

Goal: continue candidate reduction only where it is not a disguised recall
tradeoff.

Rejected paths should not be rerun as standalone slices:

- blanket caps that recover recall only by growing candidates;
- k-summary variants that do not beat the retained point;
- per-leaf caps that change the selected candidate surface and collapse recall;
- block geometry changes that require candidate inflation for same recall.

Allowed work:

- new scoring or rescue policies that target the known selected-leaf miss set
  while preserving or improving retained recall;
- learned or calibrated block policies only if they beat retained recall and
  latency at AWS 1M/q500;
- diagnostics that explain exactly which misses move and what candidate cost
  pays for them.

#### 4.5 Benchmark Harness And Evidence Extensions

Goal: make the benchmark evidence strong enough that a product/default decision
is reviewable.

Required work:

- extend `ecaz bench suite` rather than adding shell sweepers when new
  metrics are needed;
- add funnel metrics for object read bytes/calls, summary score CPU, tuple
  fetch locality, and cache state where missing;
- keep every benchmark packet-local under `reviews/task-85/` and promote
  current lanes only from immutable accepted benchmark packets;
- always compare against retained Task 79/81 and same-suite controls, not
  pre-Task-79 candidate counts.

#### 4.6 Comparator And Product Policy Gate

Goal: decide whether SPIRE is product-ready only after the implementation
workstreams above have been tried or rejected with evidence.

Required work:

- keep ec IVF/RaBitQ, ec DiskANN, and available HNSW/external comparator rows
  in the closeout table;
- explicitly state where SPIRE is slower/faster, larger/smaller, or has a
  recall advantage;
- update defaults or profiles only with ADR-backed rationale;
- close SPIRE as research/opt-in only after the comprehensive workstreams have
  packet-local results or a documented stop condition.

### 5. Default and Operator Policy

Decide whether SPIRE has a product-ready 1M profile:

- keep current defaults unchanged;
- introduce an explicit high-recall or balanced profile;
- change SPIRE defaults;
- or close with evidence that SPIRE remains research/opt-in at 1M.

Any default/profile change needs durable documentation and, if it changes
behavioral contracts, an ADR.

### 6. Benchmark and Review Discipline

All benchmark matrices must run through `ecaz bench suite`. If a needed
measurement is missing from the suite runner, extend the runner first and land
that as its own checkpoint.

All evidence must be packet-local under `reviews/task-85/`, with immutable
benchmark packets under `benchmarks/` where appropriate and promoted current
lanes updated only from cited immutable packets.

## Gates

- Task 84 handoff is explicit and cited.
- Every accepted optimization reports recall, p50/p95/p99, candidates,
  heap-rerank count, and storage/build impact.
- AWS 1M/q500 is the acceptance scale for product claims.
- Local 100k evidence is allowed for iteration but cannot complete the task.
- Comparisons use the retained Task 79/81 candidate baseline and Task 83 cap
  controls, not pre-Task-79 candidate surfaces.
- AWS `1m` is paused after every AWS run and final status is captured in the
  owning packet.

## Exit Criteria

One of:

- a SPIRE 1M product profile lands with AWS 1M/q500 evidence showing a better
  recall/latency/candidate Pareto point than the retained Task 79/81 surface;
- SPIRE defaults or profiles are updated with ADR-backed rationale and current
  benchmark promotion;
- the task closes with packet-local evidence that no product-scale Pareto point
  is justified yet after the required workstreams have either produced results
  or reached explicit stop conditions.

In all cases, closeout must include the strongest accepted and rejected options,
their AWS 1M/q500 recall/candidate/latency rows, and the final AWS pause status.

## Closeout

Packet `reviews/task-85/007-product-scale-closeout/` was a premature closeout:
it correctly rejected the measured Task 85 options under the same-recall
latency bar, but incorrectly treated the next research direction as out of
scope. This task is reopened so those directions are part of the Task 85
program rather than a vague follow-up.

The retained Task 79/81 block16/global1152 point remains the baseline to beat.
Task 85 rejected block8, per-leaf caps, and block32 geometry as product
profiles because each option either lost recall, worsened latency, or required
candidate inflation that was not justified by the small same-recall latency
movement.

The remaining work is the comprehensive optimization program above: physical
layout/read-path changes, summary-scoring CPU reductions, rerank locality,
candidate-set-preserving scoring, benchmark harness extensions, and a final
product/default policy gate.

The phrase "future research direction" is no longer acceptable as a Task 85
escape hatch. If the direction is the best known path to retained-recall
latency improvement, it belongs in this task until a checkpoint proves it
should stop.

## Checkpoints

- `reviews/task-85/009-funnel-read-score-breakdown/`: benchmark harness
  checkpoint. `ecaz bench spire-pipeline --funnel-output` now carries
  object/summary/row bytes, selected/skipped block counts, and split
  summary-vs-row score timings. This enables the next AWS 1M/q500 retained
  recall run to choose a real read-path or summary-scoring optimization.
- `reviews/task-85/010-aws-retained-funnel-breakdown/`: AWS 1M/q500 retained
  funnel measurement. The retained block16/global1152 repeat run produced
  `recall@10=0.9876`, `candidate_sum=9,213,846`, `heap_rerank_sum=12,500`,
  `p50=224.787 ms`, `p95=281.079 ms`, and `p99=292.543 ms`. Funnel metrics
  show object reads dominate: per-query repeat p50 read bytes were
  `684,831,192` total, including `610,463,408` row bytes and `74,357,224`
  summary bytes; object-read p50 was `181.330 ms` versus summary-score p50
  `47.541 ms` and row-score p50 `10.121 ms`. This makes read-path/layout
  reduction the next required workstream before CPU-only micro-optimization.
- `reviews/task-85/011-row-segment-read-amplification/`: benchmark harness
  checkpoint. The leaf candidate snapshot and `ecaz bench spire-pipeline`
  funnel output now distinguish total routed row-object storage bytes from
  actual selected row-segment reads with `leaf_row_segment_read_count` and
  `leaf_row_segment_read_bytes`. This is required before deciding whether a
  block-aligned V5 layout or other physical read-path change can reduce the
  retained block16/global1152 object-read component without changing recall.
  `cargo fmt --check` passed; compile validation was attempted but Cargo
  timed out before spawning `rustc` in the current environment, so the next
  checkpoint must re-run focused compile/tests before AWS deployment.
