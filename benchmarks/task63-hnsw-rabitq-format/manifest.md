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
- Build time: use the six `load-*` logs and the `load_timing` /
  `build_index` rows in `results.jsonl` or `results-report.jsonl`.
- Storage: record `ecaz bench storage` output for all six prefixes.
- Decision: record the recommended RaBitQ operating point, or mark the format
  experimental/shelved if the measured recall/storage tradeoff is not useful.

## Host Scope

Final Task 63 benchmark evidence should come from the newer Intel and m5 laptop
benchmark hosts. The older 64GB AMD workstation may be used for local
smoke/tuning only; do not cite AMD-local partial runs as the Task 63 acceptance
matrix.

On the older local workstation, keep any tuning run HNSW-only and limited to
10k or 50k rows. Leave the 100k acceptance matrix and any larger runs to agents
on faster benchmark hosts. Do not use this host for 1M, IVF, DiskANN, SPIRE, or
cross-lane benchmark work.

Record each publishable host in this packet before citing its numbers:

- host label (`newer-intel` or `m5-laptop`)
- CPU model, core/thread count, memory, OS, and PostgreSQL socket/port
- extension HEAD SHA and install command
- whether datasets were freshly fetched/prepared or reused from a prior packet
- suite command, report command, and exact result artifact paths

Use the same checked-in `suite.json` on both benchmark hosts. Host-local path
adjustments should be made only when required by that host's PostgreSQL layout,
and the manifest must state the path delta next to the host result summary.

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
- the six load logs for 50k and 100k; these are the durable source for
  per-format build times via `[loader] built ... in ...`
- the six storage logs for 50k and 100k
- the six recall logs for 50k and 100k
- the six latency logs for 50k and 100k
- `artifacts/precheck-host.log`

## Result Summary Template

Copy this section once per publishable benchmark host after a full suite run.

### Host: `<newer-intel|m5-laptop>`

| Field | Value |
| --- | --- |
| HEAD SHA | `<sha>` |
| Captured | `<timestamp and timezone>` |
| Host | `<hostname>` |
| CPU | `<model>` |
| Memory | `<GiB>` |
| OS | `<name/version/arch>` |
| PostgreSQL | `18, socket <path>, port <port>` |
| Extension build | `<command>` |
| Suite status | `<completed>/<total> succeeded, failed=<n>` |
| Dataset source | `<fresh fetch/prepare or reused path>` |

Recall, latency, build time, and storage tables should be generated from
`artifacts/results-report.jsonl` plus the six `load-*` and `storage-*` logs.
The Task 63 operating-point decision must cite the host label and the matched
`ef_search` row that justifies the recommendation.

## Current State

The suite is scaffolded for the required 50k and 100k Task 63 matrix and was
locally audited and dry-run validated. The checks wrote:

- `artifacts/suite-audit.log`
- `artifacts/suite-dry-run.log`
- `artifacts/suite-manifest.json`

Measurement artifacts are pending until this branch is installed on a PG18
benchmark host with the staged DBpedia fixtures. AMD-local baseline/tuning logs
are intentionally excluded from this packet's final acceptance evidence.
