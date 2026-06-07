# Task 85 Review Request: AWS 1M Block8 Geometry Result

## Summary

This packet tests the Task85 candidate-density hypothesis at AWS 1M/q500 by
building a separate `leaf_block_rows=8` SPIRE index and measuring matched-recall
latency against the retained Task79/Task85 block16 warm floor.

Verdict: reject block8 as a Task85 latency improvement. It reaches the retained
recall point with about half the candidates, but latency is worse at p50 and
p95. Candidate reduction alone did not move the product-scale latency target.

## Baseline

Retained Task85 warm floor from packet 001:

| Recall@10 | p50 | p95 | p99 | Candidate Sum |
| ---: | ---: | ---: | ---: | ---: |
| 0.9832 | 246.397 ms | 304.476 ms | 321.342 ms | 9,213,846 |

## Block8 Results

`reviews/task-85/004-aws-1m-block8-geometry/artifacts/aws-1m-block8-geometry-q500/suite-report.md`

| Row | Recall@10 | p50 | p95 | p99 | Candidate Sum | Verdict |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| block8 global1152 first | 0.9832 | 283.839 ms | 357.274 ms | 2729.302 ms | 4,607,442 | same recall, fewer candidates, slower |
| block8 global1536 | 0.9876 | 283.900 ms | 340.606 ms | 350.005 ms | 6,143,277 | higher recall, slower |
| block8 global2048 | 0.9914 | 308.561 ms | 355.499 ms | 361.867 ms | 8,191,063 | higher recall, slower |
| block8 global2304 | 0.9926 | 320.404 ms | 362.190 ms | 371.869 ms | 9,214,933 | higher recall, slower and no candidate win |
| block8 global2048 repeat | 0.9914 | 307.925 ms | 355.430 ms | 362.980 ms | 8,191,063 | warm repeat still slower |

The matched-recall comparison is the important row: block8/global1152 preserved
`recall@10=0.9832` and reduced candidates from `9,213,846` to `4,607,442`, but
latency regressed from `246.397/304.476/321.342 ms` to
`283.839/357.274/2729.302 ms` at p50/p95/p99.

## Operator Notes

The first AWS run failed during block8 index build because the 100 GiB data
volume was full:

`reviews/task-85/004-aws-1m-block8-geometry/artifacts/aws-1m-block8-geometry-q500-nospace-failed/build-spire-1m-rabitq-block8-tg256.log`

The instance was paused after the failed attempt, resumed for inspection, paused
again, then the data EBS volume was expanded from 100 GiB to 150 GiB and XFS was
grown. The suite was rerun successfully after that expansion.

Final AWS state after the successful run:

`reviews/task-85/004-aws-1m-block8-geometry/artifacts/cloud-status-final-paused-after-block8-success.log`

## Validation

- `ecaz bench suite audit`: passed for 8 steps.
- Failed first AWS run artifacts preserved under
  `artifacts/aws-1m-block8-geometry-q500-nospace-failed/`.
- Successful AWS suite: completed 8, failed 0, skipped 0.
- AWS profile `1m`: paused after the run.

## Requested Review

Please review the block8 rejection as Task85 latency evidence. The key question
is whether this is enough to rule out smaller block geometry as a product-scale
latency path and redirect Task85 toward the next mechanism.
