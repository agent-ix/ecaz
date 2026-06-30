# Task 124 Packet 023 Artifact Manifest

- head SHA: `eb63775f285c61965f82fd224c2d46a043eef351`
- task bucket: `reviews/task-124/`
- packet: `reviews/task-124/023-tq-nprobe60-ab-matrix/`
- timestamp: `2026-06-29 20:01:42 PDT`
- runner: `ecaz bench suite`
- suite config: `artifacts/task124-tq-nprobe60-ab-10-50-100-suite.json`
- suite config sha256: `cc50851615807320a65b75ee1d8e62aeeeb420961bef026c490ce0e1576e0def`
- host/socket: `/Users/peter/.pgrx`, port `28818`
- database: `tqvector_bench`
- surface: isolated one-index-per-table prefixes
- storage format: `coarse_rerank`
- index/quantizer: `ec_ivf`, `coarse_format=rabitq`, `rerank_format=turboquant`
- TQ shape: `rerank_width=75`, `rerank_group_width=50`, `stage2_final_rerank_width=15`
- comparison axis: `nprobe=60` vs `nprobe=64`
- corpora: `data/staged-current/ec_real_10k_*`, `ec_real_50k_*`, `ec_real_100k_*`
- generated truth caches: produced under `artifacts/nprobe60-ab-matrix/truth-*.json` but intentionally not committed

## Commands

```text
target/release/ecaz --log-file reviews/task-124/023-tq-nprobe60-ab-matrix/artifacts/suite-audit.log bench suite audit --config reviews/task-124/023-tq-nprobe60-ab-matrix/artifacts/task124-tq-nprobe60-ab-10-50-100-suite.json
target/release/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-124/023-tq-nprobe60-ab-matrix/artifacts/suite-run.log bench suite run --config reviews/task-124/023-tq-nprobe60-ab-matrix/artifacts/task124-tq-nprobe60-ab-10-50-100-suite.json --manifest-output reviews/task-124/023-tq-nprobe60-ab-matrix/artifacts/suite-manifest.json --results-output reviews/task-124/023-tq-nprobe60-ab-matrix/artifacts/results.jsonl
target/release/ecaz --log-file reviews/task-124/023-tq-nprobe60-ab-matrix/artifacts/suite-status.log bench suite status --manifest reviews/task-124/023-tq-nprobe60-ab-matrix/artifacts/suite-manifest.json
target/release/ecaz --log-file reviews/task-124/023-tq-nprobe60-ab-matrix/artifacts/suite-report.log bench suite report --manifest reviews/task-124/023-tq-nprobe60-ab-matrix/artifacts/suite-manifest.json --results-output reviews/task-124/023-tq-nprobe60-ab-matrix/artifacts/report-results.jsonl
```

## Artifacts

- `artifacts/task124-tq-nprobe60-ab-10-50-100-suite.json`
- `artifacts/suite-audit.log`
- `artifacts/suite-run.log`
- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl`
- `artifacts/suite-status.log`
- `artifacts/suite-report.log`
- `artifacts/report-results.jsonl`
- `artifacts/nprobe60-ab-matrix/load-10k-tq-w75-g50-final15-nprobe60-ab.log`
- `artifacts/nprobe60-ab-matrix/recall-10k-tq-w75-g50-final15-nprobe60-ab.log`
- `artifacts/nprobe60-ab-matrix/latency-10k-tq-w75-g50-final15-nprobe60-ab.log`
- `artifacts/nprobe60-ab-matrix/storage-10k-tq-w75-g50-final15-nprobe60-ab.log`
- `artifacts/nprobe60-ab-matrix/load-50k-tq-w75-g50-final15-nprobe60-ab.log`
- `artifacts/nprobe60-ab-matrix/recall-50k-tq-w75-g50-final15-nprobe60-ab.log`
- `artifacts/nprobe60-ab-matrix/latency-50k-tq-w75-g50-final15-nprobe60-ab.log`
- `artifacts/nprobe60-ab-matrix/storage-50k-tq-w75-g50-final15-nprobe60-ab.log`
- `artifacts/nprobe60-ab-matrix/load-100k-tq-w75-g50-final15-nprobe60-ab.log`
- `artifacts/nprobe60-ab-matrix/recall-100k-tq-w75-g50-final15-nprobe60-ab.log`
- `artifacts/nprobe60-ab-matrix/latency-100k-tq-w75-g50-final15-nprobe60-ab.log`
- `artifacts/nprobe60-ab-matrix/storage-100k-tq-w75-g50-final15-nprobe60-ab.log`

## Key Results

Suite status:

```text
[suite:task124-tq-nprobe60-ab-10-50-100-suite] completed=12 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Recall and latency:

| scale | nprobe | recall@k | ndcg@k | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 60 | 1.0000 | 1.0000 | 1.13 ms | 1.22 ms | 1.29 ms |
| 10k | 64 | 1.0000 | 1.0000 | 1.20 ms | 1.32 ms | 1.41 ms |
| 50k | 60 | 0.9980 | 1.0000 | 4.15 ms | 4.50 ms | 4.99 ms |
| 50k | 64 | 0.9980 | 1.0000 | 4.58 ms | 4.83 ms | 5.08 ms |
| 100k | 60 | 1.0000 | 1.0000 | 8.65 ms | 9.01 ms | 9.35 ms |
| 100k | 64 | 1.0000 | 1.0000 | 8.98 ms | 9.36 ms | 9.92 ms |

Latency deltas for `nprobe=60` vs `64`:

| scale | p50 delta | p95 delta | p99 delta |
| --- | ---: | ---: | ---: |
| 10k | -0.07 ms | -0.10 ms | -0.12 ms |
| 50k | -0.43 ms | -0.33 ms | -0.09 ms |
| 100k | -0.33 ms | -0.35 ms | -0.57 ms |

TQ scorer counters remained SIMD-only:

| scale | nprobe | quant | isa | scalar candidates | candidates |
| --- | ---: | --- | --- | ---: | ---: |
| 10k | 60 | turboquant | neon | 0 | 7500 |
| 10k | 64 | turboquant | neon | 0 | 7500 |
| 50k | 60 | turboquant | neon | 0 | 7500 |
| 50k | 64 | turboquant | neon | 0 | 7500 |
| 100k | 60 | turboquant | neon | 0 | 7500 |
| 100k | 64 | turboquant | neon | 0 | 7500 |

Coarse frontier counters fell with `nprobe=60`:

| scale | nprobe | coarse candidates |
| --- | ---: | ---: |
| 10k | 60 | 936366 |
| 10k | 64 | 1000000 |
| 50k | 60 | 4525933 |
| 50k | 64 | 5000000 |
| 100k | 60 | 9556278 |
| 100k | 64 | 10000000 |

Storage is unchanged by the nprobe choice:

| scale | ec_ivf index size | per row |
| --- | ---: | ---: |
| 10k | 10.9 MiB | 1143.6 B |
| 50k | 50.9 MiB | 1066.8 B |
| 100k | 100.8 MiB | 1057.2 B |

