# Task 79 Review Request: RaBitQ Block32 Global Block Pruning Benchmark

## Summary

This packet benchmarks whether rebuilding the Task 79 RaBitQ surface with `leaf_block_rows=32` makes summary-only global block pruning viable. It is negative evidence: block32 improves the candidate/recall curve shape, but it still does not produce a valid Task 79 latency win.

All runs use `ecaz bench suite` with the checked-in config:

- `reviews/task-79/018-rabitq-block32-global-pruning-benchmark/suite-rabitq-block32-global-pruning.json`

The suite completed cleanly:

- `artifacts/suite-status.log`: 8 completed, 0 failed, 0 skipped, 0 missing artifacts
- `artifacts/manifest.md`: full command/artifact/result manifest

## Results

| row | global final blocks | candidates | p50 | recall@10 |
| --- | ---: | ---: | ---: | ---: |
| baseline | 0 | 15506227 | 62.171 ms | 0.9975 |
| summary-only | 512 | 3181966 | 41.984 ms | 0.9520 |
| summary-only | 768 | 4786471 | 44.758 ms | 0.9730 |
| summary-only | 1024 | 6396498 | 49.467 ms | 0.9845 |
| summary-only | 1280 | 8010285 | 55.394 ms | 0.9920 |
| summary-only | 1536 | 9624957 | 57.598 ms | 0.9960 |

## Readout

This directly addresses the candidate explosion question, but it does not improve SPIRE latency within the required constraints. Rows under the candidate gate miss recall badly. The closest row, `global1280`, reaches 0.9920 recall@10 but still misses the 0.9925 recall gate and scans 8.010M candidates. The first row that clears recall, `global1536`, scans 9.625M candidates and has p50 57.598 ms.

Compared with the original no-prune surface at 15.506M candidates and p50 about 62 ms, these rows prove pruning can reduce work, but summary-only block scoring is not accurate enough to cut candidates to the 2M-5.2M target range while preserving recall.

## Recommendation

Do not pursue block-size-only geometry as the Task 79 fix.

The next implementation should improve the block score itself:

- preserve the summary score as a prior and let sampled rows adjust it instead of replacing it, or
- add richer per-block summaries/multi-representatives so global pruning can identify high-value row ranges before scanning millions of low-value candidates.
