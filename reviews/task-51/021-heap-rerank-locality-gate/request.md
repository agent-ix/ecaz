# Review Request: Heap Rerank Locality Gate

- task: `plan/tasks/51-ivf-rabitq-second-optimization-round.md`
- packet: `reviews/task-51/021-heap-rerank-locality-gate/`
- branch: `aws-optimization-ivf-rabitq-spire`
- code change: none

## Scope

This closes the Experiment 4 decision gate without new product code.

Task 51 says heap-rerank locality work should proceed only if counters show exact rerank or heap fetch dominates. The current scan path already sorts the heap rerank frontier by heap TID before exact vector fetch, prefetches unique heap blocks, then restores final score ranking. The measured counters do not justify more locality work in this round.

## Evidence

Local 990k packet:

- `benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/explain-static-990k-rabitq1-n1024-w50-p128.log`

AWS 1M packet artifacts already present in the final-gate bucket:

- `benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/explain-1m-rabitq1-rerank-p128.log`
- `benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/explain-1m-rabitq1-rerank-p256.log`

Counter summary:

| lane | nprobe | execution | approx scan | exact rerank | rerank rows | heap blocks | exact / execution |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| local 990k | 128 | 622.097 ms | 610717 us | 3742 us | 50 | 48 | 0.6% |
| AWS 1M | 128 | 40.877 ms | 36774 us | 945 us | 50 | 48 | 2.3% |
| AWS 1M | 256 | 77.837 ms | 73649 us | 454 us | 50 | 48 | 0.6% |

## Conclusion

Exact heap rerank is not the current bottleneck for the Task 51 IVF/RaBitQ shape. A further heap-rerank-locality implementation cannot plausibly meet the Experiment 4 promotion threshold of 15% p50 improvement or clear tail reduction on the measured high-recall cells.

Recommendation: do not spend more local or AWS time on Experiment 4 in this round. Keep the existing TID-sorted heap-rerank behavior and focus final-gate AWS time on current-head IVF/RaBitQ and the narrow sidecar microbench requested by packet 016 feedback.

## Artifacts

- `reviews/task-51/021-heap-rerank-locality-gate/artifacts/manifest.md`
