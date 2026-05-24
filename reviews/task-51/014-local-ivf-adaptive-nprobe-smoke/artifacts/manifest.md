# Artifact Manifest

- head SHA: `5cf94f0c8dad41b366a5ecc4f6a26c44df38a801`
- task bucket: `reviews/task-51`
- packet path: `reviews/task-51/014-local-ivf-adaptive-nprobe-smoke`
- benchmark packet: `benchmarks/task51-local-ivf-adaptive-nprobe/`
- lane: local PG18 IVF/RaBitQ adaptive nprobe smoke
- fixture: preserved 990k anchor corpus/index
- storage format: `rabitq`, `quant_bits=1`
- rerank mode: `heap_f32`, `rerank_width=50`
- adaptive thresholds: `1000`, `10000`, `100000` score-gap micros
- isolated one-index-per-table surface: yes, reused prefix `task51_local_990k_ivf_rabitq1_n1024_w50`
- timestamp: `2026-05-23T14:29:48Z`
- AWS: not used
- vchord / pgvectorscale: not used

## Review Artifacts

Primary artifacts live in benchmark packet `benchmarks/task51-local-ivf-adaptive-nprobe/`:

- `suite.json`
- `manifest.md`
- `request.md`
- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl`
- `artifacts/results-report.jsonl`
- `artifacts/suite-run-release.log`
- `artifacts/suite-status.log`
- `artifacts/suite-report.log`
- `artifacts/recall-*.log`
- `artifacts/latency-*.log`
- `artifacts/local-pgrx-install-release.log`
- `artifacts/cargo-build-ecaz-cli.log`

## Key Result Lines

```text
[suite:task51-local-ivf-adaptive-nprobe] completed=8 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0

static nprobe=64 recall@10=0.9570 recall_p10=0.8900 latency_p50=290.1 ms latency_p95=361.7 ms
adaptive gap=1000 nprobe=64 recall@10=0.9490 recall_p10=0.8000 latency_p50=225.8 ms latency_p95=336.4 ms
adaptive gap=10000 nprobe=64 recall@10=0.9570 recall_p10=0.8900 latency_p50=304.8 ms latency_p95=360.4 ms
adaptive gap=100000 nprobe=64 recall@10=0.9570 recall_p10=0.8900 latency_p50=289.1 ms latency_p95=348.9 ms
```
