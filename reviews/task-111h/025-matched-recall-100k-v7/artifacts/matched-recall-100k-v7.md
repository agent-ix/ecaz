# Matched-Recall 100k v7 Analysis

Source data: packet 024 post-v7 100k rerank suite reports.

Selection rule: for each target and format, pick the lowest-p50 row that reaches
the target recall. If no row reaches the target, report the maximum observed
recall row.

## Target recall@10 >= 0.90

| Placement / format | Selected row | recall@10 | p50 | p95 | p99 | Index size |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| source f32 | w32 np32 | 0.9285 | 4.50 ms | 5.02 ms | 5.36 ms | 24.6 MiB |
| index f16 | w32 np32 | 0.9280 | 4.36 ms | 5.55 ms | 7.90 ms | 342.0 MiB |
| index RaBitQ4 | w32 np64 | 0.9155 | 5.70 ms | 6.60 ms | 7.12 ms | 121.8 MiB |
| index RaBitQ8 | w32 np32 | 0.9010 | 3.80 ms | 4.56 ms | 5.15 ms | 195.4 MiB |
| index TurboQuant | w32 np32 | 0.9040 | 3.91 ms | 4.52 ms | 4.88 ms | 121.8 MiB |

## Target recall@10 >= 0.93

| Placement / format | Selected row | recall@10 | p50 | p95 | p99 | Index size |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| source f32 | w64 np32 | 0.9350 | 5.57 ms | 6.49 ms | 7.14 ms | 24.6 MiB |
| index f16 | w32 np64 | 0.9620 | 6.51 ms | 8.06 ms | 8.60 ms | 342.0 MiB |
| index RaBitQ4 | w32 np128 | 0.9310 | 9.49 ms | 10.70 ms | 12.50 ms | 121.8 MiB |
| index RaBitQ8 | w64 np64 | 0.9345 | 6.83 ms | 7.87 ms | 8.44 ms | 183.6 MiB |
| index TurboQuant | w32 np64 | 0.9330 | 5.89 ms | 7.32 ms | 8.82 ms | 121.8 MiB |

## Target recall@10 >= 0.95

| Placement / format | Selected row | recall@10 | p50 | p95 | p99 | Index size |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| source f32 | w32 np64 | 0.9625 | 6.23 ms | 6.98 ms | 7.51 ms | 24.6 MiB |
| index f16 | w32 np64 | 0.9620 | 6.51 ms | 8.06 ms | 8.60 ms | 342.0 MiB |
| index RaBitQ4 | NO_REACH; max w64 np200 | 0.9380 | 15.30 ms | 17.30 ms | 18.90 ms | 110.2 MiB |
| index RaBitQ8 | w64 np200 | 0.9525 | 14.40 ms | 15.80 ms | 16.80 ms | 183.6 MiB |
| index TurboQuant | w128 np128 | 0.9530 | 11.60 ms | 14.50 ms | 18.10 ms | 104.4 MiB |

## Target recall@10 >= 0.97

| Placement / format | Selected row | recall@10 | p50 | p95 | p99 | Index size |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| source f32 | w64 np64 | 0.9720 | 7.65 ms | 8.61 ms | 10.00 ms | 24.6 MiB |
| index f16 | w64 np64 | 0.9710 | 8.75 ms | 11.00 ms | 13.40 ms | 330.1 MiB |
| index RaBitQ4 | NO_REACH; max w64 np200 | 0.9380 | 15.30 ms | 17.30 ms | 18.90 ms | 110.2 MiB |
| index RaBitQ8 | NO_REACH; max w64 np200 | 0.9525 | 14.40 ms | 15.80 ms | 16.80 ms | 183.6 MiB |
| index TurboQuant | NO_REACH; max w64 np200 | 0.9565 | 19.70 ms | 21.00 ms | 22.00 ms | 110.1 MiB |

## Target recall@10 >= 0.99

| Placement / format | Selected row | recall@10 | p50 | p95 | p99 | Index size |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| source f32 | w64 np128 | 0.9945 | 11.10 ms | 12.00 ms | 13.10 ms | 24.6 MiB |
| index f16 | w64 np128 | 0.9935 | 11.70 ms | 13.40 ms | 16.10 ms | 330.1 MiB |
| index RaBitQ4 | NO_REACH; max w64 np200 | 0.9380 | 15.30 ms | 17.30 ms | 18.90 ms | 110.2 MiB |
| index RaBitQ8 | NO_REACH; max w64 np200 | 0.9525 | 14.40 ms | 15.80 ms | 16.80 ms | 183.6 MiB |
| index TurboQuant | NO_REACH; max w64 np200 | 0.9565 | 19.70 ms | 21.00 ms | 22.00 ms | 110.1 MiB |

## Interpretation

- At the low 0.90 target, RaBitQ8 and TurboQuant are faster than source f32 by
  roughly 0.6-0.7 ms p50, but they require much larger ec_ivf indexes and return
  lower recall than the selected source row.
- At 0.93, source f32 is already faster than f16, RaBitQ4, and RaBitQ8, and only
  0.32 ms p50 faster than TurboQuant while using a 24.6 MiB ec_ivf index instead
  of 121.8 MiB.
- At 0.95, source f32 is faster than every compact quantized option that reaches
  the target. RaBitQ4 does not reach the target.
- At 0.97 and 0.99, only source f32 and f16 reach the target in this post-v7
  100k grid. Source f32 is faster and its ec_ivf index is roughly 13x smaller
  than f16.
- On this warm-cache local 100k data, the matched-recall table promotes source
  f32 as the reference path. It does not justify promoting the current compact
  index-side formats. f16 remains recall-neutral but layout/storage-prohibitive;
  RaBitQ4/RaBitQ8/TurboQuant remain iterate candidates for other storage/cache
  conditions or further representation changes, not abandon decisions.
