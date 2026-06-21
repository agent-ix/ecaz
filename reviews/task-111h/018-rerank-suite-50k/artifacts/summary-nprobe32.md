# Task 111h Packet 018: 50k Nprobe 32 Summary

Source: `artifacts/results.jsonl`, filtered to recall and latency rows where
`nprobe=32`, joined with `storage_index`, `storage_field total`, and
`load_timing phase=build_index` rows for the same prefix.

| placement | format | width | recall@10 | NDCG@10 | p50 latency | p95 latency | index size | index B/row | total size | build_s |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| source | f32 | 32 | 0.9520 | 0.9973 | 3.74 ms | 4.11 ms | 13.8 MiB | 290.3 | 808.7 MiB | 5.730000 |
| source | f32 | 64 | 0.9590 | 0.9974 | 4.50 ms | 4.87 ms | 13.8 MiB | 290.3 | 808.7 MiB | 6.060000 |
| source | f32 | 128 | 0.9600 | 0.9974 | 6.42 ms | 6.92 ms | 13.8 MiB | 290.3 | 808.7 MiB | 6.040000 |
| source | f32 | 256 | 0.9600 | 0.9974 | 10.2 ms | 11.2 ms | 13.8 MiB | 290.3 | 808.7 MiB | 5.390000 |
| index | f16 | 32 | 0.9520 | 0.9973 | 3.08 ms | 3.64 ms | 172.5 MiB | 3618.2 | 967.4 MiB | 7.050000 |
| index | f16 | 64 | 0.9590 | 0.9974 | 4.31 ms | 5.36 ms | 166.7 MiB | 3495.5 | 961.5 MiB | 7.380000 |
| index | f16 | 128 | 0.9600 | 0.9974 | 6.80 ms | 9.42 ms | 164.0 MiB | 3439.8 | 958.9 MiB | 7.620000 |
| index | f16 | 256 | 0.9600 | 0.9974 | 9.31 ms | 11.7 ms | 163.5 MiB | 3429.5 | 958.4 MiB | 7.120000 |
| index | rabitq4 | 32 | 0.9180 | 0.9969 | 2.57 ms | 2.87 ms | 62.3 MiB | 1307.0 | 857.2 MiB | 6.490000 |
| index | rabitq4 | 64 | 0.9200 | 0.9970 | 3.04 ms | 3.55 ms | 56.6 MiB | 1187.8 | 851.5 MiB | 6.990000 |
| index | rabitq4 | 128 | 0.9200 | 0.9970 | 3.38 ms | 4.12 ms | 54.0 MiB | 1132.5 | 848.9 MiB | 6.720000 |
| index | rabitq4 | 256 | 0.9200 | 0.9970 | 5.30 ms | 6.17 ms | 53.7 MiB | 1125.3 | 848.5 MiB | 6.860000 |
| index | rabitq8 | 32 | 0.9230 | 0.9970 | 2.73 ms | 3.14 ms | 99.2 MiB | 2080.4 | 894.1 MiB | 7.130000 |
| index | rabitq8 | 64 | 0.9265 | 0.9971 | 3.42 ms | 4.15 ms | 93.4 MiB | 1959.5 | 888.3 MiB | 7.040000 |
| index | rabitq8 | 128 | 0.9260 | 0.9971 | 4.15 ms | 5.38 ms | 90.8 MiB | 1903.3 | 885.6 MiB | 7.060000 |
| index | rabitq8 | 256 | 0.9260 | 0.9971 | 7.21 ms | 8.56 ms | 90.5 MiB | 1896.9 | 885.3 MiB | 7.150000 |
| index | turboquant | 32 | 0.9175 | 0.9970 | 2.69 ms | 3.00 ms | 62.3 MiB | 1306.3 | 857.1 MiB | 6.700000 |
| index | turboquant | 64 | 0.9195 | 0.9971 | 3.31 ms | 4.35 ms | 56.6 MiB | 1187.0 | 851.5 MiB | 6.560000 |
| index | turboquant | 128 | 0.9200 | 0.9971 | 3.47 ms | 4.11 ms | 53.9 MiB | 1129.8 | 848.7 MiB | 6.390000 |
| index | turboquant | 256 | 0.9200 | 0.9971 | 4.35 ms | 5.17 ms | 52.8 MiB | 1107.2 | 847.7 MiB | 6.260000 |

Additional checks from `artifacts/results.jsonl`:

- All 120 latency rows: p50 min `1.83 ms` (`index_rabitq4_w32`, nprobe 8); p50 max `19.7 ms` (`index_f16_w256`, nprobe 128); max single-query latency `38.6 ms` (`index_f16_w256`, nprobe 200).
- Turboquant nprobe32 block-kernel counters scaled directly with width: w32 `6400` candidates / `1.596921 ms`, w64 `12800` / `3.318962 ms`, w128 `25600` / `5.922408 ms`, w256 `51200` / `12.496158 ms`.
- Nprobe200 width endpoints: source f32 w32 `0.9895`, source f32 w256 `1.0000`; index f16 w32 `0.9895`, index f16 w256 `1.0000`; rabitq4 w32 `0.9425`, rabitq4 w256 `0.9455`; rabitq8 w32 `0.9500`, rabitq8 w256 `0.9540`; turboquant w32 `0.9445`, turboquant w256 `0.9470`.
