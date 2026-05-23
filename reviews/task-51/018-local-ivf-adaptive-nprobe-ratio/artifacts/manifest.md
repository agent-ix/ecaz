# Review Packet 018 Artifact Manifest

- head SHA: `7e215f5edf9bc4e8dd906bc2d36f861ae9f00b61`
- task bucket: `reviews/task-51/018-local-ivf-adaptive-nprobe-ratio`
- benchmark packet: `benchmarks/task51-local-ivf-adaptive-nprobe-ratio/`
- lane: local PG18 IVF/RaBitQ adaptive nprobe ratio follow-up
- fixture: preserved isolated prefix `task51_local_990k_ivf_rabitq1_n1024_w50`
- storage format: `rabitq`, `quant_bits=1`
- rerank mode: `heap_f32`, `rerank_width=50`
- isolated one-index-per-table surface: yes
- suite runner: `ecaz bench suite`
- suite config: `benchmarks/task51-local-ivf-adaptive-nprobe-ratio/suite.json`
- suite status: `completed=8 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- vchord / pgvectorscale: not run

## Key Artifacts

- `benchmarks/task51-local-ivf-adaptive-nprobe-ratio/manifest.md`
- `benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/suite-run.log`
- `benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/suite-status.log`
- `benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/suite-report.log`
- `benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/results.jsonl`
- `benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/results-report.jsonl`

## Commands

See the benchmark packet manifest for the full command list. Load-bearing commands:

```text
target/debug/ecaz --log-file benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/suite-audit.log bench suite audit --config benchmarks/task51-local-ivf-adaptive-nprobe-ratio/suite.json
target/debug/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/suite-run.log bench suite run --config benchmarks/task51-local-ivf-adaptive-nprobe-ratio/suite.json --manifest-output benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/suite-status.log bench suite status --manifest benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/suite-report.log bench suite report --manifest benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/suite-manifest.json --results-output benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/results-report.jsonl
```

## Key Result Lines

Recall preserved across static and all ratio thresholds:

```text
static nprobe=64 recall@10=0.9570 recall_p10=0.8900 recall_worst=0.5000
ratio=2500 nprobe=64 recall@10=0.9570 recall_p10=0.8900 recall_worst=0.5000
ratio=10000 nprobe=64 recall@10=0.9570 recall_p10=0.8900 recall_worst=0.5000
ratio=50000 nprobe=64 recall@10=0.9570 recall_p10=0.8900 recall_worst=0.5000
static nprobe=128 recall@10=0.9750 recall_p10=0.9000 recall_worst=0.5000
ratio=2500 nprobe=128 recall@10=0.9750 recall_p10=0.9000 recall_worst=0.5000
ratio=10000 nprobe=128 recall@10=0.9750 recall_p10=0.9000 recall_worst=0.5000
ratio=50000 nprobe=128 recall@10=0.9750 recall_p10=0.9000 recall_worst=0.5000
```

Latency did not show a useful promotion signal:

```text
static nprobe=64 p50=282.5 ms p95=336.4 ms p99=356.9 ms
ratio=2500 nprobe=64 p50=301.3 ms p95=362.1 ms p99=389.2 ms
ratio=10000 nprobe=64 p50=290.7 ms p95=355.9 ms p99=362.3 ms
ratio=50000 nprobe=64 p50=286.5 ms p95=343.0 ms p99=362.6 ms
static nprobe=128 p50=557.2 ms p95=646.9 ms p99=659.6 ms
ratio=2500 nprobe=128 p50=592.0 ms p95=681.8 ms p99=715.5 ms
ratio=10000 nprobe=128 p50=557.8 ms p95=637.7 ms p99=709.4 ms
ratio=50000 nprobe=128 p50=551.6 ms p95=628.0 ms p99=646.9 ms
```
