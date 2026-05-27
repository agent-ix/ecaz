# Task 63 HNSW RaBitQ Format Benchmark Packet

- task: `plan/tasks/63-hnsw-rabitq-storage-format.md`
- branch: `task/60-diskann-rabitq`
- suite config: `benchmarks/task63-hnsw-rabitq-format/suite.json`
- artifact directory: `benchmarks/task63-hnsw-rabitq-format/artifacts/`

## Purpose

This packet is the `ecaz bench suite` measurement plan for the HNSW
`storage_format` comparison required by Task 63. It keeps TurboQuant,
PqFastScan, and RaBitQ on the same benchmark host, dataset, query set, HNSW
`m`, `ef_construction`, `ef_search` sweep, cache-state label, and PG18 socket.

## Matrix

| size | profile | storage formats | recall/latency sweep | cache state |
| --- | --- | --- | --- | --- |
| 50k | `ec_real_50k` | `turboquant`, `pq_fastscan`, `rabitq` | `40,64,100,128,160,200` | `post_recall_warm` |
| 100k | `ec_real_100k` | `turboquant`, `pq_fastscan`, `rabitq` | `40,64,100,128,160,200` | `post_recall_warm` |

The suite uses the DBpedia-derived 1536d fixture used by adjacent HNSW and
DiskANN benchmark packets and stages it under
`/var/lib/pgsql/18/datasets/staged-task63-hnsw-rabitq/`.

## Acceptance Checks

- Build and scan: all three formats load with `ec_hnsw`, `m=16`,
  `ef_construction=128`, and the suite's HNSW built-in
  `build_source_column=source` reloption.
- Recall: report recall@10 for all three formats at matched `ef_search`.
- Latency: report p50/p95/p99 latency for all three formats at matched
  `ef_search` after the recall pass has warmed the cache.
- Storage: record `ecaz bench storage` output for all six prefixes.
- Decision: record the recommended RaBitQ operating point, or mark the format
  experimental/shelved if the measured recall/storage tradeoff is not useful.

## Commands

Dry-run validation:

```sh
ecaz \
  --log-file benchmarks/task63-hnsw-rabitq-format/artifacts/suite-dry-run.log \
  bench suite run \
  --config benchmarks/task63-hnsw-rabitq-format/suite.json \
  --dry-run \
  --manifest-output benchmarks/task63-hnsw-rabitq-format/artifacts/suite-manifest.json \
  --results-output benchmarks/task63-hnsw-rabitq-format/artifacts/results.jsonl
```

Full run on the benchmark host:

```sh
ecaz \
  --log-file benchmarks/task63-hnsw-rabitq-format/artifacts/suite-run.log \
  bench suite run \
  --config benchmarks/task63-hnsw-rabitq-format/suite.json \
  --manifest-output benchmarks/task63-hnsw-rabitq-format/artifacts/suite-manifest.json \
  --results-output benchmarks/task63-hnsw-rabitq-format/artifacts/results.jsonl
```

Report extraction after the full run:

```sh
ecaz \
  --log-file benchmarks/task63-hnsw-rabitq-format/artifacts/suite-report.log \
  bench suite report \
  --manifest benchmarks/task63-hnsw-rabitq-format/artifacts/suite-manifest.json \
  --results-output benchmarks/task63-hnsw-rabitq-format/artifacts/results-report.jsonl
```

Final packet evidence should include:

- `artifacts/suite-run.log`
- `artifacts/suite-report.log`
- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl` from `suite run`
- `artifacts/results-report.jsonl` from `suite report`
- the six storage logs for 50k and 100k
- the six recall logs for 50k and 100k
- the six latency logs for 50k and 100k
- `artifacts/precheck-host.log`

## Current State

The suite is scaffolded for the required 50k and 100k Task 63 matrix and was
locally audited and dry-run validated. The checks wrote:

- `artifacts/suite-audit.log`
- `artifacts/suite-dry-run.log`
- `artifacts/suite-manifest.json`

Measurement artifacts are pending until this branch is installed on a PG18
benchmark host with the staged DBpedia fixtures.
