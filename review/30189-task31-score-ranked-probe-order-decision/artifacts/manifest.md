# Artifact Manifest

## Packet

- head SHA: `9d1e59b9`
- implementation head under test: `422e5ddd`
- packet/topic: `review/30189-task31-score-ranked-probe-order-decision`
- lane: `task31 score-ranked probe-order decision`
- fixture: copied normalized outputs from `30187` and `30188`
- storage format: `pq_fastscan`
- rerank mode: `heap_f32`
- isolation/shared-table surface: one-index-per-table Task 31 prefix reused from the loaded `task31_m5_real100k_pqg8_n128` surface
- timestamp: 2026-05-03

## Artifacts

### `balanced-suite-manifest.json`

- command: copied from `review/30188-task31-suite-balanced-score-ranked/artifacts/suite-manifest.json`
- key result: balanced suite run completed `3` steps with no failures, missing artifacts, or stale artifacts

### `balanced-results.jsonl`

- command: copied from `review/30188-task31-suite-balanced-score-ranked/artifacts/results.jsonl`
- key result: balanced `nprobe=96,rerank_width=500` recorded `recall@k=0.9676`, `p50=10.7 ms`, `p95=11.4 ms`, `p99=11.9 ms`

### `quality-suite-manifest.json`

- command: copied from `review/30187-task31-suite-quality-score-ranked/artifacts/suite-manifest.json`
- key result: quality suite run passed `quality-candidate-recall100-floor` with actual `0.992` and `quality-candidate-p50-budget-ms` with actual `13.1`

### `quality-results.jsonl`

- command: copied from `review/30187-task31-suite-quality-score-ranked/artifacts/results.jsonl`
- key result: quality `nprobe=96,rerank_width=1000` recorded `recall@k=0.9920`, `p50=13.1 ms`, `p95=14.2 ms`, `p99=15.1 ms`
