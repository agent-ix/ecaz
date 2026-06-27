# Task 111h Packet 019: 100k Nprobe 32 Summary

Source: `artifacts/results.jsonl`, filtered to recall and latency rows where
`nprobe=32`, joined with `storage_index`, `storage_field total`, and
`load_timing phase=build_index` rows for the same prefix.

| placement | format | width | recall@10 | NDCG@10 | p50 latency | p95 latency | index size | index B/row | total size | build_s |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| source | f32 | 32 | 0.9285 | 0.9947 | 4.57 ms | 5.13 ms | 24.6 MiB | 258.2 | 1.6 GiB | 9.230000 |
| source | f32 | 64 | 0.9350 | 0.9948 | 5.35 ms | 6.09 ms | 24.6 MiB | 258.2 | 1.6 GiB | 8.980000 |
| source | f32 | 128 | 0.9350 | 0.9948 | 7.41 ms | 8.72 ms | 24.6 MiB | 258.2 | 1.6 GiB | 8.830000 |
| source | f32 | 256 | 0.9350 | 0.9948 | 12.2 ms | 15.4 ms | 24.6 MiB | 258.2 | 1.6 GiB | 9.130000 |
| index | f16 | 32 | 0.9280 | 0.9947 | 4.38 ms | 5.52 ms | 342.0 MiB | 3586.5 | 1.9 GiB | 12.730000 |
| index | f16 | 64 | 0.9345 | 0.9948 | 5.92 ms | 7.66 ms | 330.1 MiB | 3461.8 | 1.9 GiB | 12.960000 |
| index | f16 | 128 | 0.9345 | 0.9948 | 8.98 ms | 13.1 ms | 324.3 MiB | 3400.3 | 1.9 GiB | 12.940000 |
| index | f16 | 256 | 0.9345 | 0.9948 | 13.8 ms | 20.0 ms | 323.7 MiB | 3394.4 | 1.9 GiB | 12.850000 |
| index | rabitq4 | 32 | 0.8895 | 0.9942 | 3.76 ms | 4.34 ms | 121.8 MiB | 1277.5 | 1.7 GiB | 11.530000 |
| index | rabitq4 | 64 | 0.8930 | 0.9943 | 4.30 ms | 4.97 ms | 110.2 MiB | 1155.2 | 1.7 GiB | 11.760000 |
| index | rabitq4 | 128 | 0.8930 | 0.9943 | 6.82 ms | 8.11 ms | 104.5 MiB | 1095.5 | 1.7 GiB | 11.550000 |
| index | rabitq4 | 256 | 0.8930 | 0.9943 | 6.19 ms | 8.38 ms | 104.0 MiB | 1090.8 | 1.7 GiB | 10.970000 |
| index | rabitq8 | 32 | 0.8990 | 0.9944 | 4.21 ms | 5.23 ms | 195.4 MiB | 2049.0 | 1.7 GiB | 11.730000 |
| index | rabitq8 | 64 | 0.9000 | 0.9945 | 4.62 ms | 5.89 ms | 183.6 MiB | 1925.4 | 1.7 GiB | 11.720000 |
| index | rabitq8 | 128 | 0.9000 | 0.9945 | 6.05 ms | 7.85 ms | 177.9 MiB | 1865.1 | 1.7 GiB | 11.970000 |
| index | rabitq8 | 256 | 0.9000 | 0.9945 | 9.52 ms | 12.7 ms | 177.4 MiB | 1860.2 | 1.7 GiB | 13.250000 |
| index | turboquant | 32 | 0.8965 | 0.9944 | 3.83 ms | 4.35 ms | 121.8 MiB | 1276.8 | 1.7 GiB | 10.930000 |
| index | turboquant | 64 | 0.9005 | 0.9945 | 4.18 ms | 4.83 ms | 110.1 MiB | 1154.4 | 1.7 GiB | 11.670000 |
| index | turboquant | 128 | 0.9000 | 0.9945 | 5.00 ms | 5.76 ms | 104.4 MiB | 1094.3 | 1.7 GiB | 11.570000 |
| index | turboquant | 256 | 0.9000 | 0.9945 | 6.10 ms | 7.34 ms | 101.8 MiB | 1067.9 | 1.7 GiB | 10.880000 |

Additional checks from `artifacts/results.jsonl`:

- All 120 latency rows: p50 min `2.15 ms` (`index_rabitq4_w32`, nprobe 8); p50 max `32.9 ms` (`index_f16_w256`, nprobe 200); max single-query latency `211.8 ms` (`index_f16_w256`, nprobe 32).
- Turboquant nprobe32 latency rows: w32 p50 `3.83 ms` max `8.32 ms`; w64 p50 `4.18 ms` max `9.04 ms`; w128 p50 `5.00 ms` max `10.2 ms`; w256 p50 `6.10 ms` max `12.0 ms`.
- Turboquant nprobe32 block-kernel counters scaled directly with width: w32 `6400` candidates / `1.653836 ms`, w64 `12800` / `3.257094 ms`, w128 `25600` / `6.829726 ms`, w256 `51200` / `12.567503 ms`.
- Nprobe200 width endpoints: source f32 w32 `0.9875`, source f32 w256 `0.9990`; index f16 w32 `0.9870`, index f16 w256 `0.9980`; rabitq4 w32 `0.9360`, rabitq4 w256 `0.9420`; rabitq8 w32 `0.9475`, rabitq8 w256 `0.9520`; turboquant w32 `0.9460`, turboquant w256 `0.9520`.
