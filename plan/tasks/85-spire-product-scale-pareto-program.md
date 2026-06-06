# Task 85: SPIRE Product-Scale Pareto Program

Status: proposed (2026-06-06)
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

### 4. Default and Operator Policy

Decide whether SPIRE has a product-ready 1M profile:

- keep current defaults unchanged;
- introduce an explicit high-recall or balanced profile;
- change SPIRE defaults;
- or close with evidence that SPIRE remains research/opt-in at 1M.

Any default/profile change needs durable documentation and, if it changes
behavioral contracts, an ADR.

### 5. Benchmark and Review Discipline

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
  is justified yet, plus the next concrete research direction.

In all cases, closeout must include the strongest accepted and rejected options,
their AWS 1M/q500 recall/candidate/latency rows, and the final AWS pause status.
