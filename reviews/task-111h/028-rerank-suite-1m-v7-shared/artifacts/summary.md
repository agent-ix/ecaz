# Task 111h 1M Rerank Format/Width Sweep Summary

Packet: `reviews/task-111h/028-rerank-suite-1m-v7-shared`

Head SHA before packet commit: `9f8432220c65b8d0d590d29899e5cb6e3874f44f`

Suite: `task111h-1m-rerank-format-width-v7-shared`

Dataset/profile: `dbpedia-openai3-large-1536-1m`, staged as `ec_real_ann_benchmarks_anchor` with 990,000 corpus rows and 10,000 query rows.

Database/socket: `task111h_rerank_1m_v7`, `/home/peter/.pgrx`, port `28818`.

Run shape:

- One shared corpus table prefix: `task111h028_1m_shared`.
- One active IVF index per cell; every cell drops its index before and after.
- 20 load cells: `source/f32`, `index/f16`, `index/rabitq4`, `index/rabitq8`, `index/turboquant` at widths 32, 64, 128, and 256.
- Recall sweep: `nprobe=8,16,32,64,128,200`, `k=10`, 100 queries, shared truth cache.
- Latency sweep: same nprobe values, `k=10`, concurrency 1, 100 iterations, `cache_state=post_recall_warm`.
- Storage measured after each load cell.

## Nprobe 32

| Cell | Recall@10 | Recall mean q-time | Latency mean | Latency p95 | IVF index size | IVF index B/row | Build index s |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| source/f32 w32 | 0.9470 | 17.70 ms | 12.0 ms | 14.3 ms | 226.8 MiB | 240.2 | 220.11 |
| source/f32 w64 | 0.9570 | 12.44 ms | 12.4 ms | 14.0 ms | 226.8 MiB | 240.2 | 163.27 |
| source/f32 w128 | 0.9580 | 15.44 ms | 15.6 ms | 17.5 ms | 226.8 MiB | 240.2 | 165.20 |
| source/f32 w256 | 0.9580 | 20.62 ms | 21.2 ms | 23.5 ms | 226.8 MiB | 240.2 | 162.43 |
| index/f16 w32 | 0.9470 | 12.07 ms | 11.7 ms | 13.6 ms | 3.3 GiB | 3568.5 | 219.65 |
| index/f16 w64 | 0.9570 | 15.39 ms | 13.5 ms | 16.1 ms | 3.2 GiB | 3441.4 | 217.89 |
| index/f16 w128 | 0.9580 | 27.06 ms | 19.5 ms | 28.5 ms | 3.1 GiB | 3378.6 | 216.55 |
| index/f16 w256 | 0.9580 | 45.45 ms | 29.7 ms | 50.7 ms | 3.1 GiB | 3376.9 | 210.17 |
| index/rabitq4 w32 | 0.9070 | 10.54 ms | 10.7 ms | 13.0 ms | 1.2 GiB | 1262.6 | 187.22 |
| index/rabitq4 w64 | 0.9100 | 11.29 ms | 11.3 ms | 13.3 ms | 1.0 GiB | 1136.5 | 191.19 |
| index/rabitq4 w128 | 0.9100 | 11.95 ms | 12.5 ms | 15.8 ms | 1014.4 MiB | 1074.4 | 181.70 |
| index/rabitq4 w256 | 0.9100 | 15.97 ms | 16.3 ms | 23.3 ms | 1012.8 MiB | 1072.7 | 207.16 |
| index/rabitq8 w32 | 0.9140 | 10.99 ms | 11.2 ms | 13.6 ms | 1.9 GiB | 2031.7 | 202.21 |
| index/rabitq8 w64 | 0.9200 | 12.14 ms | 12.0 ms | 14.4 ms | 1.8 GiB | 1905.1 | 190.93 |
| index/rabitq8 w128 | 0.9210 | 14.92 ms | 14.1 ms | 18.0 ms | 1.7 GiB | 1842.8 | 194.96 |
| index/rabitq8 w256 | 0.9210 | 22.13 ms | 19.5 ms | 29.3 ms | 1.7 GiB | 1841.2 | 190.28 |
| index/turboquant w32 | 0.9160 | 10.33 ms | 14.6 ms | 17.0 ms | 1.2 GiB | 1262.3 | 178.57 |
| index/turboquant w64 | 0.9220 | 10.74 ms | 11.0 ms | 13.0 ms | 1.0 GiB | 1136.2 | 175.13 |
| index/turboquant w128 | 0.9230 | 12.13 ms | 12.2 ms | 15.6 ms | 1013.9 MiB | 1073.9 | 174.52 |
| index/turboquant w256 | 0.9230 | 14.61 ms | 15.2 ms | 20.5 ms | 985.6 MiB | 1043.9 | 175.82 |

## Nprobe 200 Endpoint

| Cell | Recall@10 | Recall mean q-time | Latency mean | Latency p95 |
| --- | ---: | ---: | ---: | ---: |
| source/f32 w64 | 0.9880 | 39.74 ms | 41.1 ms | 46.4 ms |
| source/f32 w128 | 0.9910 | 42.91 ms | 43.9 ms | 51.2 ms |
| index/f16 w64 | 0.9880 | 42.83 ms | 43.2 ms | 50.5 ms |
| index/f16 w128 | 0.9910 | 54.42 ms | 63.9 ms | 91.1 ms |
| index/rabitq4 w128 | 0.9370 | 41.46 ms | 41.8 ms | 48.3 ms |
| index/rabitq8 w128 | 0.9520 | 43.18 ms | 47.9 ms | 60.9 ms |
| index/turboquant w128 | 0.9510 | 39.92 ms | 42.2 ms | 48.9 ms |

## Conclusions From This Run

- `source/f32 w64` is the strongest normal operating point in this sweep: `0.9570` recall at `nprobe=32` with `12.4 ms` formal latency and a `226.8 MiB` IVF index. It does not add index-side rerank payload beyond the baseline coarse-rerank index.
- Wider source/f32 increases high-nprobe recall: `source/f32 w128` reaches `0.9910` at `nprobe=200`, but costs more latency than w64.
- Current index/f16 does not win. It matches source/f32 recall, but adds roughly `3.1-3.3 GiB` of IVF index storage and gets slower as width grows.
- rabitq4 is storage/latency competitive but recall-limited in this suite: best observed endpoint is `0.9370`.
- rabitq8 and turboquant improve over rabitq4, but only reach about `0.95` recall at `nprobe=200`, while source/f32 is already `0.9880-0.9910` there.
- No quantized index-side rerank mode in this v7 shared-table sweep beats source/f32 on the combined recall/latency/storage tradeoff.
