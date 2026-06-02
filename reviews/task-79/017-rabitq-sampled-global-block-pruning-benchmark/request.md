# Task 79 Review Request: RaBitQ Sampled Global Block Pruning Benchmark

## Summary

This packet benchmarks the packet 016 sampled global block pruning path on the Task 79 100k RaBitQ surface. It is negative evidence: sampled row reranking reduces candidates mechanically, but it does not preserve recall and does not produce a valid latency win.

All runs use `ecaz bench suite` with the checked-in config:

- `reviews/task-79/017-rabitq-sampled-global-block-pruning-benchmark/suite-rabitq-sampled-global-block-pruning.json`

The suite completed cleanly:

- `artifacts/suite-status.log`: 10 completed, 0 failed, 0 skipped, 0 missing artifacts
- `artifacts/manifest.md`: full command/artifact/result manifest

## Results

| row | global final blocks | probe blocks | samples/block | candidates | p50 | recall@10 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| baseline | 0 | 0 | 0 | 15506227 | 61.752 ms | 0.9975 |
| summary-only | 384 | 0 | 0 | 4684566 | 43.281 ms | 0.9675 |
| sampled | 384 | 768 | 1 | 4838410 | 48.622 ms | 0.8605 |
| sampled | 384 | 1024 | 1 | 4894338 | 47.898 ms | 0.8425 |
| sampled | 384 | 1536 | 1 | 4943961 | 48.383 ms | 0.8335 |
| sampled | 400 | 1024 | 1 | 5092327 | 49.449 ms | 0.8495 |
| sampled | 384 | 1024 | 2 | 5105439 | 49.089 ms | 0.8985 |
| sampled | 512 | 1024 | 1 | 6480189 | 52.387 ms | 0.8905 |

## Readout

Sampling does not rescue the global block selector as implemented. The sample rows are honestly counted, so the candidate totals are comparable to earlier packets, but the row-sample score is too noisy to replace the summary score. It also under-returns in several sampled rows (`returned_sum` below 2000 in the production read profile), which is another symptom of bad block selection.

The best sampled row here is `global384/probe1024/sample2`: 5.105M candidates, 49.089 ms p50, 0.8985 recall@10. That misses the Task 79 recall gate by about 9.4 percentage points. Summary-only global384 is better on recall and latency, though it also misses the recall gate.

## Recommendation

Do not treat packet 016 sampling as the latency solution. Keep it disabled-by-default as diagnostic scaffolding only if useful, or remove it before merge if we do not want a dead tuning surface.

The next Task 79 implementation should either:

- preserve the summary score as a prior and let samples adjust it instead of replacing it, or
- implement richer per-block summaries/multi-representatives so block pruning has a better score than one mean vector or one/two sampled rows.
