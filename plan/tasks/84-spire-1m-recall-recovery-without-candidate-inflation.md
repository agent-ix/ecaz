# Task 84: SPIRE 1M Recall Recovery Without Candidate Inflation

Status: active (2026-06-06)
Owner: coder (to be assigned). One coder, one branch.
Priority: 0 (large SPIRE continuation after Tasks 79-83)

## Why

Tasks 79-83 narrowed the SPIRE 1M recall/latency problem to a specific
tradeoff:

- Task 79/81 retained surface: `recall@10=0.9832`,
  `candidate_sum=9,213,846` on AWS 1M/q500.
- Task 82 attributed the remaining `84/5000` missed truth rows: `3` routing
  misses and `81` selected-leaf block-pruning/candidate-budget misses.
- Task 83 proved all `81` selected-leaf misses had target blocks ranked outside
  the retained `global1152` block cap.
- Task 83 also showed blanket cap increases recover recall only by growing the
  q500 candidate surface:
  - `global1280`: `recall@10=0.9846`, `candidate_sum=10,237,554`
  - `global1536`: `recall@10=0.9876`, `candidate_sum=12,284,852`
  - `global1664`: `recall@10=0.9892`, `candidate_sum=13,308,518`

That means the next meaningful SPIRE project is not another tiny cap sweep. It
is a larger recall-recovery program: recover the selected-leaf misses by making
block choice better or selectively spending extra budget only where it is
earned, while preserving the Task 79/81 candidate surface as the first-class
acceptance bar.

## Objective

Recover AWS 1M/q500 SPIRE recall above the retained `0.9832` point without
returning to broad candidate inflation. The implementation may combine block
scoring, calibration, route-aware priors, selected-leaf confidence signals, and
bounded selective rescue, but it must prove the improvement against the
Task 79/81 retained candidate baseline.

This is intentionally a larger task. It should contain multiple review packets
and implementation checkpoints rather than being treated as a single narrow
slice.

## Scope

### 1. Baseline and Attribution Harness

- Promote the Task 83 target-block containment diagnostic into a repeatable
  task-local analysis harness if needed.
- Produce a baseline packet that joins, for the `81` selected-leaf misses:
  block rank, block score, leaf route rank, route score, row position, block
  size, assignment flags, summary radius/contrast signals, and whether nearby
  blocks were selected.
- Keep all benchmark matrices under `ecaz bench suite`; extend the suite runner
  if a missing analysis step would otherwise require shell glue.

### 2. Block-Scoring Recovery

Investigate and implement scoring changes that can move true target blocks
inside the retained `global1152` cap:

- route-prior weighting and route-rank normalization;
- summary radius / block contrast calibration;
- multi-representative or endpoint-summary variants when evidence shows the
  current summary under-ranks target blocks;
- per-leaf or per-route normalization only if it does not reintroduce the
  full-leaf candidate surface.

Each scoring change must have a packet that records the hypothesis, code
change, local validation, and 100k or 1M evidence before moving on.

### 3. Selective Near-Cap Rescue

If scoring alone cannot recover enough misses, implement a bounded rescue path
that spends extra block budget only for queries/leaves with measured ambiguity:

- near-cap rescue windows keyed by score margin, not a blanket higher cap;
- per-query or per-leaf rescue limits with a hard global candidate budget;
- deterministic ordering and stable behavior under PG18 repeated runs;
- query metrics that expose how often rescue triggers, how many blocks it adds,
  and how many final candidates it contributes.

The rescue path must be disabled or neutral by default until AWS evidence shows
it improves the retained recall/candidate tradeoff.

### 4. Latency and Candidate Guardrails

Candidate control is part of the product requirement, not a secondary metric.

Acceptance comparisons must always include:

- retained baseline: `global1152`, `recall@10=0.9832`,
  `candidate_sum=9,213,846`;
- Task 83 cap-sweep controls: `global1280`, `global1536`, `global1664`;
- q500 `candidate_sum`, `heap_rerank_sum`, p50, p95, p99, and route totals;
- selected-leaf miss recovery counts, split from routing misses.

Do not accept a policy that merely recreates the Task 80/83 global-cap behavior
under another name.

## Evidence Plan

Use staged evidence so the task remains large but still reviewable:

- Local 100k iteration packet for scoring/rescue correctness and fast feedback.
- AWS 1M/q500 packet for any candidate policy that looks promising locally.
- Optional AWS or local profiling packet if candidate counts stay flat but
  latency regresses.
- ADR packet if the accepted policy changes durable SPIRE behavior, exposed
  GUCs, reloptions, or persisted index metadata.

All benchmark and measurement runs must use `ecaz bench suite` with checked-in
suite configs and packet-local artifacts.

## Gates

- Establish a current Task 84 baseline packet against the Task 79/81 retained
  surface before landing recovery code.
- For any scoring or rescue policy, report selected-leaf miss recovery separately
  from routing miss recovery.
- Preserve or improve candidate surface versus the retained `9.21M` q500
  baseline, or justify any small candidate increase with a materially better
  recall/latency point than the Task 83 blanket-cap sweep.
- Show AWS 1M/q500 recall, `candidate_sum`, `heap_rerank_sum`, p50, p95, and
  p99 before accepting a policy.
- Pause AWS `1m` after every AWS run and capture packet-local final status.

## Exit Criteria

- One or more code commits land the accepted recovery policy, or the task closes
  with evidence that no bounded recovery path is justified.
- Review packets under `reviews/task-84/` record each major checkpoint,
  artifacts, manifests, and reviewer-visible results.
- If a policy lands, it has focused PG18 validation and AWS 1M/q500 evidence.
- If no policy lands, the closeout must include the strongest rejected options,
  their recall/candidate/latency results, and the next concrete recommendation.
- AWS `1m` is paused at closeout.
