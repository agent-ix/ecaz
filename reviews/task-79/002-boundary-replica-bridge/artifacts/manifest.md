# Artifact Manifest: Boundary Replica Bridge

- head SHA: `6ca06bf7b4743d4c4ca588633917bc480e937c52`
- task bucket: `reviews/task-79/`
- packet path: `reviews/task-79/002-boundary-replica-bridge/`
- timestamp: `2026-06-01T09:45:10-07:00`
- lane: Intel-local PG18, 100k real corpus, 200 query rows
- fixture: `target/real-corpus/staged-task50/`
- storage format: `rabitq`
- rerank mode: `rerank_width=25`, exact heap rerank enabled
- isolated surface: existing benchmark database, `task79_spire_candidate_surface`
- index surface: one table/index pair, `task79_surface_100k_corpus` and `task79_surface_100k_idx`

## Suite Config

- `../suite-rabitq-boundary-bridge.json`
- config SHA256 from `suite-report.md`: `5834aab8fa5b0e8cceae9d11a87edd95ee6347a9980a7ccbb590a176729878ac`

The suite used `ecaz bench suite`; no ad hoc sweeper was added.

## Commands

Audit:

```sh
target/debug/ecaz --log-file reviews/task-79/002-boundary-replica-bridge/artifacts/suite-audit.log --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite audit --config reviews/task-79/002-boundary-replica-bridge/suite-rabitq-boundary-bridge.json
```

Dry run:

```sh
target/debug/ecaz --log-file reviews/task-79/002-boundary-replica-bridge/artifacts/suite-dry-run.log --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/002-boundary-replica-bridge/suite-rabitq-boundary-bridge.json --manifest-output reviews/task-79/002-boundary-replica-bridge/artifacts/suite-dry-run-manifest.json
```

Run:

```sh
target/debug/ecaz --log-file reviews/task-79/002-boundary-replica-bridge/artifacts/suite-run.log --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/002-boundary-replica-bridge/suite-rabitq-boundary-bridge.json --manifest-output reviews/task-79/002-boundary-replica-bridge/artifacts/suite-manifest.json --results-output reviews/task-79/002-boundary-replica-bridge/artifacts/results.jsonl
```

Status and report:

```sh
target/debug/ecaz bench suite status --manifest reviews/task-79/002-boundary-replica-bridge/artifacts/suite-manifest.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --log-file reviews/task-79/002-boundary-replica-bridge/artifacts/suite-status.log
target/debug/ecaz bench suite report --manifest reviews/task-79/002-boundary-replica-bridge/artifacts/suite-manifest.json --results-output reviews/task-79/002-boundary-replica-bridge/artifacts/report-results.jsonl --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --log-file reviews/task-79/002-boundary-replica-bridge/artifacts/suite-report.md
```

## Suite Status

- `suite-audit.log`: audit passed, 9 steps.
- `suite-status.log`: completed `9`, failed `0`, skipped `0`, dry-run `0`, missing artifacts `0`, stale `0`.
- `suite-report.md`: parsed report emitted, with `report-results.jsonl`.

## Matrix

All rows used:

- `storage_format=rabitq`
- `top_graph_enabled=1`
- `top_graph_degree=32`
- `top_graph_build_list_size=100`
- `top_graph_search_list_size=128`
- adaptive nprobe off
- `max_candidate_rows`: default
- `rerank_width=25`

| step | nlists | fanout | boundary replicas | nprobe sweep |
| --- | ---: | ---: | ---: | --- |
| `pipeline-100k-rabitq-n256-f16-b0-tg128` | 256 | 16 | 0 | 64, 96, 128 |
| `pipeline-100k-rabitq-n512-f16-b1-tg128` | 512 | 16 | 1 | 64, 96, 128 |
| `pipeline-100k-rabitq-n512-f16-b2-tg128` | 512 | 16 | 2 | 64, 96 |
| `pipeline-100k-rabitq-n1024-f32-b1-tg128` | 1024 | 32 | 1 | 96, 128 |

## Key Results

The `nlists=256` bridge improves the `nlists=512` recall gap, but still does
not satisfy both Task 79 gates:

| nlists | fanout | boundary replicas | nprobe | route_sum | candidates | candidates/query | p50 | p95 | recall@10 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 128 | 8 | 0 | 96 | 19,200 | 15,506,227 | 77,531 | 61.234 ms | 76.701 ms | 0.9975 |
| 256 | 16 | 0 | 64 | 12,800 | 5,067,941 | 25,340 | 28.420 ms | 31.161 ms | 0.9580 |
| 256 | 16 | 0 | 96 | 19,200 | 7,582,639 | 37,913 | 37.339 ms | 43.043 ms | 0.9805 |
| 256 | 16 | 0 | 128 | 25,600 | 10,072,003 | 50,360 | 46.143 ms | 53.004 ms | 0.9910 |
| 512 | 16 | 1 | 64 | 12,800 | 5,327,226 | 26,636 | 41.098 ms | 48.934 ms | 0.9580 |
| 512 | 16 | 1 | 96 | 19,200 | 8,042,299 | 40,211 | 51.444 ms | 60.475 ms | 0.9750 |
| 512 | 16 | 1 | 128 | 25,600 | 10,691,154 | 53,456 | 61.441 ms | 74.225 ms | 0.9860 |
| 512 | 16 | 2 | 64 | 12,800 | 8,046,327 | 40,232 | 50.386 ms | 58.327 ms | 0.9690 |
| 512 | 16 | 2 | 96 | 19,200 | 12,110,960 | 60,555 | 67.142 ms | 80.452 ms | 0.9845 |
| 1024 | 32 | 1 | 96 | 19,200 | 4,300,115 | 21,501 | 55.114 ms | 65.469 ms | 0.9635 |
| 1024 | 32 | 1 | 128 | 25,600 | 5,692,251 | 28,461 | 60.871 ms | 72.591 ms | 0.9735 |

Baseline row is the accepted Task 79 packet 001 reproduction of the Task 78
RaBitQ high-recall point.

No measured row meets both gates:

- recall floor: within `0.5 pp` of `0.9975`
- candidate gate: `<=5.2M` over 200 queries

## Build Cost

Boundary replicas recover some recall by duplicating boundary rows, but the
build-time cost is severe in this fixture:

| config | suite duration | build notice total_ms | draft_leaf_rows_ms |
| --- | ---: | ---: | ---: |
| n256/f16/b0 | 11.084 s | 11,011 | 43 |
| n512/f16/b1 | 144.192 s | 144,106 | 128,136 |
| n512/f16/b2 | 146.358 s | 146,271 | 129,068 |
| n1024/f32/b1 | 295.858 s | 295,735 | 272,690 |

## Leaf-Density Evidence

The suite funnel artifacts emit per-query selected-leaf summaries. Exact
per-selected-leaf p50/p99 rows are not emitted by the current suite artifact;
the table below uses exact `candidate_sum / route_sum` plus query-level
selected-leaf summary fields from the funnel JSONL.

| nlists | fanout | boundary replicas | nprobe | candidates/route | p50 query leaf mean | p95 query leaf p95 | max query leaf max |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 256 | 16 | 0 | 96 | 394.93 | 398.53 | 857 | 1,227 |
| 256 | 16 | 0 | 128 | 393.44 | 393.98 | 849 | 1,227 |
| 512 | 16 | 1 | 96 | 418.87 | 419.83 | 987 | 1,362 |
| 512 | 16 | 2 | 96 | 630.78 | 632.77 | 1,417 | 2,060 |
| 1024 | 32 | 1 | 96 | 223.96 | 225.61 | 616 | 1,222 |

## Routing Diagnostics

For all measured nprobe rows, final route count is still fixed by the requested
leaf route count:

- nprobe64: `route_sum=12,800`
- nprobe96: `route_sum=19,200`
- nprobe128: `route_sum=25,600`

The `n256` bridge therefore recovers recall by scanning more rows than the
candidate gate permits. Boundary replicas recover only limited recall while
increasing row surface, p50 latency, and build cost. This rules out simple
boundary replication as the accuracy-preserving Task 79 solution.

## Artifacts

- `suite-rabitq-boundary-bridge.json`: checked-in suite configuration.
- `suite-audit.log`: suite audit output.
- `suite-dry-run.log`, `suite-dry-run-manifest.json`: dry-run evidence.
- `suite-run.log`, `suite-manifest.json`, `results.jsonl`: full suite run.
- `suite-status.log`, `suite-report.md`, `report-results.jsonl`: completion and parsed report.
- `precheck-existing-task79-surface.log`: existing PG18 surface and extension evidence.
- `rebuild-100k-rabitq-*.log`: rebuild timing for each geometry/replica setting.
- `pipeline-100k-rabitq-*.log`: human-readable pipeline tables.
- `funnel-100k-rabitq-*.jsonl`: per-query candidate-funnel rows.

## Decision

This packet addresses the packet 001 reviewer feedback for the missing
`nlists=256` bridge and makes a controlled boundary-replica attempt. It does
not produce a passing recipe. The next implementation slice should avoid
scoring every row inside selected high-recall leaves, either through row-budgeted
routing with persisted row-count estimates or a leaf-local subpartition/pruning
layer. Simply adding replicas makes the candidate surface larger.
