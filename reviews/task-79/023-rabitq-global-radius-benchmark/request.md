# Task 79 Review Request: RaBitQ Global Radius Benchmark

## Summary

This packet benchmarks packet 022's RaBitQ radius-bound global block scoring fix. It is negative evidence: the code slice is mechanically consistent with per-leaf scoring, but the raw radius bound is too loose for global ranking and drops recall well below the Task 79 gate.

All runs use `ecaz bench suite` with the checked-in config:

- `reviews/task-79/023-rabitq-global-radius-benchmark/suite-rabitq-global-radius.json`

The suite completed cleanly:

- `artifacts/suite-status.log`: 7 completed, 0 failed, 0 skipped, 0 missing artifacts
- `artifacts/manifest.md`: full command/artifact/result manifest
- `artifacts/compact-results.tsv`: compact result table

## Results

| row | candidates | p50 | recall@10 | returned |
| --- | ---: | ---: | ---: | ---: |
| baseline global0 | 15506227 | 62.200 ms | 0.9975 | 2000 |
| radius global384 | 4798334 | 43.686 ms | 0.9310 | 2000 |
| radius global400 | 4997529 | 44.131 ms | 0.9355 | 2000 |
| radius global416 | 5197484 | 46.195 ms | 0.9385 | 2000 |
| radius global512 | 6394228 | 47.938 ms | 0.9565 | 2000 |

## Readout

No row clears the Task 79 gates. The candidate gate is 5.2M, the recall gate is 0.9925, and the baseline p50 in this run is 62.200 ms, so a 25 percent p50 improvement would require p50 at or below 46.650 ms.

The radius-bound selector is worse than packet 021's summary-only selector:

- `global384`: summary-only was 0.9675 recall at 4.685M candidates; radius-bound is 0.9310 at 4.798M candidates.
- `global512`: summary-only was 0.9860 recall at 6.269M candidates; radius-bound is 0.9565 at 6.394M candidates.

The best candidate-gate row is `global416`, but it reaches only 0.9385 recall at 5.197M candidates.

## Recommendation

Do not pursue raw radius-bound global block scoring as the Task 79 latency fix.

The implementation exposed a real consistency issue, but the benchmark shows the bound is not discriminating enough for global block admission. The next implementation slice should move to richer selector information, likely multi-representative block summaries or another admission rule that preserves high-recall blocks before enforcing the global cap.
