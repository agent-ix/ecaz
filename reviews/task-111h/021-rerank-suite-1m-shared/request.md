# Review Request: 1M Shared-Table Rerank Format Sweep

## Scope

This packet adds benchmark evidence for Task 111h's persisted rerank format sweep on the 1M dbpedia/OpenAI3 fixture. It compares:

- source-side f32 rerank baseline;
- index-side f16;
- index-side rabitq4;
- index-side rabitq8;
- index-side turboquant;
- widths 32, 64, 128, and 256 for each format.

The suite used `ecaz bench suite` only. It completed 124 steps with 0 failed, 0 skipped, 0 missing, and 0 stale artifacts.

## Evidence

- Artifact manifest: `artifacts/manifest.md`
- Human nprobe 32 summary: `artifacts/summary-nprobe32.md`
- Suite config: `artifacts/task111h-1m-rerank-format-width-shared-suite.json`
- Suite manifest: `artifacts/suite-manifest.json`
- Parsed results: `artifacts/results.jsonl`
- Report replay: `artifacts/results-report.jsonl`
- Status/report logs: `artifacts/suite-status.log`, `artifacts/suite-report.log`
- Raw step logs: `artifacts/suite/*.log`

## Key Nprobe 32 Results

| Placement / format | Width | Recall@10 | Lat p50 | Lat p95 | Index size | B/row |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| source f32 | 32 | 0.9470 | 12.1 ms | 14.0 ms | 226.8 MiB | 240.2 |
| source f32 | 64 | 0.9570 | 13.2 ms | 18.6 ms | 226.8 MiB | 240.2 |
| source f32 | 128 | 0.9580 | 15.5 ms | 17.3 ms | 226.8 MiB | 240.2 |
| source f32 | 256 | 0.9580 | 20.7 ms | 23.1 ms | 226.8 MiB | 240.2 |
| index f16 | 32 | 0.9470 | 11.4 ms | 13.1 ms | 3.3 GiB | 3568.5 |
| index f16 | 64 | 0.9570 | 13.7 ms | 16.5 ms | 3.2 GiB | 3441.4 |
| index f16 | 128 | 0.9580 | 20.2 ms | 37.8 ms | 3.1 GiB | 3378.6 |
| index f16 | 256 | 0.9580 | 25.7 ms | 50.6 ms | 3.1 GiB | 3376.9 |
| index rabitq4 | 32 | 0.9120 | 14.5 ms | 17.7 ms | 1.2 GiB | 1262.6 |
| index rabitq4 | 64 | 0.9160 | 11.6 ms | 13.6 ms | 1.0 GiB | 1136.5 |
| index rabitq4 | 128 | 0.9160 | 12.3 ms | 15.5 ms | 1014.4 MiB | 1074.4 |
| index rabitq4 | 256 | 0.9160 | 14.5 ms | 20.2 ms | 1012.8 MiB | 1072.7 |
| index rabitq8 | 32 | 0.9200 | 10.5 ms | 12.6 ms | 1.9 GiB | 2031.7 |
| index rabitq8 | 64 | 0.9250 | 11.9 ms | 14.3 ms | 1.8 GiB | 1905.1 |
| index rabitq8 | 128 | 0.9250 | 14.0 ms | 19.4 ms | 1.7 GiB | 1842.8 |
| index rabitq8 | 256 | 0.9250 | 19.5 ms | 30.9 ms | 1.7 GiB | 1841.2 |
| index turboquant | 32 | 0.9090 | 10.5 ms | 12.7 ms | 1.2 GiB | 1262.3 |
| index turboquant | 64 | 0.9140 | 11.1 ms | 13.2 ms | 1.0 GiB | 1136.2 |
| index turboquant | 128 | 0.9150 | 12.0 ms | 15.6 ms | 1013.9 MiB | 1073.9 |
| index turboquant | 256 | 0.9150 | 15.3 ms | 22.7 ms | 985.6 MiB | 1043.9 |

## Interpretation

- The earlier f16 concern is validated as an implementation/layout issue, not as evidence that compact f16 is intrinsically bad. Current index-side f16 matches source-f32 recall but consumes 3.1-3.3 GiB, far larger than the 226.8 MiB source-f32 IVF index.
- Source-side f32 remains the high-recall baseline. It is not beaten on recall+storage by any index-side quantized option in this packet.
- Turboquant is fast and compact, but at nprobe 32 it does not beat rabitq4 recall at any width. It mostly tracks rabitq4 storage and latency while landing 0.001-0.003 recall lower.
- Rabitq8 is the only quantized rerank format here that clearly improves recall over rabitq4/turboquant, but it costs roughly 0.7-0.8 GiB more index storage and still remains well below source-f32 recall.

## Caveats

- This is the 1M shared-table lane, not the full Task 111h closeout. It does not satisfy remaining Task 111h requirements such as 0x2A legacy sidecar baseline, table-owned compact payload/blocker evidence, copy/double-copy measurements, EXPLAIN/admin/counter coverage, PG18 correctness fixtures, or the full 10k/50k/100k/1M decision matrix.
- The first source-f32 load includes the initial table load. Later cells reuse the shared corpus table and load logs show corpus/query chunks skipped.
- Corpus TSVs, parquet shards, and the exact-truth cache are not committed per AGENTS.md. Manifest paths and SHA256s are recorded in `artifacts/manifest.md`.
