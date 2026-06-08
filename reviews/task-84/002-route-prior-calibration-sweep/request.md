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

## AWS Result

The AWS 1M/q500 sweep completed and AWS was paused afterward. Compact result
table: `artifacts/route-prior-summary.tsv`.

| route prior | recall@10 | candidate_sum | p50 | p95 | p99 | miss split |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| `0.02` | `0.9832` | `9,213,802` | `279.228 ms` | `351.570 ms` | `364.541 ms` | `4916/3/81` |
| `0.05` | `0.9832` | `9,213,740` | `256.523 ms` | `320.070 ms` | `334.609 ms` | `4916/3/81` |
| `0.10` | `0.9832` | `9,213,619` | `255.472 ms` | `317.784 ms` | `334.127 ms` | `4916/3/81` |
| `0.20` | `0.9832` | `9,213,310` | `255.583 ms` | `319.458 ms` | `334.041 ms` | `4916/3/81` |

All route-prior points preserved the same selected-leaf miss identity as packet
001: zero symmetric-diff rows for the `81`
`selected_leaf_block_pruning_or_candidate_cap` query/rank pairs.

Target-block cap containment worsened as route prior increased:

- `0.02`: `84` ranked target blocks outside cap.
- `0.05`: `86` outside cap.
- `0.10`: `91` outside cap.
- `0.20`: `109` outside cap.

## Conclusion

Route-prior weighting is not the Task 84 recall recovery path. It slightly
reduced `candidate_sum` and improved latency versus packet 001's diagnostic
baseline, but it recovered none of the retained `81` selected-leaf misses and
increasing the weight displaced additional truth target blocks from the fixed
cap.

The next Task 84 slice should move from leaf-level route prior to block-summary
calibration: radius/contrast scoring or a bounded ambiguity metric that targets
the under-ranked block summaries directly.

## Requested Review

Please review the route-prior rejection evidence and whether the next recovery
slice should focus on summary radius/contrast calibration before any selective
near-cap rescue policy.
