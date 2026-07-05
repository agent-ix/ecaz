# Task 86 SPIRE TurboQuant Real-Spread Benchmark Delta

## Summary

This packet compares pre-LUT SPIRE TurboQuant against the current SPIRE TurboQuant LUT scorer on the normal real10k/real50k/real100k spread.

- Index/profile: `ec_spire`
- Storage format: `turboquant`
- Bits/seed: `4` / `42`
- Query limit: `200`
- Latency iterations: `1000`
- Suites: `suite-lutoff.json` and `suite-luton.json`
- Baseline source: pre-LUT worktree at `eda36f088dfafc1c3c379de7f3e0cfac888fae06`
- Current source: branch source at `c200632f5835b3a0cd08938f3e9cdff5b836a8f9`; packet head `0d93ef0de47bf10a2aa455204e4e0b97ce89be54` only adds reviewer feedback over that source.

Result: recall and storage are unchanged. Query latency improves modestly but consistently across all nine sweep points.

## SQL Latency And Pipeline Query Metrics

Negative delta means faster.

| Corpus | nprobe | recall@10 baseline -> after | SQL mean baseline -> after | SQL delta | Pipeline p50 baseline -> after | Pipeline delta |
| --- | ---: | --- | ---: | ---: | ---: | ---: |
| real10k | 8 | 0.9985 -> 0.9985 | 3.44 -> 3.30 ms | -4.1% | 3.549 -> 3.406 ms | -4.0% |
| real10k | 24 | 1.0000 -> 1.0000 | 8.02 -> 7.69 ms | -4.1% | 8.089 -> 7.675 ms | -5.1% |
| real10k | 32 | 1.0000 -> 1.0000 | 10.2 -> 9.74 ms | -4.5% | 10.283 -> 9.711 ms | -5.6% |
| real50k | 16 | 0.9660 -> 0.9660 | 12.3 -> 12.0 ms | -2.4% | 12.660 -> 11.938 ms | -5.7% |
| real50k | 48 | 0.9975 -> 0.9975 | 33.9 -> 32.9 ms | -2.9% | 33.779 -> 32.299 ms | -4.4% |
| real50k | 64 | 1.0000 -> 1.0000 | 48.0 -> 46.1 ms | -4.0% | 48.192 -> 46.042 ms | -4.5% |
| real100k | 32 | 0.9600 -> 0.9600 | 25.3 -> 24.5 ms | -3.2% | 25.646 -> 24.670 ms | -3.8% |
| real100k | 96 | 0.9980 -> 0.9980 | 74.1 -> 71.7 ms | -4.0% | 74.584 -> 72.274 ms | -4.0% |
| real100k | 128 | 1.0000 -> 1.0000 | 95.3 -> 92.3 ms | -3.9% | 95.084 -> 92.184 ms | -5.2% |

## SQL Latency Tails

| Corpus | nprobe | SQL p50 baseline -> after | SQL p95 baseline -> after | SQL p99 baseline -> after |
| --- | ---: | ---: | ---: | ---: |
| real10k | 8 | 3.50 -> 3.37 ms | 3.78 -> 3.57 ms | 4.08 -> 3.64 ms |
| real10k | 24 | 8.01 -> 7.66 ms | 8.39 -> 8.09 ms | 8.61 -> 8.52 ms |
| real10k | 32 | 10.1 -> 9.71 ms | 10.4 -> 9.96 ms | 10.9 -> 10.3 ms |
| real50k | 16 | 12.3 -> 11.9 ms | 13.7 -> 13.3 ms | 14.1 -> 13.7 ms |
| real50k | 48 | 33.7 -> 32.7 ms | 36.3 -> 35.4 ms | 38.0 -> 36.7 ms |
| real50k | 64 | 47.9 -> 46.0 ms | 48.5 -> 46.6 ms | 50.1 -> 48.3 ms |
| real100k | 32 | 25.3 -> 24.5 ms | 27.6 -> 26.7 ms | 28.7 -> 27.8 ms |
| real100k | 96 | 74.0 -> 71.8 ms | 77.0 -> 74.4 ms | 78.6 -> 75.7 ms |
| real100k | 128 | 95.2 -> 92.2 ms | 96.0 -> 93.0 ms | 98.4 -> 94.7 ms |

## Pipeline Latency Tails

| Corpus | nprobe | Pipeline p50 baseline -> after | Pipeline p95 baseline -> after | Pipeline p99 baseline -> after |
| --- | ---: | ---: | ---: | ---: |
| real10k | 8 | 3.549 -> 3.406 ms | 3.767 -> 3.601 ms | 3.907 -> 3.732 ms |
| real10k | 24 | 8.089 -> 7.675 ms | 8.440 -> 8.038 ms | 8.649 -> 8.206 ms |
| real10k | 32 | 10.283 -> 9.711 ms | 10.717 -> 9.909 ms | 10.844 -> 9.997 ms |
| real50k | 16 | 12.660 -> 11.938 ms | 13.933 -> 13.156 ms | 14.169 -> 13.367 ms |
| real50k | 48 | 33.779 -> 32.299 ms | 36.289 -> 34.625 ms | 37.368 -> 35.757 ms |
| real50k | 64 | 48.192 -> 46.042 ms | 48.659 -> 46.616 ms | 50.154 -> 47.437 ms |
| real100k | 32 | 25.646 -> 24.670 ms | 28.270 -> 27.196 ms | 29.383 -> 28.483 ms |
| real100k | 96 | 74.584 -> 72.274 ms | 77.602 -> 75.026 ms | 79.099 -> 76.258 ms |
| real100k | 128 | 95.084 -> 92.184 ms | 96.026 -> 92.743 ms | 100.015 -> 93.718 ms |

## Recall Timing

Recall-stage mean query time also improved while recall stayed identical.

| Corpus | nprobe | baseline mean q-time | after mean q-time |
| --- | ---: | ---: | ---: |
| real10k | 8 | 3.48 ms | 3.35 ms |
| real10k | 24 | 8.01 ms | 7.65 ms |
| real10k | 32 | 10.13 ms | 9.68 ms |
| real50k | 16 | 12.55 ms | 12.17 ms |
| real50k | 48 | 33.67 ms | 32.55 ms |
| real50k | 64 | 47.72 ms | 47.32 ms |
| real100k | 32 | 26.02 ms | 25.21 ms |
| real100k | 96 | 74.68 ms | 72.59 ms |
| real100k | 128 | 94.98 ms | 92.73 ms |

## Storage

The LUT scorer changes query scoring only; vector encoding and index storage are unchanged.

| Corpus | baseline SPIRE index | after SPIRE index | baseline per row | after per row |
| --- | ---: | ---: | ---: | ---: |
| real10k | 8.2 MiB | 8.2 MiB | 857.7 B | 857.7 B |
| real50k | 39.8 MiB | 39.8 MiB | 834.1 B | 834.1 B |
| real100k | 79.5 MiB | 79.5 MiB | 833.9 B | 833.9 B |

Total table+index storage also stayed unchanged within reported precision:

| Corpus | baseline total | after total | baseline per row total | after per row total |
| --- | ---: | ---: | ---: | ---: |
| real10k | 167.2 MiB | 167.2 MiB | 17535.8 B | 17535.8 B |
| real50k | 834.7 MiB | 834.7 MiB | 17504.2 B | 17504.2 B |
| real100k | 1.6 GiB | 1.6 GiB | 17502.9 B | 17503.0 B |

## Interpretation

The SPIRE LUT change is a measured production improvement for our TurboQuant implementation. It does not introduce TurboVec's calibrated encoding, and it does not reduce vector/index size. It closes a local gap where SPIRE was not using the same no-QJL 4-bit dim-LUT scoring path already available elsewhere.

This should be treated as a low-risk no-format-change improvement. The stronger TurboVec-derived idea, calibration-only TQ+, remains unproven until it gets its own real-corpus recall/latency/storage suite.
