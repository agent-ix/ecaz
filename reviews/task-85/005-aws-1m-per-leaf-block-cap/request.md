# Task 85 Review Request: AWS 1M Per-Leaf Block Cap Result

## Summary

Packet 004 rejected `leaf_block_rows=8`: it halved candidate rows at the
retained recall point but worsened p50/p95 because leaf object read and
summary/row scoring time increased. This packet tested a query-only alternative
on the retained block16 index: use a per-leaf block cap instead of a global
block cap.

Verdict: reject per-leaf block caps as a Task85 latency path. They do not
preserve recall, and they are far slower than the same-suite global1152 warm
control.

## Results

`reviews/task-85/005-aws-1m-per-leaf-block-cap/artifacts/aws-1m-per-leaf-block-cap-q500/suite-report.md`

| Row | Recall@10 | p50 | p95 | p99 | Candidate Sum | Verdict |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| global1152 control first | 0.9832 | 271.674 ms | 337.657 ms | 349.454 ms | 9,213,846 | cold/control |
| perleaf8 | 0.7108 | 479.673 ms | 498.422 ms | 505.507 ms | 6,142,746 | recall loss, slower |
| perleaf10 | 0.7454 | 474.696 ms | 491.903 ms | 500.470 ms | 7,678,346 | recall loss, slower |
| perleaf12 | 0.7714 | 477.209 ms | 494.663 ms | 500.243 ms | 9,213,924 | same candidate budget, recall loss, slower |
| perleaf14 | 0.7980 | 486.028 ms | 503.404 ms | 510.191 ms | 10,749,465 | recall loss, slower, more candidates |
| global1152 control repeat | 0.9832 | 249.160 ms | 306.730 ms | 318.763 ms | 9,213,846 | warm floor |

The headline `perleaf12` row was the matched-candidate-budget test. It had
nearly the same candidate sum as global1152 but recall collapsed from `0.9832`
to `0.7714`, and p50/p95 worsened from `249.160/306.730 ms` to
`477.209/494.663 ms`.

## Timing Diagnosis

Funnel timing counters show why per-leaf selection is slower despite comparable
candidate count at `perleaf12`:

| Row | p50 Object Read | p50 Candidate/summary Score | Candidate Sum |
| --- | ---: | ---: | ---: |
| global1152 warm repeat | 170.822 ms | 55.787 ms | 9,213,846 |
| perleaf12 | 350.475 ms | 56.665 ms | 9,213,924 |

Per-leaf selection preserves similar candidate scoring cost but roughly doubles
leaf object read time. Equal per-leaf allocation also destroys recall because it
cannot concentrate block budget on the leaves where the global scorer sees the
best block evidence.

## Validation

- `ecaz bench suite audit`: passed for 7 steps.
- AWS suite: completed 7, failed 0, skipped 0.
- AWS profile `1m`: paused after the run.

## Requested Review

Please review the per-leaf cap rejection as Task85 latency evidence. This closes
the cheap query-policy alternative to global block allocation at the retained
block16 geometry.
