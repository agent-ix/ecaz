# Task 63 HNSW RaBitQ Format Benchmark Packet

- task: `plan/tasks/63-hnsw-rabitq-storage-format.md`
- branch: `task/60-diskann-rabitq`
- recommended host install head or newer: `f20e91c3494060ba64927bf9482112a3011438a0`
- minimum code source head: `36807d607606808717e0b645cde9b251d3fa2e23`
- Linux/newer-Intel suite config: `benchmarks/task63-hnsw-rabitq-format/suite.json`
- m5 laptop suite config: `benchmarks/task63-hnsw-rabitq-format/suite-m5.json`
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

The decision row must use measurements taken at or after source head
`36807d607606808717e0b645cde9b251d3fa2e23`. Earlier local AMD runs are useful
for trend diagnosis only. They predate the latest scalar 1-bit RaBitQ byte-LUT
scorer checkpoint and must not be used to make the final Task 63 ship/shelve
decision.

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
- checked-in suite config path, suite command, report command, and exact result
  artifact paths

The checked-in `suite.json` is the Linux/newer-Intel benchmark-host config. The
checked-in `suite-m5.json` is the m5 laptop config, with the same HNSW
50k/100k matrix, M5 socket path, M5 staged DBpedia paths, and separate
`artifacts/m5-laptop/` outputs. Do not run publishable Task 63 measurements
from an untracked host-local edit of either config. Cite the config path and
the generated `suite-manifest.json` SHA in each host summary.

Both publishable hosts should install the recommended host install head or a
newer branch head. The minimum code source head is the oldest acceptable
benchmark code point because it contains the post-local-smoke 1-bit scalar
byte-LUT scorer change; newer heads include docs, fixtures, and handoff
cleanups. If only one host is available, keep the second host section pending
rather than backfilling it with AMD-local output.

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

Full run on the m5 laptop:

```sh
ecaz \
  --log-file benchmarks/task63-hnsw-rabitq-format/artifacts/m5-laptop/suite-run.log \
  bench suite run \
  --config benchmarks/task63-hnsw-rabitq-format/suite-m5.json \
  --manifest-output benchmarks/task63-hnsw-rabitq-format/artifacts/m5-laptop/suite-manifest.json \
  --results-output benchmarks/task63-hnsw-rabitq-format/artifacts/m5-laptop/results.jsonl
```

Report extraction after the full run:

```sh
ecaz \
  --log-file benchmarks/task63-hnsw-rabitq-format/artifacts/suite-report.log \
  bench suite report \
  --manifest benchmarks/task63-hnsw-rabitq-format/artifacts/suite-manifest.json \
  --results-output benchmarks/task63-hnsw-rabitq-format/artifacts/results-report.jsonl
```

Report extraction after the m5 full run:

```sh
ecaz \
  --log-file benchmarks/task63-hnsw-rabitq-format/artifacts/m5-laptop/suite-report.log \
  bench suite report \
  --manifest benchmarks/task63-hnsw-rabitq-format/artifacts/m5-laptop/suite-manifest.json \
  --results-output benchmarks/task63-hnsw-rabitq-format/artifacts/m5-laptop/results-report.jsonl
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

For the m5 laptop, use the corresponding paths under
`artifacts/m5-laptop/`.

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

### Decision Row

| Host | Size | Format | `ef_search` | recall@10 | p50 / p95 / p99 latency | build time | storage | Decision |
| --- | --- | --- | ---: | ---: | --- | ---: | ---: | --- |
| `<host>` | `<50k|100k>` | `rabitq` | `<n>` | `<value>` | `<ms>` | `<duration>` | `<bytes/row>` | `<recommend|experimental|shelve>` |

Recommend RaBitQ only if the publishable host results show a meaningful storage
win at a recall/latency point that is competitive with matched TurboQuant and
PqFastScan. If RaBitQ remains at storage parity with worse recall or latency,
record the format as experimental/shelved for HNSW and cite the matched rows.

## Current State

The suite is scaffolded for the required 50k and 100k Task 63 matrix and was
locally audited and dry-run validated. The checks wrote:

- `artifacts/suite-audit.log`
- `artifacts/suite-dry-run.log`
- `artifacts/suite-manifest.json`

Measurement artifacts are pending until this branch is installed on a PG18
benchmark host with the staged DBpedia fixtures. AMD-local baseline/tuning logs
are intentionally excluded from this packet's final acceptance evidence.

Local HNSW-only 10k/50k smoke packets already showed that binary RaBitQ search
codes fixed the earlier storage regression, but did not establish a useful
RaBitQ HNSW operating point on the older AMD workstation:

- `reviews/task-63/008-hnsw-rabitq-binary-search-code/` records the local 10k
  tuning snapshot and the binary 1-bit search-code change.
- `reviews/task-63/009-hnsw-rabitq-local-50k-smoke/` records the local 50k
  tuning snapshot.
- `reviews/task-63/010-rabitq-bits1-scalar-byte-lut/` adds the later common
  scalar 1-bit RaBitQ scorer improvement, so publishable host measurements
  must be rerun after that checkpoint before making the final decision.
- `reviews/task-63/011-hnsw-rabitq-benchmark-handoff/` through
  `reviews/task-63/017-hnsw-rabitq-handoff-head-wording/` record the final
  local handoff/status cleanup before faster-host measurement: benchmark
  manifest gating, user docs caveat, HNSW V4 RaBitQ on-disk fixture and upgrade
  matrix, reloption/spec docs, confirmation that non-1-bit RaBitQ prepared
  queries do not retain the 1-bit byte LUT, the final local handoff checkpoint,
  and the host-install-head wording correction.
