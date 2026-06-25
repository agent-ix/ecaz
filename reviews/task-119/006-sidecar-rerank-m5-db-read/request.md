# Task 119: M5 DB-Read Sidecar Evidence

## Summary

This packet adds M5-local production-style sidecar read evidence for the viable
Task 119 rerank lanes:

- `f32`
- `rabitq8`
- `turboquant_4bit`
- `turboquant_8bit`

It uses `ecaz bench suite` with `read_mode=tid-sorted`, `ef_search=1000`,
`candidate_k=1000`, `queries_limit=200`, and 10k/50k/100k corpora.

This is a follow-up to the full free-I/O matrix in packet `005`; it is focused
on heap/source read behavior for the lanes that still looked viable.

## Artifacts

- Suite config: `reviews/task-119/006-sidecar-rerank-m5-db-read/suite.json`
- Manifest: `reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts/manifest.md`
- 10k results: `reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts/suite-results.10k.jsonl`
- 50k results: `reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts/suite-results.50k.jsonl`
- 100k results: `reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts/suite-results.100k.jsonl`
- Full table logs:
  - `reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts/sidecar-10k-hnsw-rabitq-db-read-viable-lanes.log`
  - `reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts/sidecar-50k-hnsw-rabitq-db-read-viable-lanes.log`
  - `reviews/task-119/006-sidecar-rerank-m5-db-read/artifacts/sidecar-100k-hnsw-rabitq-db-read-viable-lanes.log`

## Key 100k Result

| Variant | Recall@10 | heap/source reads p50 | sidecar I/O p50 | score p50 | total bound p50 | bytes/vector | sidecar size |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `f32` | 0.9850 | 1000 | 23.285 ms | 37.768 ms | 86.750 ms | 6144 | 585.94 MiB |
| `rabitq8` | 0.9420 | 1000 | 6.611 ms | 9.266 ms | 41.170 ms | 1548 | 147.63 MiB |
| `turboquant_4bit` | 0.9415 | 1000 | 4.598 ms | 10.004 ms | 39.873 ms | 772 | 73.62 MiB |
| `turboquant_8bit` | 0.9760 | 1000 | 8.245 ms | 84.082 ms | 117.642 ms | 1540 | 146.87 MiB |

## Outcome

This confirms that the production-style sidecar read cost does not rescue the
high-recall lanes:

- `f32` remains too large and too slow for the storage-saving goal.
- `turboquant_8bit` remains score-latency dominated.
- `rabitq8` and `turboquant_4bit` are the only compact practical lanes, with
  `turboquant_4bit` using about half the sidecar bytes/vector and nearly the
  same total-bound latency as `rabitq8`.

Recommendation for Task 119 remains: keep experimental and iterate; do not
promote HNSW RaBitQ coarse-rerank as a production profile yet.
