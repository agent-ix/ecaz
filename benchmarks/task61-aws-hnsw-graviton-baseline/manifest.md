# Task 61 AWS HNSW Graviton Baseline

- head SHA at packet creation: `035b9412dfde1a1bc3ab1bb8855c168b77ace583`
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
- artifact dir: `artifacts/`
- command after install:

```sh
target/release/ecaz cloud bench \
  --profile 10k-medium \
  --config benchmarks/task61-aws-hnsw-graviton-baseline/suite.json \
  --log-file benchmarks/task61-aws-hnsw-graviton-baseline/artifacts/cloud-bench-10k-medium.log
```

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

Pending suite execution.
