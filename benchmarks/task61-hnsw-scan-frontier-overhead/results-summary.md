# Task 61 HNSW Scan Frontier Overhead Summary

Code commit `7928649b0` improved the measured 10k/50k HNSW scan latency on the
Graviton `10k-medium` host without changing recall.

| Scale | ef_search | Baseline p50 / p95 / p99 | Optimized p50 / p95 / p99 |
| --- | ---: | --- | --- |
| 10k | 40 | 1.02 / 1.54 / 1.84 ms | 0.93 / 1.31 / 1.59 ms |
| 10k | 200 | 3.54 / 4.35 / 6.32 ms | 3.11 / 3.75 / 4.87 ms |
| 50k | 40 | 1.31 / 1.73 / 2.54 ms | 1.23 / 1.54 / 2.37 ms |
| 50k | 200 | 4.35 / 5.20 / 5.44 ms | 3.99 / 4.76 / 5.08 ms |

100k could not be remeasured on this retained host because the 100 GiB data
volume ran out of space during corpus encoding. After cleanup, only `4.2 GiB`
was free, while the 100k staging directory alone had used `3.2 GiB`.

The cloud host was paused after cleanup; final status reported `state: paused`
and `$0.00/hr` running cost.
