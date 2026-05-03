# Artifact Manifest

## Packet

- head SHA: `c1a761fd`
- packet/topic: `review/30197-task31-rerank-state-cache-decision`
- lane: `task31 rerank-state-cache decision`
- fixture: copied normalized outputs from `30195` and `30196`
- storage format: `pq_fastscan`
- rerank mode: `heap_f32`
- isolation/shared-table surface: one-index-per-table Task 31 prefix reused from the loaded `task31_m5_real100k_pqg8_n128` surface
- timestamp: 2026-05-03

## Artifacts

### `balanced-suite-manifest.json`

- command: copied from `review/30196-task31-suite-balanced-rerank-state-cache/artifacts/suite-manifest.json`
- key result: balanced suite run completed `3` steps with no failures, missing artifacts, or stale artifacts

### `balanced-results.jsonl`

- command: copied from `review/30196-task31-suite-balanced-rerank-state-cache/artifacts/results.jsonl`
- key result: balanced `nprobe=96,rerank_width=500` recorded `recall@k=0.9676`, `p50=10.7 ms`, `p95=11.6 ms`, `p99=12.1 ms`

### `quality-suite-manifest.json`

- command: copied from `review/30195-task31-suite-quality-rerank-state-cache/artifacts/suite-manifest.json`
- key result: quality suite run passed `quality-candidate-recall100-floor` with actual `0.992` and `quality-candidate-p50-budget-ms` with actual `12.8`

### `quality-results.jsonl`

- command: copied from `review/30195-task31-suite-quality-rerank-state-cache/artifacts/results.jsonl`
- key result: quality `nprobe=96,rerank_width=1000` recorded `recall@k=0.9920`, `p50=12.8 ms`, `p95=13.5 ms`, `p99=13.9 ms`
