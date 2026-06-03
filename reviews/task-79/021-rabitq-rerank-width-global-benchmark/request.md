# Task 79 Review Request: RaBitQ Rerank-Width Global Benchmark

## Summary

This packet benchmarks the reviewer-requested rerank-width axis on the Task 79 RaBitQ surface. It is negative evidence for rerank-width tuning as the fix: widening exact heap rerank from 25 to 500 does not recover recall for either tested global block cap.

All runs use `ecaz bench suite` with the checked-in config:

- `reviews/task-79/021-rabitq-rerank-width-global-benchmark/suite-rabitq-rerank-width-global.json`

The suite completed cleanly:

- `artifacts/suite-status.log`: 13 completed, 0 failed, 0 skipped, 0 missing artifacts
- `artifacts/manifest.md`: full command/artifact/result manifest
- `artifacts/compact-results.tsv`: compact result table

## Results

| row | candidates | p50 | recall@10 | returned |
| --- | ---: | ---: | ---: | ---: |
| baseline global0/rerank25 | 15506227 | 63.487 ms | 0.9975 | 2000 |
| global384/rerank25 | 4684566 | 45.101 ms | 0.9675 | 2000 |
| global384/rerank50 | 4684566 | 45.715 ms | 0.9675 | 2000 |
| global384/rerank100 | 4684566 | 47.468 ms | 0.9675 | 2000 |
| global384/rerank200 | 4684566 | 51.480 ms | 0.9675 | 2000 |
| global384/rerank500 | 4684566 | 61.987 ms | 0.9675 | 2000 |
| global512/rerank25 | 6269044 | 48.510 ms | 0.9860 | 2000 |
| global512/rerank50 | 6269044 | 51.052 ms | 0.9860 | 2000 |
| global512/rerank100 | 6269044 | 50.749 ms | 0.9860 | 2000 |
| global512/rerank200 | 6269044 | 55.359 ms | 0.9860 | 2000 |
| global512/rerank500 | 6269044 | 67.384 ms | 0.9860 | 2000 |

## Readout

No pruned row clears the Task 79 gates. The candidate gate is 5.2M, the recall gate is 0.9925, and the baseline p50 in this run is 63.487 ms, so a 25 percent p50 improvement would require p50 at or below 47.615 ms.

The important signal is recall flatness:

- `global384` stays at recall@10 0.9675 from rerank25 through rerank500.
- `global512` stays at recall@10 0.9860 from rerank25 through rerank500.

Wider rerank only adds latency. At `global512`, p50 moves from 48.510 ms at rerank25 to 67.384 ms at rerank500. At `global384`, p50 moves from 45.101 ms to 61.987 ms.

## Recommendation

Do not pursue wider `rerank_width` as the Task 79 latency fix.

This packet supports the selector-side conclusion: the summary-only global block cap is dropping true-neighbor-bearing blocks before the heap reranker sees them. The next implementation slice should directly address candidate surface admission, preserving high-recall blocks before enforcing a smaller global cap.
