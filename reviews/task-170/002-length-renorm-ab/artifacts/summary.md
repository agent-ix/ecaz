# Task 170 Length Renorm A/B Summary

- Baseline: `9bc66bcabe22697b4edc91300914b1e692938c44`
- Renorm-fixed: `9ded201453cb851076f54c1b787d69f6519b0578`
- Suite config: `reviews/task-170/002-length-renorm-ab/task148-length-renorm-suite.json`

## Verdict

Length renormalization is a measured negative for the 4-bit no-QJL pure TQ cell. It improves 100k recall by about 0.62-0.63 pp, but query latency regresses by about 5-6x at the required nprobe 32/40 checks. The 10k and 50k recall grids are unchanged.

The stage2@25 cell is effectively unchanged. The persisted dense-block sidecar does not carry gamma, so the code now safely treats sidecar scoring as unrenormalized instead of forcing an on-disk format change. Its recall deltas are 0.00 pp across the measured grid and latency remains neutral.

No 1m suite was run because the 100k gate did not show a viable latency-neutral win, and stage2 cannot receive the correction without a format decision.

## tqdefault recall@10

| scale | nprobe | baseline | renorm | delta pp | baseline q ms | renorm q ms |
|---|---:|---:|---:|---:|---:|---:|
| 10k | 8 | 96.25% | 96.25% | 0.00 | 0.37 | 1.29 |
| 10k | 16 | 97.19% | 97.19% | 0.00 | 0.34 | 1.63 |
| 10k | 24 | 97.50% | 97.50% | 0.00 | 0.42 | 2.22 |
| 10k | 32 | 97.50% | 97.50% | 0.00 | 0.50 | 2.92 |
| 10k | 48 | 97.50% | 97.50% | 0.00 | 0.67 | 4.36 |
| 10k | 64 | 97.50% | 97.50% | 0.00 | 0.83 | 5.83 |
| 50k | 8 | 91.87% | 91.87% | 0.00 | 1.10 | 2.82 |
| 50k | 16 | 95.00% | 95.00% | 0.00 | 0.74 | 3.38 |
| 50k | 24 | 95.31% | 95.31% | 0.00 | 1.01 | 4.93 |
| 50k | 32 | 95.94% | 95.94% | 0.00 | 1.00 | 6.75 |
| 50k | 48 | 95.94% | 95.94% | 0.00 | 1.32 | 9.73 |
| 50k | 64 | 95.94% | 95.94% | 0.00 | 1.70 | 12.66 |
| 100k | 8 | 78.44% | 79.06% | 0.62 | 1.53 | 3.66 |
| 100k | 16 | 83.44% | 84.06% | 0.62 | 1.23 | 5.01 |
| 100k | 24 | 87.50% | 88.12% | 0.62 | 1.25 | 7.31 |
| 100k | 32 | 89.38% | 90.00% | 0.62 | 1.42 | 9.49 |
| 100k | 48 | 91.25% | 91.87% | 0.62 | 1.91 | 14.03 |
| 100k | 64 | 92.50% | 93.13% | 0.63 | 2.44 | 18.41 |

## tqdefault latency

| scale | nprobe | baseline mean ms | renorm mean ms | delta % | baseline p95 ms | renorm p95 ms |
|---|---:|---:|---:|---:|---:|---:|
| 10k | 32 | 0.59 | 3.00 | 408.5 | 0.71 | 3.36 |
| 10k | 40 | 0.66 | 3.67 | 456.1 | 0.76 | 4.09 |
| 50k | 32 | 1.06 | 6.67 | 529.2 | 1.18 | 7.31 |
| 50k | 40 | 1.22 | 8.14 | 567.2 | 1.37 | 8.68 |
| 100k | 32 | 1.66 | 9.92 | 497.6 | 2.06 | 11.90 |
| 100k | 40 | 1.85 | 11.80 | 537.8 | 2.21 | 13.00 |

## tqdefault storage

| scale | baseline total/row B | renorm total/row B | delta B/row | baseline index/row B | renorm index/row B |
|---|---:|---:|---:|---:|---:|
| 10k | 17615 | 17617 | 2 | 24.6 | 24.6 |
| 50k | 17541 | 17542 | 1 | 22.8 | 22.8 |
| 100k | 17525 | 17526 | 1 | 22.6 | 22.6 |

## stage2 recall@10

| scale | nprobe | baseline | renorm | delta pp | baseline q ms | renorm q ms |
|---|---:|---:|---:|---:|---:|---:|
| 10k | 8 | 98.12% | 98.12% | 0.00 | 0.52 | 1.71 |
| 10k | 16 | 99.38% | 99.38% | 0.00 | 0.42 | 0.54 |
| 10k | 24 | 100.00% | 100.00% | 0.00 | 0.51 | 0.57 |
| 10k | 32 | 100.00% | 100.00% | 0.00 | 0.61 | 0.56 |
| 10k | 48 | 100.00% | 100.00% | 0.00 | 0.72 | 0.67 |
| 10k | 64 | 100.00% | 100.00% | 0.00 | 0.85 | 0.81 |
| 50k | 8 | 94.37% | 94.37% | 0.00 | 1.38 | 1.33 |
| 50k | 16 | 98.12% | 98.12% | 0.00 | 0.88 | 0.85 |
| 50k | 24 | 98.75% | 98.75% | 0.00 | 1.00 | 1.01 |
| 50k | 32 | 99.38% | 99.38% | 0.00 | 1.03 | 1.01 |
| 50k | 48 | 99.38% | 99.38% | 0.00 | 1.34 | 1.28 |
| 50k | 64 | 99.38% | 99.38% | 0.00 | 1.65 | 1.62 |
| 100k | 8 | 81.56% | 81.56% | 0.00 | 1.75 | 1.71 |
| 100k | 16 | 87.50% | 87.50% | 0.00 | 1.14 | 1.22 |
| 100k | 24 | 91.87% | 91.87% | 0.00 | 1.18 | 1.18 |
| 100k | 32 | 93.75% | 93.75% | 0.00 | 1.38 | 1.34 |
| 100k | 48 | 95.63% | 95.63% | 0.00 | 1.83 | 1.78 |
| 100k | 64 | 97.19% | 97.19% | 0.00 | 2.30 | 2.21 |

## stage2 latency

| scale | nprobe | baseline mean ms | renorm mean ms | delta % | baseline p95 ms | renorm p95 ms |
|---|---:|---:|---:|---:|---:|---:|
| 10k | 32 | 0.65 | 0.65 | 0.0 | 0.81 | 0.74 |
| 10k | 40 | 0.70 | 0.68 | -2.9 | 0.81 | 0.78 |
| 50k | 32 | 1.10 | 1.08 | -1.8 | 1.20 | 1.19 |
| 50k | 40 | 1.23 | 1.21 | -1.6 | 1.38 | 1.29 |
| 100k | 32 | 1.55 | 1.47 | -5.2 | 1.76 | 1.64 |
| 100k | 40 | 1.75 | 1.68 | -4.0 | 2.02 | 1.84 |

## stage2 storage

| scale | baseline total/row B | renorm total/row B | delta B/row | baseline index/row B | renorm index/row B |
|---|---:|---:|---:|---:|---:|
| 10k | 17876 | 17877 | 1 | 24.6 | 24.6 |
| 50k | 17785 | 17786 | 1 | 22.8 | 22.8 |
| 100k | 17765 | 17766 | 1 | 22.6 | 22.6 |

## Block Kernel Timing

| variant | scale | nprobe | quant | baseline ms | renorm ms | delta % | baseline candidates | renorm candidates |
|---|---|---:|---|---:|---:|---:|---:|---:|
| tqdefault | 10k | 32 | turboquant_int8 | 3.854 | 3.898 | 1.1 | 99207 | 99207 |
| tqdefault | 10k | 40 | turboquant_int8 | 4.864 | 4.804 | -1.2 | 123870 | 123870 |
| tqdefault | 50k | 32 | turboquant_int8 | 8.728 | 8.792 | 0.7 | 225031 | 225031 |
| tqdefault | 50k | 40 | turboquant_int8 | 10.928 | 10.830 | -0.9 | 281405 | 281405 |
| tqdefault | 100k | 32 | turboquant_int8 | 13.276 | 13.187 | -0.7 | 331757 | 331757 |
| tqdefault | 100k | 40 | turboquant_int8 | 16.238 | 15.920 | -2.0 | 411901 | 411901 |
| stage2 | 10k | 32 | rabitq | 6.422 | 6.476 | 0.8 | 99207 | 99207 |
| stage2 | 10k | 32 | turboquant_int8 | 0.065 | 0.064 | -0.6 | 1600 | 1600 |
| stage2 | 10k | 40 | rabitq | 7.946 | 7.914 | -0.4 | 123870 | 123870 |
| stage2 | 10k | 40 | turboquant_int8 | 0.064 | 0.063 | -0.4 | 1600 | 1600 |
| stage2 | 50k | 32 | rabitq | 14.340 | 14.442 | 0.7 | 225031 | 225031 |
| stage2 | 50k | 32 | turboquant_int8 | 0.066 | 0.066 | -0.1 | 1600 | 1600 |
| stage2 | 50k | 40 | rabitq | 17.853 | 17.942 | 0.5 | 281405 | 281405 |
| stage2 | 50k | 40 | turboquant_int8 | 0.065 | 0.066 | 1.8 | 1600 | 1600 |
| stage2 | 100k | 32 | rabitq | 21.285 | 21.018 | -1.3 | 331757 | 331757 |
| stage2 | 100k | 32 | turboquant_int8 | 0.066 | 0.065 | -1.6 | 1600 | 1600 |
| stage2 | 100k | 40 | rabitq | 26.609 | 26.228 | -1.4 | 411901 | 411901 |
| stage2 | 100k | 40 | turboquant_int8 | 0.066 | 0.064 | -2.5 | 1600 | 1600 |
