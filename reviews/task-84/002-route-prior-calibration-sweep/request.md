# Task 84 Review Request: Route Prior Calibration Sweep

## Summary

This packet tests the first recovery hypothesis from the enriched baseline:
selected-leaf misses are often already in plausible routed leaves, so adding a
small route prior to global block summary scoring may recover truth blocks
inside the fixed `global1152` block budget.

The sweep does not change code. It exercises the existing
`ec_spire.leaf_block_pruning_route_prior_weight` GUC at the retained AWS 1M/q500
surface:

- `global1152`
- `nprobe=96`
- `rerank_width=25`
- `summary_radius_weight=0.25`
- route prior weights: `0.02`, `0.05`, `0.10`, `0.20`

## Requested Review

Please review the suite shape before AWS execution. The acceptance question is
whether any route-prior point improves `recall@10` above `0.9832` while keeping
`candidate_sum` at or below the retained `9,213,846` baseline, or clearly better
than the Task 83 blanket-cap controls if it moves candidate volume slightly.

