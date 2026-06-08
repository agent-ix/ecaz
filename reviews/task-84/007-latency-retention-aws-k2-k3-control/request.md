# Task 84 Review Request: Latency-Retention k2/k3 Control

## Summary

This packet corrects the Task 84 interpretation against the user's actual goal:
**improve AWS 1M/q500 latency while retaining the current recall point**, not
recover additional recall.

It runs a paired same-suite AWS 1M/q500 control:

1. retained k2 `global1152`;
2. k3 `global1152`;
3. k2 route-prior `0.10`;
4. retained k2 `global1152` repeat.

All rows preserve:

- `recall@10=0.9832`;
- `heap_rerank_sum=12,500`;
- miss split `4916/3/81`;
- candidate surface around `9.21M`.

## Result

| row | recall@10 | candidate_sum | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: |
| k2 first | 0.9832 | 9,213,846 | 274.607 ms | 340.723 ms | 354.243 ms |
| k3 | 0.9832 | 9,213,742 | 257.047 ms | 319.000 ms | 336.771 ms |
| k2 route-prior 0.10 | 0.9832 | 9,213,619 | 254.764 ms | 317.188 ms | 332.755 ms |
| k2 repeat | 0.9832 | 9,213,846 | 255.571 ms | 314.469 ms | 331.985 ms |

## Interpretation

The earlier k3 result looked like a same-recall latency win because it was
compared to older retained k2 rows around `282-289 ms` p50. This paired run
shows that conclusion was too weak:

- k3 beats the first k2 row, but not the warmed k2 repeat;
- route-prior `0.10` is only `0.807 ms` faster than k2 repeat at p50 and has
  worse p95/p99 than k2 repeat;
- all rows keep the same recall and miss split.

So k3 should not be promoted as a latency optimization from the current
evidence. Route-prior also should not be accepted as a meaningful latency
policy.

The corrected result is still important: current warmed k2 can run around
`255 ms` p50 at the retained `0.9832` recall point, which is much faster than
some earlier diagnostic/baseline rows. Future latency work needs standardized
warmup and paired-order controls before claiming small deltas.

## Artifacts

- `artifacts/manifest.md`
- `artifacts/latency-retention-summary.tsv`
- `artifacts/aws-1m-latency-retention-k2-k3-q500/results.jsonl`
- `artifacts/aws-1m-latency-retention-k2-k3-q500/suite-report.md`
- `artifacts/cloud-status-after-latency-retention-pause.log`

AWS `1m` was paused after the run.
