# Task 79 Review Request: RaBitQ Summary-Prior Sampled Global Benchmark

## Summary

This packet benchmarks the packet 019 summary-prior sampled global block scoring implementation on the Task 79 RaBitQ surface. It is negative evidence: summary-prior sampling does not recover recall at the desired 2M-5.2M candidate surface, and it adds enough scoring overhead that the sampled rows are slower than summary-only global pruning.

All runs use `ecaz bench suite` with the checked-in config:

- `reviews/task-79/020-rabitq-summary-prior-sampled-global-benchmark/suite-rabitq-summary-prior-sampled-global.json`

The suite completed cleanly:

- `artifacts/suite-status.log`: 13 completed, 0 failed, 0 skipped, 0 missing artifacts
- `artifacts/manifest.md`: full command/artifact/result manifest

## Results

| row | candidates | p50 | recall@10 | returned |
| --- | ---: | ---: | ---: | ---: |
| baseline global0 | 15506227 | 63.798 ms | 0.9975 | 2000 |
| summary-only global384 | 4684566 | 46.625 ms | 0.9675 | 2000 |
| summary-only global512 | 6269044 | 49.141 ms | 0.9860 | 2000 |
| sampled global320/probe768/sample2/prior0.8 | 4202021 | 46.876 ms | 0.9545 | 1933 |
| sampled global384/probe768/sample1/prior0.8 | 4837901 | 49.095 ms | 0.9725 | 1964 |
| sampled global384/probe1024/sample1/prior0.7 | 4889325 | 49.315 ms | 0.9690 | 1964 |
| sampled global384/probe1024/sample1/prior0.8 | 4889058 | 49.640 ms | 0.9725 | 1964 |
| sampled global384/probe1024/sample1/prior0.9 | 4888223 | 49.189 ms | 0.9695 | 1964 |
| sampled global384/probe1024/sample2/prior0.8 | 5093512 | 50.506 ms | 0.9670 | 1932 |
| sampled global384/probe1536/sample1/prior0.8 | 4935235 | 50.237 ms | 0.9730 | 1963 |
| sampled global400/probe1024/sample1/prior0.8 | 5085472 | 50.310 ms | 0.9740 | 1964 |

## Readout

No row clears the Task 79 gates. The candidate gate is 5.2M, the recall gate is 0.9925, and the baseline p50 in this run is 63.798 ms, so a 25 percent p50 improvement would require p50 at or below 47.849 ms.

The summary-only controls reproduce the known curve: global384 cuts the candidate surface to 4.685M and reaches p50 46.625 ms, but recall is only 0.9675. Global512 improves recall to 0.9860, but scans 6.269M candidates and has p50 49.141 ms.

The sampled-prior rows do not fix the recall loss. The best sampled recall is 0.9740 at 5.085M candidates and p50 50.310 ms. Several sampled rows also under-return (`returned` 1932-1964), which makes them worse than the summary-only rows at the same general candidate budget.

## Recommendation

Do not pursue summary-prior sampled global block scoring as the Task 79 latency fix.

After processing the latest reviewer feedback, the next packet should test the rerank-width axis on the existing summary-only global selector before any multi-representative selector work:

- block64 RaBitQ, global384/global512
- `rerank_width` sweep at 25/50/100/200/500
- same candidate, latency, recall, and returned-row gates

That will distinguish whether the remaining recall failure is actually narrow rerank under a smaller candidate surface, or whether the one-summary-per-block selector is the binding constraint.
