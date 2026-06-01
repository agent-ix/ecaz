# Artifact Manifest: Row-Budgeted Routing

- head SHA: `043c283f4fdfb39f50c870e66737905b5d657260`
- task bucket: `reviews/task-79/`
- packet path: `reviews/task-79/003-row-budgeted-routing/`
- timestamp: `2026-06-01T10:56:27-07:00`
- lane: Intel-local PG18, 100k real corpus, 200 query rows
- fixture: `target/real-corpus/staged-task50/`
- storage format: `rabitq`
- rerank mode: `rerank_width=25`, exact heap rerank enabled
- isolated surface: existing benchmark database, `task79_spire_candidate_surface`
- index surface: one table/index pair, `task79_surface_100k_corpus` and `task79_surface_100k_idx`

## Suite Config

- `../suite-rabitq-row-budget.json`
- config SHA256: `d5e412b9a2d656204be64bac516988be5ce6398f6d88dbaa561e9f99dd099fa9`

The suite used `ecaz bench suite`; no ad hoc sweeper was added.

## Code Under Review

Code commit `043c283f4fdfb39f50c870e66737905b5d657260` adds:

- session GUC `ec_spire.max_routed_candidate_rows`, default `0` disabled
- scan-plan plumbing for an optional routed candidate row budget
- post-route leaf filtering by exact leaf assignment counts from object headers
- `ecaz bench spire-pipeline --max-routed-candidate-rows`
- `ecaz bench suite` support for `max_routed_candidate_rows`
- focused scan/options/CLI tests

The budget is applied after ordered leaf routes are produced and before leaf
payload reads/prefetch. It always keeps at least one leaf, so a single large
leaf can exceed the target budget.

## Commands

Install current extension into PG18:

```sh
target/debug/ecaz dev install ecaz-pg-test --pg 18 --log-file reviews/task-79/003-row-budgeted-routing/artifacts/install-current-ecaz-pg18.log
```

Restart PG18:

```sh
/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/003-row-budgeted-routing/artifacts/pg18-restart.log restart -m fast
```

Audit:

```sh
target/debug/ecaz --log-file reviews/task-79/003-row-budgeted-routing/artifacts/suite-audit.log --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite audit --config reviews/task-79/003-row-budgeted-routing/suite-rabitq-row-budget.json
```

Dry run:

```sh
target/debug/ecaz --log-file reviews/task-79/003-row-budgeted-routing/artifacts/suite-dry-run.log --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/003-row-budgeted-routing/suite-rabitq-row-budget.json --manifest-output reviews/task-79/003-row-budgeted-routing/artifacts/suite-dry-run-manifest.json
```

Run:

```sh
target/debug/ecaz --log-file reviews/task-79/003-row-budgeted-routing/artifacts/suite-run.log --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/003-row-budgeted-routing/suite-rabitq-row-budget.json --manifest-output reviews/task-79/003-row-budgeted-routing/artifacts/suite-manifest.json --results-output reviews/task-79/003-row-budgeted-routing/artifacts/results.jsonl
```

Status and report:

```sh
target/debug/ecaz --log-file reviews/task-79/003-row-budgeted-routing/artifacts/suite-status.log --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite status --manifest reviews/task-79/003-row-budgeted-routing/artifacts/suite-manifest.json
target/debug/ecaz --log-file reviews/task-79/003-row-budgeted-routing/artifacts/suite-report.md --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite report --manifest reviews/task-79/003-row-budgeted-routing/artifacts/suite-manifest.json --results-output reviews/task-79/003-row-budgeted-routing/artifacts/report-results.jsonl
```

Focused validation:

```sh
script -q -e -c "cargo test max_routed_candidate_rows --no-default-features --features pg18" reviews/task-79/003-row-budgeted-routing/artifacts/test-max-routed-candidate-rows.log
script -q -e -c "cargo test caps_routed_candidate_rows --no-default-features --features pg18" reviews/task-79/003-row-budgeted-routing/artifacts/test-caps-routed-candidate-rows.log
script -q -e -c "cargo test -p ecaz-cli expands_spire_pipeline_with_production_profile" reviews/task-79/003-row-budgeted-routing/artifacts/test-cli-spire-pipeline-row-budget.log
```

## Suite Status

- `suite-audit.log`: audit passed, 8 steps.
- `suite-status.log`: completed `8`, failed `0`, skipped `0`, dry-run `0`, missing artifacts `0`, stale `0`.
- `suite-report.md`: parsed report emitted, with `report-results.jsonl`.
- focused validation logs: all three commands passed.

## Matrix

All rows used:

- `storage_format=rabitq`
- `boundary_replica_count=0`
- `top_graph_enabled=1`
- `top_graph_degree=32`
- `top_graph_build_list_size=100`
- `top_graph_search_list_size=256`
- adaptive nprobe off
- `max_candidate_rows`: default
- `rerank_width=25`

| step | nlists | fanout | max routed rows/query | nprobe sweep |
| --- | ---: | ---: | ---: | --- |
| `pipeline-100k-rabitq-n256-f16-b0-tg256-row26k` | 256 | 16 | 26,000 | 128, 192, 256 |
| `pipeline-100k-rabitq-n256-f16-b0-tg256-row36k` | 256 | 16 | 36,000 | 128, 192, 256 |
| `pipeline-100k-rabitq-n256-f16-b0-tg256-row52k` | 256 | 16 | 52,000 | 128, 192, 256 |
| `pipeline-100k-rabitq-n512-f16-b0-tg256-row26k` | 512 | 16 | 26,000 | 128, 192, 256 |
| `pipeline-100k-rabitq-n512-f16-b0-tg256-row36k` | 512 | 16 | 36,000 | 128, 192, 256 |

## Key Results

Baseline is the accepted Task 79 packet 001 reproduction of the Task 78 RaBitQ
high-recall point: `nlists=128`, `fanout=8`, `nprobe=96`, `candidate_sum=15,506,227`,
`route_sum=19,200`, p50 `61.234 ms`, recall@10 `0.9975`.

| nlists | fanout | max rows/query | nprobe | leaf route sum | candidates | candidates/query | p50 | p95 | recall@10 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 128 | 8 | disabled | 96 | 19,200 | 15,506,227 | 77,531 | 61.234 ms | 76.701 ms | 0.9975 |
| 256 | 16 | 26,000 | 128 | 13,270 | 5,252,750 | 26,264 | 47.380 ms | 54.908 ms | 0.9910 |
| 256 | 16 | 26,000 | 192 | 13,270 | 5,252,750 | 26,264 | 65.288 ms | 79.479 ms | 0.9975 |
| 256 | 16 | 26,000 | 256 | 13,270 | 5,252,750 | 26,264 | 82.708 ms | 98.227 ms | 1.0000 |
| 256 | 16 | 36,000 | 128 | 18,316 | 7,250,965 | 36,255 | 48.684 ms | 60.293 ms | 0.9910 |
| 256 | 16 | 36,000 | 192 | 18,316 | 7,250,965 | 36,255 | 67.415 ms | 75.301 ms | 0.9975 |
| 256 | 16 | 36,000 | 256 | 18,316 | 7,250,965 | 36,255 | 83.726 ms | 91.441 ms | 1.0000 |
| 256 | 16 | 52,000 | 128 | 25,445 | 10,015,302 | 50,077 | 47.482 ms | 54.569 ms | 0.9910 |
| 256 | 16 | 52,000 | 192 | 26,538 | 10,455,918 | 52,280 | 65.592 ms | 78.677 ms | 0.9975 |
| 256 | 16 | 52,000 | 256 | 26,538 | 10,455,918 | 52,280 | 84.614 ms | 97.553 ms | 1.0000 |
| 512 | 16 | 26,000 | 128 | 24,698 | 5,147,209 | 25,736 | 39.216 ms | 45.148 ms | 0.9645 |
| 512 | 16 | 26,000 | 192 | 25,116 | 5,231,408 | 26,157 | 48.636 ms | 57.204 ms | 0.9840 |
| 512 | 16 | 26,000 | 256 | 25,116 | 5,231,408 | 26,157 | 59.326 ms | 68.403 ms | 0.9940 |
| 512 | 16 | 36,000 | 128 | 25,600 | 5,337,119 | 26,686 | 39.101 ms | 45.082 ms | 0.9645 |
| 512 | 16 | 36,000 | 192 | 34,759 | 7,223,304 | 36,117 | 49.462 ms | 57.209 ms | 0.9840 |
| 512 | 16 | 36,000 | 256 | 34,788 | 7,228,848 | 36,144 | 60.154 ms | 66.955 ms | 0.9940 |

No measured row meets all Task 79 gates:

- recall floor: within `0.5 pp` of `0.9975`
- candidate gate: `<=5.2M` over 200 queries
- p50 target: at least `25%` better than the Task 78 high-recall point or `<=45 ms`

## Routing Diagnostics

The row cap reduces the placed/prefetched leaf route surface, but it does not
avoid generating the full requested route frontier first.

Examples from `report-results.jsonl`:

- `n256/row26k/nprobe128`: routing `route_sum=25,600`, placement `route_sum=13,270`, candidates `5,252,750`.
- `n256/row26k/nprobe192`: routing `route_sum=38,400`, placement `route_sum=13,270`, candidates `5,252,750`.
- `n256/row26k/nprobe256`: routing `route_sum=51,200`, placement `route_sum=13,270`, candidates `5,252,750`.
- `n512/row26k/nprobe128`: routing `route_sum=25,600`, placement `route_sum=24,698`, candidates `5,147,209`.
- `n512/row26k/nprobe256`: routing `route_sum=51,200`, placement `route_sum=25,116`, candidates `5,231,408`.

This explains the main outcome: the code directly reduces selected/scored
candidate rows, but high-recall rows still pay high-nprobe routing cost before
the post-route row filter runs.

## Decision

This is a real candidate-surface reduction slice, not a retained-candidate
cutoff. It confirms that row budgeting can collapse the selected row surface
from `15.5M` toward the Task 79 candidate gate, but the post-route placement
point is too late to satisfy the latency gate at high recall.

The next slice should push the row budget into route generation/top-graph
expansion itself, so routing can stop once enough estimated leaf rows have been
selected instead of first producing `nprobe` leaves and filtering afterward.

## Artifacts

- `suite-rabitq-row-budget.json`: checked-in suite configuration.
- `suite-audit.log`: suite audit output.
- `suite-dry-run.log`, `suite-dry-run-manifest.json`: dry-run evidence.
- `suite-run.log`, `suite-manifest.json`, `results.jsonl`: full suite run.
- `suite-status.log`, `suite-report.md`, `report-results.jsonl`: completion and parsed report.
- `install-current-ecaz-pg18.log`, `pg18-restart.log`: deployed extension evidence.
- `test-*.log`: focused validation logs.
- `precheck-existing-task79-surface.log`: existing PG18 surface and extension evidence.
- `rebuild-100k-rabitq-*.log`: rebuild timing for each geometry setting.
- `pipeline-100k-rabitq-*.log`: human-readable pipeline tables.
- `funnel-100k-rabitq-*.jsonl`: per-query candidate-funnel rows.
