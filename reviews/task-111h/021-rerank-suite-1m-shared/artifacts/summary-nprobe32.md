# Task 111h 1M Shared-Table Rerank Format Sweep

Packet: `reviews/task-111h/021-rerank-suite-1m-shared`

Suite: `task111h-1m-rerank-format-width-shared`

Scope:

- Dataset: dbpedia OpenAI3 1536-dim 1M profile, prepared as 990000 corpus rows and 10000 query rows.
- Surface: shared table, one active index per cell; every cell drops the prior index before loading the next one.
- Index reloptions: `nlists=1024`, `nprobe=32`, `storage_format=coarse_rerank`, `coarse_bits=1`, `rerank=heap_f32`.
- Recall and latency sweeps ran nprobe `8,16,32,64,128,200`; this summary compares the nprobe 32 row.
- Latency: post-recall warm cache, concurrency 1, 100 iterations, `--force-index`, backend memory sampling enabled.

Primary nprobe 32 results:

| Placement / format | Width | Recall@10 | Lat p50 | Lat p95 | Index size | B/row | Build index |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| source f32 | 32 | 0.9470 | 12.1 ms | 14.0 ms | 226.8 MiB | 240.2 | 201.05s |
| source f32 | 64 | 0.9570 | 13.2 ms | 18.6 ms | 226.8 MiB | 240.2 | 167.96s |
| source f32 | 128 | 0.9580 | 15.5 ms | 17.3 ms | 226.8 MiB | 240.2 | 166.85s |
| source f32 | 256 | 0.9580 | 20.7 ms | 23.1 ms | 226.8 MiB | 240.2 | 168.41s |
| index f16 | 32 | 0.9470 | 11.4 ms | 13.1 ms | 3.3 GiB | 3568.5 | 213.99s |
| index f16 | 64 | 0.9570 | 13.7 ms | 16.5 ms | 3.2 GiB | 3441.4 | 212.33s |
| index f16 | 128 | 0.9580 | 20.2 ms | 37.8 ms | 3.1 GiB | 3378.6 | 207.23s |
| index f16 | 256 | 0.9580 | 25.7 ms | 50.6 ms | 3.1 GiB | 3376.9 | 204.14s |
| index rabitq4 | 32 | 0.9120 | 14.5 ms | 17.7 ms | 1.2 GiB | 1262.6 | 190.17s |
| index rabitq4 | 64 | 0.9160 | 11.6 ms | 13.6 ms | 1.0 GiB | 1136.5 | 189.05s |
| index rabitq4 | 128 | 0.9160 | 12.3 ms | 15.5 ms | 1014.4 MiB | 1074.4 | 190.95s |
| index rabitq4 | 256 | 0.9160 | 14.5 ms | 20.2 ms | 1012.8 MiB | 1072.7 | 189.07s |
| index rabitq8 | 32 | 0.9200 | 10.5 ms | 12.6 ms | 1.9 GiB | 2031.7 | 195.72s |
| index rabitq8 | 64 | 0.9250 | 11.9 ms | 14.3 ms | 1.8 GiB | 1905.1 | 199.00s |
| index rabitq8 | 128 | 0.9250 | 14.0 ms | 19.4 ms | 1.7 GiB | 1842.8 | 212.55s |
| index rabitq8 | 256 | 0.9250 | 19.5 ms | 30.9 ms | 1.7 GiB | 1841.2 | 196.95s |
| index turboquant | 32 | 0.9090 | 10.5 ms | 12.7 ms | 1.2 GiB | 1262.3 | 179.71s |
| index turboquant | 64 | 0.9140 | 11.1 ms | 13.2 ms | 1.0 GiB | 1136.2 | 176.75s |
| index turboquant | 128 | 0.9150 | 12.0 ms | 15.6 ms | 1013.9 MiB | 1073.9 | 179.21s |
| index turboquant | 256 | 0.9150 | 15.3 ms | 22.7 ms | 985.6 MiB | 1043.9 | 187.39s |

Readout:

- Source-side f32 remains the recall baseline and adds only the baseline IVF index footprint in this measurement: 226.8 MiB, 240.2 B/row.
- Index-side f16 matches source-f32 recall, but it is not compact in this implementation: 3.1-3.3 GiB and 3376.9-3568.5 B/row.
- Rabitq4 and turboquant occupy essentially the same storage class. Turboquant is usually slightly faster at nprobe 32, but does not beat rabitq4 recall in this run.
- Rabitq8 improves nprobe 32 recall over rabitq4/turboquant by about 0.009-0.011 absolute, but costs roughly 0.7-0.8 GiB more than the rabitq4/turboquant class.
- No index-side quantized format in this matrix beats source-f32 on recall and storage together.

Durable evidence:

- Suite config: `artifacts/task111h-1m-rerank-format-width-shared-suite.json`
- Suite manifest: `artifacts/suite-manifest.json`
- Parsed results: `artifacts/results.jsonl`
- Report replay: `artifacts/results-report.jsonl`
- Raw logs: `artifacts/suite/*.log`
