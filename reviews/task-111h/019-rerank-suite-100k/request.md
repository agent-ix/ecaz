# Task 111h / Packet 019 Review Request: 100k Rerank Format-Width Suite

## Summary

This packet adds and runs the 100k `ecaz bench suite` rerank format/width
matrix. It is measurement evidence only; there are no staged code changes in
this packet.

The suite completed cleanly:

```text
[suite:task111h-100k-rerank-format-width] completed=81 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## What Was Tested

- Formats: source-side f32, index f16, index rabitq4, index rabitq8, index turboquant.
- Widths: 32, 64, 128, 256.
- Per cell: isolated 100k load, recall sweep, latency sweep, storage check.
- Recall/latency nprobe sweep: 8, 16, 32, 64, 128, 200.
- Latency mode: 200 iterations, concurrency 1, post-recall-warm, force-index, memory samples, Task 87 counters.

Full structured results are in `artifacts/results-report.jsonl`. The nprobe=32
comparison view is in `artifacts/summary-nprobe32.md`.

## Nprobe 32 Summary

| placement | format | width | recall@10 | p50 latency | index size | index B/row | total size |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| source | f32 | 32 | 0.9285 | 4.57 ms | 24.6 MiB | 258.2 | 1.6 GiB |
| source | f32 | 64 | 0.9350 | 5.35 ms | 24.6 MiB | 258.2 | 1.6 GiB |
| source | f32 | 128 | 0.9350 | 7.41 ms | 24.6 MiB | 258.2 | 1.6 GiB |
| source | f32 | 256 | 0.9350 | 12.2 ms | 24.6 MiB | 258.2 | 1.6 GiB |
| index | f16 | 32 | 0.9280 | 4.38 ms | 342.0 MiB | 3586.5 | 1.9 GiB |
| index | f16 | 64 | 0.9345 | 5.92 ms | 330.1 MiB | 3461.8 | 1.9 GiB |
| index | f16 | 128 | 0.9345 | 8.98 ms | 324.3 MiB | 3400.3 | 1.9 GiB |
| index | f16 | 256 | 0.9345 | 13.8 ms | 323.7 MiB | 3394.4 | 1.9 GiB |
| index | rabitq4 | 32 | 0.8895 | 3.76 ms | 121.8 MiB | 1277.5 | 1.7 GiB |
| index | rabitq4 | 64 | 0.8930 | 4.30 ms | 110.2 MiB | 1155.2 | 1.7 GiB |
| index | rabitq4 | 128 | 0.8930 | 6.82 ms | 104.5 MiB | 1095.5 | 1.7 GiB |
| index | rabitq4 | 256 | 0.8930 | 6.19 ms | 104.0 MiB | 1090.8 | 1.7 GiB |
| index | rabitq8 | 32 | 0.8990 | 4.21 ms | 195.4 MiB | 2049.0 | 1.7 GiB |
| index | rabitq8 | 64 | 0.9000 | 4.62 ms | 183.6 MiB | 1925.4 | 1.7 GiB |
| index | rabitq8 | 128 | 0.9000 | 6.05 ms | 177.9 MiB | 1865.1 | 1.7 GiB |
| index | rabitq8 | 256 | 0.9000 | 9.52 ms | 177.4 MiB | 1860.2 | 1.7 GiB |
| index | turboquant | 32 | 0.8965 | 3.83 ms | 121.8 MiB | 1276.8 | 1.7 GiB |
| index | turboquant | 64 | 0.9005 | 4.18 ms | 110.1 MiB | 1154.4 | 1.7 GiB |
| index | turboquant | 128 | 0.9000 | 5.00 ms | 104.4 MiB | 1094.3 | 1.7 GiB |
| index | turboquant | 256 | 0.9000 | 6.10 ms | 101.8 MiB | 1067.9 | 1.7 GiB |

## Interpretation Boundaries

- This 100k run does not reproduce a 150 ms turboquant latency. At nprobe=32, turboquant p50 latencies were 3.83, 4.18, 5.00, and 6.10 ms for widths 32, 64, 128, and 256; the corresponding max latencies were 8.32, 9.04, 10.2, and 12.0 ms.
- Across all 120 latency rows, the worst p50 row was index f16 width256 at nprobe=200: 32.9 ms p50, 56.4 ms max. The largest single-query outlier was also index f16 width256, but at nprobe=32: 13.8 ms p50, 211.8 ms max.
- At nprobe=32, index f16 nearly matches source f32 recall at each width, but the current index-side layout adds large storage: 323.7 to 342.0 MiB of index size versus 24.6 MiB for source f32.
- Rabitq4, rabitq8, and turboquant are faster/smaller than f16 but lose substantial recall on this 100k corpus. At nprobe=32, compressed-format recall is 0.8895 to 0.9005 versus 0.9285 to 0.9350 for source f32 and 0.9280 to 0.9345 for index f16.
- Rabitq8 improves recall over rabitq4 here, but modestly relative to its storage cost: nprobe32 width32 is 0.8990 versus 0.8895, and nprobe200 width256 is 0.9520 versus 0.9420. Rabitq8 uses 177.4 to 195.4 MiB of index storage versus 104.0 to 121.8 MiB for rabitq4.
- Turboquant width scaling is visible in packet-local counters: at nprobe=32, turboquant scored 6400/12800/25600/51200 candidates for widths 32/64/128/256, with kernel elapsed 1.653836/3.257094/6.829726/12.567503 ms. Recall plateaued at roughly 0.900 by width 64.

This is not final acceptance evidence for Task 111h. It does not cover the 1M
corpus scale, table-owned compact storage, cold-cache behavior, remote hosts,
or the legacy 0x2A/vanilla IVF baseline.

## Review Ask

Please review whether this 100k suite packet is sufficient evidence for the
100k warm-cache local slice, and whether the interpretation above is strictly
supported by the packet-local artifacts.
