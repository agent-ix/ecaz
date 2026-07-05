# Task 85 Review Request: Joint Miss Targeting

## Summary

This packet turns the retained 1M/q500 baseline miss evidence into a Task 85
mechanism decision. It joins the Task 85 warm retained miss-attribution rows
with Task 84 enriched target-block context so the next latency slice is based on
the joint distribution of miss stage, block rank, route rank, and margin to the
global cap.

## Result

The retained warm row has 84 missed truth neighbors over q500. The joint
distribution is:

| Bucket | Count |
| --- | ---: |
| routing miss / no target context | 3 |
| selected-leaf/candidate-cap misses with target context | 81 |
| block rank 1153..1280 | 7 |
| block rank 1281..1536 | 15 |
| block rank 1537..2048 | 19 |
| block rank >2048 | 40 |

Block-rank stats for the 81 contextual misses: min `1154`, p50 `2014`, p75
`3501`, p90 `5848`, p95 `8789`, max `11559`.

Route-rank stats for the same misses: min `1`, p50 `15`, p75 `32`, p90 `50`,
p95 `69`, max `88`.

## Interpretation

The miss distribution is too broad for a small bounded recovery band to be a
credible latency optimization. Only `7/84` misses fall in the `1153..1280` band,
and even `global2048` would cover only `41/84` while materially increasing the
candidate surface. That matches the Task83 control result: `global1280` and
`global1536` recover recall by spending more candidates and do not beat the
Task85 warm latency floor.

The next latency mechanism should therefore target candidate density, not a
post-hoc block-cap expansion. The most direct remaining axis is a smaller
leaf-block geometry (`leaf_block_rows=8`) because it can reduce candidate rows
per selected block while preserving routing shape and rerank width. It must
still be judged only against the Task85 warm floor:

- recall@10 must stay at or above `0.9832`;
- p50/p95/p99 must beat `246.397 ms` / `304.476 ms` / `321.342 ms`;
- candidate_sum should drop below `9,213,846`;
- the result must include storage/build impact.

## Evidence

- `artifacts/joint-miss-records.json`: joined miss records.
- `artifacts/joint-miss-summary.json`: aggregate counts and rank percentiles.

Inputs:

- Task85 warm retained miss attribution:
  `reviews/task-85/001-handoff-product-baseline-suite/artifacts/aws-1m-product-baseline-q500/miss-attribution-retained-global1152-q500-repeat.jsonl`
- Task84 enriched block context:
  `reviews/task-84/001-enriched-block-context-diagnostic/artifacts/aws-1m-enriched-block-context-q500/target-block-context-spire-1m-global1152-q500.jsonl`

## Requested Review

Please review whether this joint distribution supports moving Task85 to the
block_rows=8 product-scale slice instead of another cap/recovery sweep.
