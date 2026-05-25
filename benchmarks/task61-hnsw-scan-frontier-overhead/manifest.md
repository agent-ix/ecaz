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

Optimized execution completed for 10k and 50k. The 100k cell failed during
load because the `10k-medium` data volume ran out of space while encoding the
copied 100k corpus table.

| Scale | Config | S3 run | Packet snapshot |
| --- | --- | --- | --- |
| 10k | `suite-10k.json` | `20260525T061413Z` | `artifacts/s3-10k-061413/` |
| 50k | `suite-50k.json` | `20260525T061537Z` | `artifacts/s3-50k-061537/` |
| 100k | `suite-100k.json` | `20260525T061819Z` | failed before result rows |

### Load / Build

| Scale | Total load | HNSW build phase |
| --- | ---: | ---: |
| 10k | 8.21 s | 3.72 s |
| 50k | 69.63 s | 46.99 s |

### Recall@10

Recall is unchanged from the baseline, as expected for a scan-path constant
factor change over the same graph build settings.

| Scale | ef40 | ef64 | ef100 | ef128 | ef160 | ef200 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 0.8845 | 0.9305 | 0.9445 | 0.9645 | 0.9645 | 0.9700 |
| 50k | 0.8700 | 0.8985 | 0.9155 | 0.9190 | 0.9250 | 0.9315 |

### Latency

Mean / p50 / p95 / p99, `concurrency=1`, `iterations=200`,
`cache_state=post_recall_warm`.

| Scale | ef40 | ef64 | ef100 | ef128 | ef160 | ef200 |
| --- | --- | --- | --- | --- | --- | --- |
| 10k | 1.00 / 0.93 / 1.31 / 1.59 ms | 1.40 / 1.31 / 1.87 / 2.28 ms | 1.89 / 1.79 / 2.39 / 3.31 ms | 2.27 / 2.15 / 2.93 / 3.94 ms | 2.68 / 2.60 / 3.28 / 4.39 ms | 3.21 / 3.11 / 3.75 / 4.87 ms |
| 50k | 1.26 / 1.23 / 1.54 / 2.37 ms | 1.69 / 1.66 / 2.09 / 2.67 ms | 2.33 / 2.32 / 2.84 / 3.06 ms | 2.80 / 2.78 / 3.34 / 3.54 ms | 3.32 / 3.33 / 4.01 / 4.21 ms | 4.02 / 3.99 / 4.76 / 5.08 ms |

### Baseline Comparison

| Scale | ef_search | Baseline p50 / p95 / p99 | Optimized p50 / p95 / p99 |
| --- | ---: | --- | --- |
| 10k | 40 | 1.02 / 1.54 / 1.84 ms | 0.93 / 1.31 / 1.59 ms |
| 10k | 200 | 3.54 / 4.35 / 6.32 ms | 3.11 / 3.75 / 4.87 ms |
| 50k | 40 | 1.31 / 1.73 / 2.54 ms | 1.23 / 1.54 / 2.37 ms |
| 50k | 200 | 4.35 / 5.20 / 5.44 ms | 3.99 / 4.76 / 5.08 ms |

### 100k Disposition

The 100k optimized cell is capacity-blocked on this retained `10k-medium`
volume, not rejected on HNSW behavior. The SSM failure record is
`artifacts/ssm-100k-failure.json`; the key error was:

```text
ERROR: could not extend file "base/5/8860779": No space left on device
```

Cleanup after the failed 100k attempt removed the partial optimized tables and
task-scoped staging. The final scratch cleanup check reported `4.2 GiB` free on
`/var/lib/pgsql/18`, but the 100k corpus prepare alone used `3.2 GiB`, leaving
insufficient room for copy, encode, and HNSW build on this host.

## Host State

The host was paused after cleanup. See
`artifacts/cloud-status-after-pause.log`.

```text
profile:  10k-medium
state:    paused
cost:     ~$0.00/hr running, ~$8.00/mo retained storage
```
