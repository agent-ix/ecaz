# Artifact Manifest: Candidate Surface Baseline

- head SHA: `f3db11275a2c61a19a43eef1bcccb0cc56c78ea7`
- task bucket: `reviews/task-79/`
- packet path: `reviews/task-79/001-candidate-surface-baseline/`
- timestamp: `2026-06-01T09:08:54-07:00`
- lane: Intel-local PG18, 100k real corpus, 200 query rows
- fixture: `target/real-corpus/staged-task50/`
- storage format: `rabitq`
- rerank mode: `rerank_width=25`, exact heap rerank enabled
- isolated surface: one benchmark database, `task79_spire_candidate_surface`
- index surface: one table/index pair, `task79_surface_100k_corpus` and `task79_surface_100k_idx`

## Suite Config

- `../suite-rabitq-geometry.json`
- config SHA256 from `suite-report.md`: `70b022b30edca4ffc60e798f5fa27cd032e31513d8ded78585c6c1bc2c467ba5`

The suite used `ecaz bench suite`; no ad hoc sweeper was added.

## Commands

Audit:

```sh
target/debug/ecaz --log-file reviews/task-79/001-candidate-surface-baseline/artifacts/suite-audit.log --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite audit --config reviews/task-79/001-candidate-surface-baseline/suite-rabitq-geometry.json
```

Dry run:

```sh
target/debug/ecaz --log-file reviews/task-79/001-candidate-surface-baseline/artifacts/suite-dry-run.log --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/001-candidate-surface-baseline/suite-rabitq-geometry.json --manifest-output reviews/task-79/001-candidate-surface-baseline/artifacts/suite-dry-run-manifest.json
```

Run:

```sh
target/debug/ecaz --log-file reviews/task-79/001-candidate-surface-baseline/artifacts/suite-run.log --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/001-candidate-surface-baseline/suite-rabitq-geometry.json --manifest-output reviews/task-79/001-candidate-surface-baseline/artifacts/suite-manifest.json --results-output reviews/task-79/001-candidate-surface-baseline/artifacts/results.jsonl
```

Status and report:

```sh
target/debug/ecaz bench suite status --manifest reviews/task-79/001-candidate-surface-baseline/artifacts/suite-manifest.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --log-file reviews/task-79/001-candidate-surface-baseline/artifacts/suite-status.log
target/debug/ecaz bench suite report --manifest reviews/task-79/001-candidate-surface-baseline/artifacts/suite-manifest.json --results-output reviews/task-79/001-candidate-surface-baseline/artifacts/report-results.jsonl --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --log-file reviews/task-79/001-candidate-surface-baseline/artifacts/suite-report.md
```

## Suite Status

- `suite-audit.log`: audit passed, 13 steps.
- `suite-status.log`: completed `13`, failed `0`, skipped `0`, dry-run `0`, missing artifacts `0`, stale `0`.
- `suite-report.md`: parsed report emitted, with `report-results.jsonl`.

## Matrix

All rows used:

- `top_graph_enabled=1`
- `top_graph_degree=32`
- `top_graph_build_list_size=100`
- `top_graph_search_list_size=128`, except the exact Task 78-style baseline row at `96`
- `boundary_replica_count=0`
- adaptive nprobe off
- `max_candidate_rows`: default

| step | nlists | fanout | top_graph_search | nprobe sweep |
| --- | ---: | ---: | ---: | --- |
| `pipeline-100k-rabitq-n128-f8-tg96` | 128 | 8 | 96 | 96 |
| `pipeline-100k-rabitq-n128-f8-tg128` | 128 | 8 | 128 | 32, 48, 64, 96, 128 |
| `pipeline-100k-rabitq-n512-f16-tg128` | 512 | 16 | 128 | 32, 48, 64, 96, 128 |
| `pipeline-100k-rabitq-n1024-f32-tg128` | 1024 | 32 | 128 | 32, 48, 64, 96, 128 |
| `pipeline-100k-rabitq-n2048-f64-tg128` | 2048 | 64 | 128 | 32, 48, 64, 96, 128 |

## Key Results

The exact Task 78-style baseline reproduced the candidate explosion:

- `nlists=128`, `recursive_fanout=8`, `top_graph_search_list_size=96`, `nprobe=96`
- `route_sum=19,200`
- `candidate_sum=15,506,227`
- retained after rerank `5,000`
- returned to k `2,000`
- recall@10 `0.9975`
- p50 `61.234 ms`, p95 `76.701 ms`, p99 `85.266 ms`
- local object bytes `12,642,962,128`

The geometry sweep reduced candidates but did not preserve high recall:

| nlists | fanout | nprobe | route_sum | candidates | candidates/query | p50 | p95 | recall@10 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 128 | 8 | 96 | 19,200 | 15,506,227 | 77,531 | 61.234 ms | 76.701 ms | 0.9975 |
| 512 | 16 | 96 | 19,200 | 4,008,683 | 20,043 | 35.474 ms | 39.979 ms | 0.9420 |
| 512 | 16 | 128 | 25,600 | 5,337,119 | 26,686 | 40.210 ms | 46.082 ms | 0.9645 |
| 1024 | 32 | 96 | 19,200 | 2,152,562 | 10,763 | 46.472 ms | 51.554 ms | 0.9265 |
| 1024 | 32 | 128 | 25,600 | 2,851,080 | 14,255 | 50.018 ms | 56.548 ms | 0.9425 |
| 2048 | 64 | 96 | 19,200 | 1,148,089 | 5,740 | 76.914 ms | 86.756 ms | 0.8875 |
| 2048 | 64 | 128 | 25,600 | 1,519,734 | 7,599 | 80.326 ms | 91.084 ms | 0.9095 |

No measured geometry row meets both gates:

- recall floor: within `0.5 pp` of `0.9975`
- candidate gate: `<=5.2M` over 200 queries

## Leaf-Density Evidence

The suite funnel artifacts emit per-query selected-leaf summaries. Exact
per-selected-leaf p50/p99 rows are not emitted by the current suite artifact;
the table below uses exact `candidate_sum / route_sum` plus query-level
selected-leaf summary fields from the funnel JSONL.

| nlists | fanout | nprobe | candidates/route | p50 query leaf mean | p95 query leaf p95 | p99 query leaf max | max query leaf max |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 128 | 8 | 96 | 807.62 | 808.76 | 1564 | 2200 | 2200 |
| 512 | 16 | 96 | 208.79 | 209.63 | 549 | 690 | 690 |
| 1024 | 32 | 96 | 112.11 | 113.61 | 317 | 646 | 646 |
| 2048 | 64 | 96 | 59.80 | 58.56 | 198 | 385 | 416 |

## Routing Diagnostics

At `nprobe=96`, routing selects `19,200` final routes in all measured
geometries. Candidate count falls only because selected leaves are smaller.
The finer geometries therefore directly reduce the candidate surface, but
current routing cannot preserve recall at those smaller leaf sizes.

The `n2048/f64/tg128` row also shows that reducing candidates alone is not a
sufficient latency solution: p50 rises to `76.914 ms` at `nprobe=96` while
recall drops to `0.8875`.

## Artifacts

- `suite-rabitq-geometry.json`: checked-in suite configuration.
- `suite-audit.log`: suite audit output.
- `suite-dry-run.log`, `suite-dry-run-manifest.json`: dry-run evidence.
- `suite-run.log`, `suite-manifest.json`, `results.jsonl`: full suite run.
- `suite-status.log`, `suite-report.md`, `report-results.jsonl`: completion and parsed report.
- `precheck-host-and-extension.log`: PG18 host/extension evidence.
- `load-100k-rabitq-n128-f8-tg96.log`: corpus load and baseline index build.
- `rebuild-100k-rabitq-*.log`: rebuild timing for each geometry.
- `pipeline-100k-rabitq-*.log`: human-readable pipeline tables.
- `funnel-100k-rabitq-*.jsonl`: per-query candidate-funnel rows.

## Decision

This packet rules out "make leaves smaller" as a complete Task 79 solution.
It proves the row surface can be cut from `15.5M` to `4.0M`, `2.15M`, or
`1.15M`, but every candidate-gate row misses the high-recall target by a large
margin. The next slice should directly address accuracy-preserving candidate
selection: either improve recursive/top-graph routing quality at lower row
surfaces, add row-budgeted route selection with better diagnostics, or add a
leaf-local pruning layer that avoids scoring every row in a selected high-recall
leaf.
