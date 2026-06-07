# Task 85 Packet 027: AWS Local Heap Fetch Ordering

## Summary

Packet 026 implemented the first candidate-set-preserving rerank locality
lever: local heap-resolution fetches are sorted by decoded heap TID before
source-vector loading and exact scoring. This packet measures that change on
AWS 1M/q500 against the accepted packet 023/025 surface.

The result is rejected. The run preserved recall, candidates, and rerank width,
but warm latency regressed versus packet 025:

| Surface | recall@10 | candidate_sum | heap_rerank_sum | p50 ms | p95 ms | p99 ms | max ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| packet 025 repeat | 0.9876 | 9,213,846 | 12,500 | 222.140 | 275.753 | 288.894 | 296.358 |
| packet 027 repeat | 0.9876 | 9,213,846 | 12,500 | 228.595 | 284.140 | 295.823 | 301.065 |

The TID ordering sublever therefore does not satisfy the retained-recall
latency bar and should not be carried forward as an optimization.

## Evidence

- Suite config:
  `reviews/task-85/027-aws-local-heap-fetch-order/suite-aws-1m-local-heap-fetch-order-q500.json`
- Suite report:
  `reviews/task-85/027-aws-local-heap-fetch-order/artifacts/aws-1m-local-heap-fetch-order-q500/suite-report.md`
- Repeat locality summary:
  `reviews/task-85/027-aws-local-heap-fetch-order/artifacts/aws-1m-local-heap-fetch-order-q500/local-heap-fetch-order-repeat-summary.json`
- Initial failed suite log:
  `reviews/task-85/027-aws-local-heap-fetch-order/artifacts/cloud-bench-local-heap-fetch-order-q500.log`
- Successful rerun log:
  `reviews/task-85/027-aws-local-heap-fetch-order/artifacts/cloud-bench-local-heap-fetch-order-q500-rerun.log`
- Final AWS pause status:
  `reviews/task-85/027-aws-local-heap-fetch-order/artifacts/cloud-status-final-after-local-heap-fetch-order-q500-paused.log`

## Notes

The first suite invocation failed before measurement because it attempted to
drop/recreate the diagnostic function after packet 025 had attached that
function to the installed extension. The checked-in rerun suite removes that
redundant raw apply step and uses the already-registered diagnostic surface.

The repeat funnel summary shows the same rerank prefix shape as packet 025:
`12,500` rerank rows over `500` queries, unique heap blocks p50/p95/max
`22/25/25`, and adjacent heap-block transitions p50/p95/max `24/24/24`.
The preserved locality metrics plus worse end-to-end latency indicate that
sorting this internal local heap fetch loop by TID is not enough to improve the
current AM scan path.

AWS `1m` was paused after the run; final status records `state: paused`.
