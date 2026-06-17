# Task 111a All Dense Options Benchmark Summary

Measured head: `c543e7a96`

Suite: `task111a-all-dense-options-benchmark-gate`

Status: 120 completed, 0 failed, 0 skipped, 0 missing artifacts, 0 stale.

## Variants

| variant | reloptions / scan settings |
| --- | --- |
| row | `dense_posting_blocks=0` |
| dense-old | `dense_posting_blocks=1`, `dense_posting_pack_pages=1`, `dense_posting_typed_layout=0`, coalescing off, typed views off |
| dense-a | same durable layout as dense-old, coalescing on, typed views off |
| dense-typed | `dense_posting_typed_layout=1`, coalescing off, typed views on |
| dense-b | `dense_posting_pack_pages=4`, coalescing off, typed views off |
| dense-b-typed | `dense_posting_pack_pages=4`, coalescing off, typed views on |

## Latency

Warm p50/p95/p99 at nprobe 32 and 64.

| scale | quant | variant | nprobe | p50 | p95 | p99 | mean |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| 50k | TQ | row | 32 | 15.0 ms | 16.8 ms | 17.3 ms | 15.2 ms |
| 50k | TQ | row | 64 | 30.4 ms | 34.3 ms | 39.1 ms | 30.8 ms |
| 50k | TQ | dense-old | 32 | 17.7 ms | 19.7 ms | 20.7 ms | 17.8 ms |
| 50k | TQ | dense-old | 64 | 36.1 ms | 38.9 ms | 47.7 ms | 36.7 ms |
| 50k | TQ | dense-a | 32 | 13.1 ms | 14.5 ms | 15.4 ms | 13.3 ms |
| 50k | TQ | dense-a | 64 | 26.9 ms | 32.5 ms | 35.8 ms | 28.0 ms |
| 50k | TQ | dense-typed | 32 | 22.0 ms | 26.4 ms | 30.3 ms | 21.6 ms |
| 50k | TQ | dense-typed | 64 | 37.0 ms | 39.5 ms | 43.4 ms | 37.3 ms |
| 50k | TQ | dense-b | 32 | 14.6 ms | 16.7 ms | 18.4 ms | 14.9 ms |
| 50k | TQ | dense-b | 64 | 28.8 ms | 30.3 ms | 32.1 ms | 29.0 ms |
| 50k | TQ | dense-b-typed | 32 | 13.8 ms | 14.7 ms | 15.7 ms | 13.8 ms |
| 50k | TQ | dense-b-typed | 64 | 28.7 ms | 30.8 ms | 31.6 ms | 29.0 ms |
| 50k | RaBitQ | row | 32 | 7.29 ms | 8.05 ms | 8.36 ms | 7.35 ms |
| 50k | RaBitQ | row | 64 | 15.2 ms | 16.8 ms | 17.7 ms | 15.4 ms |
| 50k | RaBitQ | dense-old | 32 | 6.06 ms | 6.89 ms | 8.19 ms | 6.16 ms |
| 50k | RaBitQ | dense-old | 64 | 12.1 ms | 13.3 ms | 14.9 ms | 12.3 ms |
| 50k | RaBitQ | dense-a | 32 | 6.17 ms | 6.95 ms | 7.25 ms | 6.26 ms |
| 50k | RaBitQ | dense-a | 64 | 12.5 ms | 13.6 ms | 15.4 ms | 12.7 ms |
| 50k | RaBitQ | dense-typed | 32 | 5.99 ms | 6.85 ms | 7.08 ms | 6.03 ms |
| 50k | RaBitQ | dense-typed | 64 | 12.0 ms | 12.8 ms | 13.3 ms | 12.1 ms |
| 50k | RaBitQ | dense-b | 32 | 7.14 ms | 7.95 ms | 9.00 ms | 7.22 ms |
| 50k | RaBitQ | dense-b | 64 | 13.2 ms | 14.3 ms | 16.3 ms | 13.4 ms |
| 50k | RaBitQ | dense-b-typed | 32 | 7.31 ms | 8.26 ms | 8.73 ms | 7.38 ms |
| 50k | RaBitQ | dense-b-typed | 64 | 13.1 ms | 16.9 ms | 18.5 ms | 13.6 ms |
| 100k | TQ | row | 32 | 32.4 ms | 36.4 ms | 38.2 ms | 32.4 ms |
| 100k | TQ | row | 64 | 60.2 ms | 65.1 ms | 66.9 ms | 60.8 ms |
| 100k | TQ | dense-old | 32 | 38.4 ms | 42.9 ms | 44.0 ms | 38.2 ms |
| 100k | TQ | dense-old | 64 | 75.2 ms | 87.8 ms | 89.2 ms | 76.7 ms |
| 100k | TQ | dense-a | 32 | 28.2 ms | 31.8 ms | 33.2 ms | 28.2 ms |
| 100k | TQ | dense-a | 64 | 53.1 ms | 70.2 ms | 72.9 ms | 55.4 ms |
| 100k | TQ | dense-typed | 32 | 37.7 ms | 42.3 ms | 44.3 ms | 37.8 ms |
| 100k | TQ | dense-typed | 64 | 71.8 ms | 77.2 ms | 82.4 ms | 72.7 ms |
| 100k | TQ | dense-b | 32 | 29.0 ms | 32.4 ms | 33.3 ms | 28.9 ms |
| 100k | TQ | dense-b | 64 | 56.7 ms | 60.2 ms | 63.3 ms | 57.1 ms |
| 100k | TQ | dense-b-typed | 32 | 30.4 ms | 34.2 ms | 36.5 ms | 30.4 ms |
| 100k | TQ | dense-b-typed | 64 | 58.9 ms | 75.7 ms | 78.5 ms | 61.5 ms |
| 100k | RaBitQ | row | 32 | 14.7 ms | 16.9 ms | 17.5 ms | 14.8 ms |
| 100k | RaBitQ | row | 64 | 26.8 ms | 29.4 ms | 30.8 ms | 27.1 ms |
| 100k | RaBitQ | dense-old | 32 | 12.2 ms | 13.8 ms | 15.7 ms | 12.3 ms |
| 100k | RaBitQ | dense-old | 64 | 23.5 ms | 25.0 ms | 26.4 ms | 23.7 ms |
| 100k | RaBitQ | dense-a | 32 | 12.6 ms | 14.7 ms | 15.6 ms | 12.7 ms |
| 100k | RaBitQ | dense-a | 64 | 23.4 ms | 25.1 ms | 26.5 ms | 23.6 ms |
| 100k | RaBitQ | dense-typed | 32 | 12.2 ms | 14.1 ms | 14.6 ms | 12.3 ms |
| 100k | RaBitQ | dense-typed | 64 | 23.5 ms | 24.8 ms | 25.7 ms | 23.6 ms |
| 100k | RaBitQ | dense-b | 32 | 14.0 ms | 16.0 ms | 16.7 ms | 14.0 ms |
| 100k | RaBitQ | dense-b | 64 | 25.6 ms | 27.1 ms | 28.1 ms | 25.8 ms |
| 100k | RaBitQ | dense-b-typed | 32 | 13.9 ms | 15.7 ms | 16.3 ms | 14.0 ms |
| 100k | RaBitQ | dense-b-typed | 64 | 26.5 ms | 28.5 ms | 31.8 ms | 26.7 ms |

## Recall

Recall and NDCG are identical across layout variants for each scale/quant/nprobe
combination. Representative nprobe 32/64 values:

| scale | quant | nprobe | recall@k | ndcg@k |
| --- | --- | ---: | ---: | ---: |
| 50k | TQ | 32 | 0.9420 | 0.9994 |
| 50k | TQ | 64 | 0.9420 | 0.9996 |
| 50k | RaBitQ | 32 | 0.7750 | 0.9896 |
| 50k | RaBitQ | 64 | 0.7770 | 0.9899 |
| 100k | TQ | 32 | 0.9370 | 0.9966 |
| 100k | TQ | 64 | 0.9560 | 0.9997 |
| 100k | RaBitQ | 32 | 0.7630 | 0.9875 |
| 100k | RaBitQ | 64 | 0.7750 | 0.9906 |

## Index Storage

Primary ANN index size only; pkey rows are omitted.

| scale | quant | variant | size | bytes/row |
| --- | --- | --- | ---: | ---: |
| 50k | TQ | row | 44.1 MiB | 925.2 |
| 50k | TQ | dense-old/a/typed | 39.8 MiB | 835.1 |
| 50k | TQ | dense-b/b-typed | 49.2 MiB | 1032.0 |
| 50k | RaBitQ | row | 15.2 MiB | 319.8 |
| 50k | RaBitQ | dense-old/a/typed | 11.6 MiB | 243.3 |
| 50k | RaBitQ | dense-b/b-typed | 12.9 MiB | 271.5 |
| 100k | TQ | row | 87.6 MiB | 918.2 |
| 100k | TQ | dense-old/a/typed | 78.9 MiB | 827.1 |
| 100k | TQ | dense-b/b-typed | 98.1 MiB | 1028.5 |
| 100k | RaBitQ | row | 29.7 MiB | 311.3 |
| 100k | RaBitQ | dense-old/a/typed | 22.5 MiB | 235.8 |
| 100k | RaBitQ | dense-b/b-typed | 25.1 MiB | 263.2 |

## Batch Counters

AVX2 counters at nprobe 32 show why dense-old regresses TQ and why dense-a
recovers it.

| scale | quant | variant | flushes | candidates | width <8 | width 8-15 | width 16-31 | width >=32 | kernel ms |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 50k | TQ | row | 9,147 | 2,328,863 | 1 | 3 | 9 | 9,134 | 564.5 |
| 50k | TQ | dense-old | 233,854 | 2,328,462 | 1,693 | 232,161 | 0 | 0 | 1109.3 |
| 50k | TQ | dense-a | 10,699 | 2,328,864 | 177 | 88 | 45 | 10,389 | 545.4 |
| 50k | TQ | dense-typed | 233,854 | 2,328,462 | 1,693 | 232,161 | 0 | 0 | 1317.7 |
| 50k | TQ | dense-b | 74,498 | 2,328,864 | 917 | 712 | 1,571 | 71,298 | 569.0 |
| 50k | TQ | dense-b-typed | 74,498 | 2,328,864 | 917 | 712 | 1,571 | 71,298 | 538.9 |
| 100k | TQ | row | 20,379 | 5,203,807 | 4 | 5 | 7 | 20,363 | 1256.1 |
| 100k | TQ | dense-old | 521,755 | 5,203,613 | 2,405 | 519,350 | 0 | 0 | 2520.7 |
| 100k | TQ | dense-a | 21,778 | 5,203,752 | 53 | 142 | 140 | 21,443 | 1263.7 |
| 100k | TQ | dense-typed | 521,755 | 5,203,613 | 2,405 | 519,350 | 0 | 0 | 2496.0 |
| 100k | TQ | dense-b | 164,416 | 5,203,752 | 972 | 875 | 1,262 | 161,307 | 1207.0 |
| 100k | TQ | dense-b-typed | 164,416 | 5,203,752 | 972 | 875 | 1,262 | 161,307 | 1250.0 |

