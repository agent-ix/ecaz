# Task 81 Packet 004 Artifact Manifest

- head SHA: `4033f2983d43f745e759c4ce1b9f27a07e35e48a`
- branch: `task-81-spire-leaf-block-summary-format`
- task bucket: `reviews/task-81/004-local-nprobe-block-summary-gate/`
- timestamp: `2026-06-04T20:45:58-07:00`
- lane: local PG18
- database: `task79_spire_candidate_surface`
- host/port: `/home/peter/.pgrx`, `28818`
- surface isolation: isolated one-index-per-table surface copied to `task81_nprobe_100k_*`
- index: `task81_nprobe_100k_idx`
- storage format: `rabitq`
- index options: `lists=128`, `leaf_block_rows=16`, `top_graph_degree=32`, `top_graph_build_list_size=100`, `top_graph_search_list_size=256`, `recursive_fanout=8`, `boundary_replica_count=0`, `rerank_width=25`
- suite config: `reviews/task-81/004-local-nprobe-block-summary-gate/suite-local-nprobe-block-summary-gate.json`
- suite config SHA-256: `303f9b4137c5ab0a502270c419043d6ef6d8c3b039af211ad6e33df2ef69381d`
- query/truth shape: 100k local corpus, q200, k10
- global candidate cap: `1152`
- rerank mode: heap rerank width `25`

## Commands

Initial audit, before the isolated tg256 prepare step was added:

```sh
script -q -c "target/debug/ecaz bench suite audit --config reviews/task-81/004-local-nprobe-block-summary-gate/suite-local-nprobe-block-summary-gate.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818" reviews/task-81/004-local-nprobe-block-summary-gate/artifacts/suite-audit.log
```

Initial run failed because the reused `task79_surface_100k_idx` had `top_graph_search_list_size=96`, so requested `nprobe > 96` violated the route-count guard.

```sh
target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-81/004-local-nprobe-block-summary-gate/suite-local-nprobe-block-summary-gate.json --manifest-output reviews/task-81/004-local-nprobe-block-summary-gate/artifacts/suite-manifest.json --results-output reviews/task-81/004-local-nprobe-block-summary-gate/artifacts/results.jsonl --log-file reviews/task-81/004-local-nprobe-block-summary-gate/artifacts/suite-run.log
```

Accepted rerun after adding an isolated tg256 prepare step:

```sh
script -q -c "target/debug/ecaz bench suite audit --config reviews/task-81/004-local-nprobe-block-summary-gate/suite-local-nprobe-block-summary-gate.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818" reviews/task-81/004-local-nprobe-block-summary-gate/artifacts/suite-audit-rerun.log
```

```sh
target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-81/004-local-nprobe-block-summary-gate/suite-local-nprobe-block-summary-gate.json --manifest-output reviews/task-81/004-local-nprobe-block-summary-gate/artifacts/suite-manifest-rerun.json --results-output reviews/task-81/004-local-nprobe-block-summary-gate/artifacts/results-rerun.jsonl --log-file reviews/task-81/004-local-nprobe-block-summary-gate/artifacts/suite-run-rerun.log
```

```sh
target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite status --config reviews/task-81/004-local-nprobe-block-summary-gate/suite-local-nprobe-block-summary-gate.json --manifest reviews/task-81/004-local-nprobe-block-summary-gate/artifacts/suite-manifest-rerun.json --results reviews/task-81/004-local-nprobe-block-summary-gate/artifacts/results-rerun.jsonl --log-file reviews/task-81/004-local-nprobe-block-summary-gate/artifacts/suite-status-rerun.log
```

```sh
target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite report --config reviews/task-81/004-local-nprobe-block-summary-gate/suite-local-nprobe-block-summary-gate.json --manifest reviews/task-81/004-local-nprobe-block-summary-gate/artifacts/suite-manifest-rerun.json --results reviews/task-81/004-local-nprobe-block-summary-gate/artifacts/results-rerun.jsonl --output reviews/task-81/004-local-nprobe-block-summary-gate/artifacts/suite-report-rerun.log --jsonl-output reviews/task-81/004-local-nprobe-block-summary-gate/artifacts/suite-report-results-rerun.jsonl
```

## Artifacts

- `artifacts/suite-audit.log`: initial audit against reused tg96 index config.
- `artifacts/suite-run.log`: initial failed run; `nprobe=128` failed because `top_graph_search_list_size=96`.
- `artifacts/suite-audit-rerun.log`: accepted rerun audit; 3 steps passed.
- `artifacts/suite-manifest-rerun.json`: accepted rerun suite manifest.
- `artifacts/results-rerun.jsonl`: accepted rerun structured result stream.
- `artifacts/suite-run-rerun.log`: accepted rerun raw suite log.
- `artifacts/suite-status-rerun.log`: accepted rerun status; completed 3, failed 0, skipped 0.
- `artifacts/suite-report-rerun.log`: accepted rerun report.
- `artifacts/suite-report-results-rerun.jsonl`: accepted rerun parsed report rows.
- `artifacts/prepare-task81-local-nprobe-tg256-surface.log`: isolated surface prepare/build log.
- `artifacts/precheck-task81-local-nprobe-surface.log`: tg256 surface precheck log.
- `artifacts/pipeline-100k-rabitq-block-summary-global1152-nprobe-sweep.log`: accepted pipeline log.
- `artifacts/funnel-100k-rabitq-block-summary-global1152-nprobe-sweep.jsonl`: accepted funnel output.

## Key Results

| nprobe | effective_nprobe | route_sum | candidates | p50 ms | p95 ms | p99 ms | recall@10 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 96 | 96 | 19,200 | 3,672,619 | 32.212 | 36.028 | 37.564 | 0.9945 |
| 128 | 128 | 25,600 | 3,672,641 | 34.351 | 41.397 | 47.028 | 0.9965 |
| 160 | 128 | 25,600 | 3,672,641 | 34.720 | 44.833 | 53.428 | 0.9965 |
| 192 | 128 | 25,600 | 3,672,641 | 34.994 | 39.457 | 46.414 | 0.9965 |

The local gate remains under the Task 81 candidate target (`<=4.0M`) and p50 target (`<=45 ms`). `nprobe=128` is the next AWS 1M candidate because it improved local recall by `+0.0020` over nprobe96 while adding only 22 scored rows over q200. Requested nprobe values above 128 were clamped by `max_routing_expansions=128`, so they did not add breadth.
