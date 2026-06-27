# Task 111h / Packet 018 Review Request: 50k Rerank Format-Width Suite

## Summary

This packet adds and runs the 50k `ecaz bench suite` rerank format/width
matrix. It is measurement evidence only; there are no staged code changes in
this packet.

The suite completed cleanly:

```text
[suite:task111h-50k-rerank-format-width] completed=81 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## What Was Tested

- Formats: source-side f32, index f16, index rabitq4, index rabitq8, index turboquant.
- Widths: 32, 64, 128, 256.
- Per cell: isolated 50k load, recall sweep, latency sweep, storage check.
- Recall/latency nprobe sweep: 8, 16, 32, 64, 128, 200.
- Latency mode: 200 iterations, concurrency 1, post-recall-warm, force-index, memory samples, Task 87 counters.

Full structured results are in `artifacts/results-report.jsonl`. The nprobe=32
comparison view is in `artifacts/summary-nprobe32.md`.

## Nprobe 32 Summary

| placement | format | width | recall@10 | p50 latency | index size | index B/row | total size |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| source | f32 | 32 | 0.9520 | 3.74 ms | 13.8 MiB | 290.3 | 808.7 MiB |
| source | f32 | 64 | 0.9590 | 4.50 ms | 13.8 MiB | 290.3 | 808.7 MiB |
| source | f32 | 128 | 0.9600 | 6.42 ms | 13.8 MiB | 290.3 | 808.7 MiB |
| source | f32 | 256 | 0.9600 | 10.2 ms | 13.8 MiB | 290.3 | 808.7 MiB |
| index | f16 | 32 | 0.9520 | 3.08 ms | 172.5 MiB | 3618.2 | 967.4 MiB |
| index | f16 | 64 | 0.9590 | 4.31 ms | 166.7 MiB | 3495.5 | 961.5 MiB |
| index | f16 | 128 | 0.9600 | 6.80 ms | 164.0 MiB | 3439.8 | 958.9 MiB |
| index | f16 | 256 | 0.9600 | 9.31 ms | 163.5 MiB | 3429.5 | 958.4 MiB |
| index | rabitq4 | 32 | 0.9180 | 2.57 ms | 62.3 MiB | 1307.0 | 857.2 MiB |
| index | rabitq4 | 64 | 0.9200 | 3.04 ms | 56.6 MiB | 1187.8 | 851.5 MiB |
| index | rabitq4 | 128 | 0.9200 | 3.38 ms | 54.0 MiB | 1132.5 | 848.9 MiB |
| index | rabitq4 | 256 | 0.9200 | 5.30 ms | 53.7 MiB | 1125.3 | 848.5 MiB |
| index | rabitq8 | 32 | 0.9230 | 2.73 ms | 99.2 MiB | 2080.4 | 894.1 MiB |
| index | rabitq8 | 64 | 0.9265 | 3.42 ms | 93.4 MiB | 1959.5 | 888.3 MiB |
| index | rabitq8 | 128 | 0.9260 | 4.15 ms | 90.8 MiB | 1903.3 | 885.6 MiB |
| index | rabitq8 | 256 | 0.9260 | 7.21 ms | 90.5 MiB | 1896.9 | 885.3 MiB |
| index | turboquant | 32 | 0.9175 | 2.69 ms | 62.3 MiB | 1306.3 | 857.1 MiB |
| index | turboquant | 64 | 0.9195 | 3.31 ms | 56.6 MiB | 1187.0 | 851.5 MiB |
| index | turboquant | 128 | 0.9200 | 3.47 ms | 53.9 MiB | 1129.8 | 848.7 MiB |
| index | turboquant | 256 | 0.9200 | 4.35 ms | 52.8 MiB | 1107.2 | 847.7 MiB |

## Interpretation Boundaries

- This 50k run does not reproduce a 150 ms f16 or turboquant latency. Across all 120 latency rows, p50 ranged from 1.83 ms to 19.7 ms; the largest single-query max was 38.6 ms.
- At nprobe=32, index f16 matches source f32 recall at every width and is faster at widths 32, 64, and 256, but the current index layout adds large storage: 163.5 to 172.5 MiB of index size versus 13.8 MiB for source f32.
- Rabitq4, rabitq8, and turboquant are faster/smaller than f16 but lose substantial recall on this 50k corpus. At nprobe=32, recall is roughly 0.918 to 0.926 versus 0.952 to 0.960 for source/f16.
- Rabitq8 improves recall over rabitq4 only modestly here: nprobe32 width 32 is 0.9230 versus 0.9180, and nprobe200 width 256 is 0.9540 versus 0.9455. It also uses much more index storage than rabitq4.
- Turboquant width scaling is visible in packet-local counters: at nprobe=32, turboquant scored 6400/12800/25600/51200 candidates for widths 32/64/128/256, with kernel elapsed 1.596921/3.318962/5.922408/12.496158 ms. Recall plateaued at 0.9200 by width 128.

This is not final acceptance evidence for Task 111h. It does not cover 100k or
1M corpus scales, table-owned compact storage, cold-cache behavior, remote
hosts, or the legacy 0x2A/vanilla IVF baseline.

## Review Ask

Please review whether this 50k suite packet is sufficient evidence for the
50k warm-cache local slice, and whether the interpretation above is strictly
supported by the packet-local artifacts.
