# Task 61 HNSW Scan Frontier Overhead Benchmark

- head SHA at packet creation: `7928649b0`
- packet path: `benchmarks/task61-hnsw-scan-frontier-overhead`
- timestamp: `2026-05-24T23:00:59-07:00`
- profile: `10k-medium`, AWS Graviton
- access method: `ec_hnsw`
- build reloptions: `m=16`, `ef_construction=128`, `build_source_column=source`
- sweep axis: `ec_hnsw.ef_search`
- sweep values: `[40, 64, 100, 128, 160, 200]`
- latency cache state: `post_recall_warm`
- isolated one-index-per-table: yes, per scale

## Intent

Measure the first Task 61 HNSW scan-path optimization against the cloud
baseline for the requested 10k, 50k, and 100k cells only. The code change
removes a per-expansion hash set from graph-prefetch block deduplication and
removes a duplicate scan from queued-frontier removal.

## Suite

- configs: `suite-10k.json`, `suite-50k.json`, `suite-100k.json`
- cleanup config: `drop-loaded-tables.json`
- artifact dir: `artifacts/`
- install command:

```sh
target/release/ecaz cloud install \
  --profile 10k-medium \
  --git-ref diskann-aws-optimization \
  --skip-extension-recreate \
  --timeout 3600 \
  --log-file benchmarks/task61-hnsw-scan-frontier-overhead/artifacts/cloud-install-10k-medium.log
```

- split run pattern:

```sh
target/release/ecaz cloud bench \
  --profile 10k-medium \
  --config benchmarks/task61-hnsw-scan-frontier-overhead/suite-10k.json \
  --log-file benchmarks/task61-hnsw-scan-frontier-overhead/artifacts/cloud-bench-10k-medium-10k.log
```

## Planned Artifacts

| Artifact | Purpose |
| --- | --- |
| `artifacts/cloud-install-10k-medium.log` | branch-head install on the live host |
| `artifacts/cloud-bench-10k-medium-*.log` | per-scale cloud suite driver logs |
| `artifacts/precheck-host.log` | PostgreSQL settings, CPU, and memory precheck |
| `artifacts/suite-manifest.json` | normalized suite step manifest from the latest split run |
| `artifacts/results.jsonl` | normalized result rows from the latest split run |
| `artifacts/load-*-hnsw-default.log` | per-scale load/build logs |
| `artifacts/recall-*-hnsw-default.log` | per-scale recall logs |
| `artifacts/latency-*-hnsw-default.log` | per-scale latency logs |
| `artifacts/storage-*-hnsw-default.log` | per-scale storage logs |
| `artifacts/explain-*-hnsw-ef200.*` | per-scale explain SQL and output |
| `artifacts/drop-task61-optimized-loaded-tables.log` | cleanup after optimized runs |

## Results

Pending.
