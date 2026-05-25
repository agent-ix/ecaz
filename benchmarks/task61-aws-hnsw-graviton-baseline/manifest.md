# Task 61 AWS HNSW Graviton Baseline

- head SHA at packet creation: `035b9412dfde1a1bc3ab1bb8855c168b77ace583`
- packet setup commit: `617414f48`
- packet path: `benchmarks/task61-aws-hnsw-graviton-baseline`
- timestamp: `2026-05-24T20:16:24-07:00`
- profile: `10k-medium`, AWS Graviton, live host from `ecaz cloud status`
- database host: `10.42.1.58`, instance `i-0d46840c00f93dbb6`
- bucket: `ecaz-cloud-10k-medium-a02e4aea`
- snapshot: `snap-0b72153293b0b749b`
- access method: `ec_hnsw`
- build reloptions: `m=16`, `ef_construction=128`, `build_source_column=source`
- sweep axis: `ec_hnsw.ef_search`
- latency cache state: `post_recall_warm`
- isolated one-index-per-table: yes, per scale

## Intent

This packet is the first Task 61 cloud baseline. It deliberately starts with
10k, 50k, and 100k DBpedia/OpenAI3 cells before attempting 1M, so the first run
can prove HNSW build time, disk use, and memory headroom on the live low-cost
Graviton host.

The 1M cell is deferred until the first baseline artifacts show whether HNSW
build is cost-valid on `10k-medium`. If 1M is attempted, add a second checked-in
suite config or a resume config under this packet and cite its artifact root
here.

## Suite

- config: `suite.json`
- executed split configs: `suite-10k.json`, `suite-50k.json`,
  `suite-100k.json`
- cleanup configs: `drop-50k-partial.json`, `drop-loaded-tables.json`
- artifact dir: `artifacts/`
- original combined command:

```sh
target/release/ecaz cloud bench \
  --profile 10k-medium \
  --config benchmarks/task61-aws-hnsw-graviton-baseline/suite.json \
  --log-file benchmarks/task61-aws-hnsw-graviton-baseline/artifacts/cloud-bench-10k-medium.log
```

The combined suite was split because the cloud runner failed before execution
when transferring the full config through SSM. The split configs preserve the
same HNSW settings and packet-local artifact paths while keeping each SSM
payload small enough to run.

## Required Install

Before running the suite, install the current branch head on the live host so
latency rows include the checked-in `cache_state` field:

```sh
target/release/ecaz cloud install \
  --profile 10k-medium \
  --git-ref diskann-aws-optimization \
  --skip-extension-recreate \
  --timeout 3600 \
  --log-file benchmarks/task61-aws-hnsw-graviton-baseline/artifacts/cloud-install-10k-medium.log
```

## Planned Artifacts

| Artifact | Purpose |
| --- | --- |
| `artifacts/cloud-install-10k-medium.log` | branch-head install on the live host |
| `artifacts/cloud-bench-10k-medium.log` | cloud suite driver log |
| `artifacts/precheck-host.log` | PostgreSQL settings, CPU, and memory precheck |
| `artifacts/suite-manifest.json` | normalized suite step manifest |
| `artifacts/results.jsonl` | normalized result rows |
| `artifacts/load-*-hnsw-default.log` | per-scale load/build logs |
| `artifacts/recall-*-hnsw-default.log` | per-scale recall logs |
| `artifacts/latency-*-hnsw-default.log` | per-scale latency logs |
| `artifacts/storage-*-hnsw-default.log` | per-scale storage logs |
| `artifacts/explain-*-hnsw-ef200.*` | per-scale explain SQL and output |

## Results

Baseline execution completed for 10k, 50k, and 100k. The successful cloud runs
were copied back into packet-local S3 snapshots:

| Scale | Config | S3 run | Packet snapshot |
| --- | --- | --- | --- |
| 10k | `suite-10k.json` | `20260525T035246Z` | `artifacts/s3-10k-035246/` |
| 50k | `suite-50k.json` | `20260525T045436Z` | `artifacts/s3-50k-045436/` |
| 100k | `suite-100k.json` | `20260525T050401Z` | `artifacts/s3-100k-050401/` |

Failed/cleanup runs are retained where they explain host state:

| Run | Purpose |
| --- | --- |
| `artifacts/s3-50k-failed-045311/` | partial 50k table caused HNSW NULL-column build failure before cleanup |
| `artifacts/s3-100k-failed-050110/` | first 100k load failed after Postgres stopped accepting socket connections under disk pressure |
| `artifacts/cloud-cleanup-after-100k-failure.log` | after failed 100k staging cleanup, data volume was `99%` used with `2.0 GiB` free |
| `artifacts/cloud-cleanup-after-100k-success.log` | after successful 100k and staging cleanup, data volume was `98%` used with `2.5 GiB` free |

### Load / Build

| Scale | Total load | HNSW build phase |
| --- | ---: | ---: |
| 10k | 1.09 s | not split out by loader |
| 50k | 76.95 s | 53.29 s |
| 100k | 118.92 s | 65.07 s |

### Recall@10

| Scale | ef40 | ef64 | ef100 | ef128 | ef160 | ef200 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 0.8845 | 0.9305 | 0.9445 | 0.9645 | 0.9645 | 0.9700 |
| 50k | 0.8700 | 0.8985 | 0.9155 | 0.9190 | 0.9250 | 0.9315 |
| 100k | 0.7325 | 0.8065 | 0.8610 | 0.8845 | 0.9040 | 0.9120 |

### Latency

Mean / p50 / p95 / p99, `concurrency=1`, `iterations=200`,
`cache_state=post_recall_warm`.

| Scale | ef40 | ef64 | ef100 | ef128 | ef160 | ef200 |
| --- | --- | --- | --- | --- | --- | --- |
| 10k | 1.11 / 1.02 / 1.54 / 1.84 ms | 1.53 / 1.41 / 2.12 / 2.83 ms | 2.10 / 1.99 / 2.86 / 3.73 ms | 2.55 / 2.43 / 3.28 / 4.42 ms | 3.03 / 2.91 / 3.75 / 4.92 ms | 3.65 / 3.54 / 4.35 / 6.32 ms |
| 50k | 1.36 / 1.31 / 1.73 / 2.54 ms | 1.88 / 1.85 / 2.30 / 2.95 ms | 2.53 / 2.52 / 3.05 / 3.48 ms | 3.06 / 3.03 / 3.70 / 3.95 ms | 3.65 / 3.64 / 4.34 / 4.64 ms | 4.40 / 4.35 / 5.20 / 5.44 ms |
| 100k | 2.08 / 1.52 / 2.45 / 9.98 ms | 2.22 / 2.14 / 2.99 / 3.91 ms | 3.08 / 2.97 / 4.23 / 5.26 ms | 3.69 / 3.56 / 4.82 / 5.75 ms | 4.43 / 4.28 / 5.49 / 8.23 ms | 5.35 / 5.18 / 6.66 / 8.80 ms |

The 100k ef40 p99 has one visible outlier (`max=64.4 ms`); same-head repeat is
needed before treating that tail as a stable scan-path signal.

### Storage

| Scale | Rows | Total | HNSW index | HNSW index per row |
| --- | ---: | ---: | ---: | ---: |
| 10k | 10,000 | 172.9 MiB | 13.0 MiB | 1366.4 B |
| 50k | 50,000 | 863.9 MiB | 65.1 MiB | 1365.6 B |
| 100k | 100,000 | 1.7 GiB | 130.2 MiB | 1365.4 B |

## 1M Disposition

1M was deferred on `10k-medium`. This is an explicit capacity blocker, not an
HNSW code conclusion:

- after the first failed 100k attempt and staging cleanup, the 100 GiB data
  volume was `99%` used with `2.0 GiB` free
  (`artifacts/cloud-cleanup-after-100k-failure.log`);
- after dropping earlier Task 61 tables, rerunning 100k successfully, and
  cleaning staging, the same volume was still `98%` used with `2.5 GiB` free
  (`artifacts/cloud-cleanup-after-100k-success.log`);
- the successful 100k cell alone reports `1.7 GiB` total table/index footprint,
  and the retained DBpedia/Task 59 data leaves insufficient room for 1M staging,
  load, and HNSW build on this 100 GiB profile.

The next 1M-capable step is host/storage cleanup beyond the Task 61 scratch
surface or a larger Graviton profile/volume. Do not infer a 1M build-time
limit from this packet.

## First-Pass Read

At 100k, HNSW does not reach Task 59 DiskANN's best recall band within the
default `ef_search <= 200` sweep. At roughly comparable recall, DiskANN is
still faster and smaller:

- HNSW ef200: recall@10 `0.9120`, p50 `5.18 ms`, p95 `6.66 ms`,
  p99 `8.80 ms`, index `130.2 MiB`;
- DiskANN L64: recall@10 `0.9165`, p50 `1.81 ms`, p95 `2.36 ms`,
  p99 `2.56 ms`, index `46.1 MiB`.

The first follow-up should not be invasive HNSW code. The evidence points first
to benchmark/profile work: create enough disk headroom for 1M and widen the
100k HNSW `ef_search` sweep above 200 to find the high-recall curve before
choosing between scan-path profiling and Graviton/profile tuning.
