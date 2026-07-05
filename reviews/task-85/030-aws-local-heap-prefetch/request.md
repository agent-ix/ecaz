# Task 85 Packet 030: AWS Local Heap Prefetch

## Summary

Packet 029 implemented explicit local heap-block prefetch before local heap
resolution source-vector scoring while preserving candidate order. This packet
measures that code on AWS 1M/q500 against the packet 023/025 accepted surface
and the packet 027 TID-order rejection.

The result is rejected. The run preserved recall, candidates, and rerank width,
but warm latency did not beat the accepted/control rows:

| Surface | recall@10 | candidate_sum | heap_rerank_sum | p50 ms | p95 ms | p99 ms | max ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| packet 023 accepted repeat | 0.9876 | 9,213,846 | 12,500 | 222.692 | 275.769 | 286.980 | n/a |
| packet 025 locality control repeat | 0.9876 | 9,213,846 | 12,500 | 222.140 | 275.753 | 288.894 | 296.358 |
| packet 027 TID-order repeat | 0.9876 | 9,213,846 | 12,500 | 228.595 | 284.140 | 295.823 | 301.065 |
| packet 030 prefetch repeat | 0.9876 | 9,213,846 | 12,500 | 227.414 | 282.375 | 297.652 | 348.608 |

The prefetch sublever therefore does not satisfy the retained-recall latency
bar and should be removed from the branch.

## Evidence

- Suite config:
  `reviews/task-85/030-aws-local-heap-prefetch/suite-aws-1m-local-heap-prefetch-q500.json`
- Suite report:
  `reviews/task-85/030-aws-local-heap-prefetch/artifacts/aws-1m-local-heap-prefetch-q500/suite-report.md`
- Repeat funnel summary:
  `reviews/task-85/030-aws-local-heap-prefetch/artifacts/aws-1m-local-heap-prefetch-q500/local-heap-prefetch-repeat-summary.json`
- Cloud bench log:
  `reviews/task-85/030-aws-local-heap-prefetch/artifacts/cloud-bench-local-heap-prefetch-q500.log`
- Final AWS pause status:
  `reviews/task-85/030-aws-local-heap-prefetch/artifacts/cloud-status-final-after-local-heap-prefetch-q500-paused.log`

## Notes

The suite completed `3` steps with `0` failures. First-run latency was
`244.557/312.621/2563.915 ms` p50/p95/p99 with the usual cold outlier shape.
The warm repeat preserved the accepted surface exactly:
`recall@10=0.9876`, `candidate_sum=9,213,846`, and
`heap_rerank_sum=12,500`.

Repeat funnel component summary:

- object-read p50/p95: `27.037/28.276 ms`
- candidate-score p50/p95: `57.470/59.308 ms`
- summary-score p50/p95: `47.368/49.192 ms`
- row-score p50/p95: `10.067/10.194 ms`
- rerank-prefix rows: `12,500`
- unique heap blocks p50/p95/max: `22/25/25`
- heap-block transitions p50/p95/max: `24/24/24`

AWS `1m` was paused after the run; final status records `state: paused`.
