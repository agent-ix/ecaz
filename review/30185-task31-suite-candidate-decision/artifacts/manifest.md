# Artifact Manifest

## Packet

- head SHA: `c2ff36ee`
- packet/topic: `review/30185-task31-suite-candidate-decision`
- lane: `task31 suite-runner candidate decision`
- fixture: copied normalized outputs from `30183` and `30184`
- storage format: `pq_fastscan`
- rerank mode: `heap_f32`
- isolation/shared-table surface: one-index-per-table Task 31 prefix reused from the loaded `task31_m5_real100k_pqg8_n128` surface
- timestamp: 2026-05-03

## Artifacts

### `balanced-suite-manifest.json`

- command: copied from `review/30184-task31-suite-balanced-run/artifacts/suite-manifest.json`
- key result: balanced suite run completed `3` steps with no failures, missing artifacts, or stale artifacts

### `balanced-results.jsonl`

- command: copied from `review/30184-task31-suite-balanced-run/artifacts/results.jsonl`
- key result: balanced `nprobe=96,rerank_width=500` recorded `recall@k=0.9676`, `p50=10.9 ms`, `p95=11.8 ms`, `p99=13.3 ms`

### `quality-suite-manifest.json`

- command: copied from `review/30183-task31-suite-candidate-run/artifacts/suite-manifest.json`
- key result: quality suite run passed `quality-candidate-recall100-floor` with actual `0.992` and `quality-candidate-p50-budget-ms` with actual `12.9`

### `quality-results.jsonl`

- command: copied from `review/30183-task31-suite-candidate-run/artifacts/results.jsonl`
- key result: quality `nprobe=96,rerank_width=1000` recorded `recall@k=0.9920`, `p50=12.9 ms`, `p95=13.6 ms`, `p99=14.0 ms`
