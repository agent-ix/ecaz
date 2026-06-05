# Task 83: SPIRE Selected-Block Containment Recovery

Status: active (2026-06-05)
Owner: coder (to be assigned). One coder, one branch.
Priority: 0 (Task 82 follow-up)

## Why

Task 82 attributed the retained AWS 1M/q500 SPIRE recall gap at the optimized
Task 79/81 surface:

- retained surface: `recall@10=0.9832`, `candidate_sum=9,213,846`
- q500 truth rows: `5,000`
- missed truth rows: `84`
- pure routing misses: `3`
- selected-leaf block-pruning/candidate-budget misses: `81`

The gap is therefore inside selected leaves, not primarily in top-graph routing
breadth. The next useful slice is to determine whether those `81` missed truth
rows live in blocks that narrowly miss the global block cap, and then recover
recall with a bounded selected-block scoring/pruning change if the evidence
supports it.

## Scope

Add a target-only selected-block containment diagnostic that can answer, for a
truth row in a selected leaf:

- whether the row's assigned leaf was routed and loaded;
- which leaf block contains the row;
- that block's global summary rank for the query;
- whether the retained global block cap selected that block;
- how far outside the cap the missed block landed when it was not selected.

Use the diagnostic to choose and validate one narrow recall-recovery slice only
if it can plausibly improve recall above `0.9832` without recreating the
wide-top-graph candidate explosion. Candidate recovery should preserve the
Task 79/81 optimized candidate surface as the comparison baseline.

## Required Evidence

- Use `ecaz bench suite` for all benchmark or attribution runs.
- Reuse the AWS 1M/q500 truth cache from
  `benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/truth-aws-real-1m-q500-k10.json`.
- Compare against Task 82 retained-surface evidence, not the old pre-Task-79
  full-leaf candidate surface.
- Capture all durable evidence under `reviews/task-83/`.
- Preserve AWS cost hygiene: pause `1m` after each AWS run and capture
  packet-local final status.

## Gates

- Produce a selected-block containment table for the Task 82 missed truth rows.
- Quantify missed-block ranks relative to the retained global block cap
  (`1152`), including counts inside cap, near-cap, and far outside cap.
- If implementing a recovery policy, show AWS 1M/q500 recall, candidate_sum,
  p50, p95, and p99 against the Task 82 baseline.
- Do not accept a policy that only recovers recall by returning to the
  `251M-495M` q500 candidate ceiling observed in the top-graph recall packet.

## Exit Criteria

- A review packet under `reviews/task-83/` records the diagnostic method,
  suite config, commands, artifacts, and key result rows.
- If a recovery policy lands, it has focused PG18 validation and AWS 1M/q500
  evidence.
- If no narrow recovery policy is justified, close with the measured rank
  distribution and the next concrete recommendation.
- AWS `1m` is paused at closeout.
