# Artifact Manifest: Task 33 HNSW M5 Reference Refresh Run

- head SHA: `e7efb2f3eac486036fa1ff139c3214df4c7e44e3`
- task bucket: `reviews/task-33`
- packet path: `reviews/task-33/002-30213-task33-hnsw-m5-reference-refresh-run`
- lane: HNSW M5 real50K worker refresh
- timestamp: `2026-05-30T17:03:24Z`
- hardware: Apple Silicon local development host; see `hardware-fingerprint.log`
- PostgreSQL: PG18 on socket `/Users/peter/.pgrx`, port `28818`
- fixture: `data/task31_m5_dbpedia_staged/ec_hnsw_real_50k_*`
- isolation/shared-table surface: one Task 33 corpus prefix with one index per
  requested worker count during the raw SQL sweep
- storage format: `ec_hnsw`, `m = 16`, `ef_construction = 128`,
  `build_source_column = source`
- query surface: recall, latency, and storage were measured against the
  `workers=4` build, renamed from `task33_m5_hnsw_real50k_w4_idx` to
  `task33_m5_hnsw_real50k_m16_idx`

## Artifacts

| Artifact | Purpose |
| --- | --- |
| `../task33-hnsw-m5-reference-refresh-run.packet.json` | suite config |
| `task33_hnsw_m5_worker_sweep.sql` | packet-local SQL for requested worker-count build sweep |
| `audit.log` | suite audit output |
| `suite-manifest.json` | executed suite manifest and per-step status |
| `results.jsonl` | parsed suite result rows plus worker-sweep summary rows |
| `load-hnsw-real50k-setup.log` | fixture load and setup index log |
| `task33_hnsw_m5_worker_sweep.log` | requested worker sweep log |
| `recall-hnsw-real50k-best-worker-index.log` | recall sweep on workers=4 index |
| `latency-hnsw-real50k-best-worker-index.log` | latency sweep on workers=4 index |
| `storage-hnsw-real50k-best-worker-index.log` | storage report after sweep |
| `truth-real50k-k10.json` | recall ground-truth cache |
| `hardware-fingerprint.log` | local host and OS fingerprint |

## Commands

```sh
/Users/peter/.cargo/bin/ecaz --log-file reviews/task-33/002-30213-task33-hnsw-m5-reference-refresh-run/artifacts/audit.log bench suite audit --config reviews/task-33/002-30213-task33-hnsw-m5-reference-refresh-run/task33-hnsw-m5-reference-refresh-run.packet.json
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-33/002-30213-task33-hnsw-m5-reference-refresh-run/task33-hnsw-m5-reference-refresh-run.packet.json --manifest-output reviews/task-33/002-30213-task33-hnsw-m5-reference-refresh-run/artifacts/suite-manifest.json
```

## Results

The suite completed all `5` steps with no failed steps in
`suite-manifest.json`.

Requested worker sweep on real50K:

| requested workers | build wall seconds | PG parallel-worker counter delta | index bytes |
| --- | ---: | ---: | ---: |
| 1 | 11.402388 | 0 | 68280320 |
| 2 | 8.363440 | 0 | 68280320 |
| 4 | 6.055808 | 0 | 68280320 |
| 8 | 7.728653 | 0 | 68280320 |

The release-installed extension does not expose the `tests.*` debug helper
functions used by older pgrx-test benchmark SQL. The packet therefore records
requested workers, build wall time, index size, and PostgreSQL's database-level
parallel-worker counter delta. That PG counter stayed `0` for every build, so
launched graph-worker count is not proved by this packet. Build-time memory HWM
is also `not_measured` because the worker sweep is a raw SQL suite step and
`ecaz dev sql` does not sample backend RSS.

Recall on the workers=4 index:

| ef_search | recall@10 | mean q-time |
| --- | ---: | ---: |
| 64 | 0.9400 | 1.22 ms |
| 128 | 0.9560 | 1.38 ms |
| 200 | 0.9710 | 1.88 ms |
| 400 | 0.9800 | 3.28 ms |

Latency on the workers=4 index:

| ef_search | p50 | p95 | p99 | mean | memory HWM |
| --- | ---: | ---: | ---: | ---: | --- |
| 64 | 0.92 ms | 1.17 ms | 1.48 ms | 0.96 ms | not sampled (`0` samples) |
| 128 | 1.40 ms | 1.76 ms | 2.15 ms | 1.45 ms | not sampled (`0` samples) |
| 200 | 1.92 ms | 2.43 ms | 2.87 ms | 1.96 ms | not sampled (`0` samples) |
| 400 | 3.29 ms | 4.09 ms | 4.47 ms | 3.35 ms | not sampled (`0` samples) |

Storage after retaining the workers=4 canonical index and the other worker
indexes:

- rows: `50000`
- table: `796.7 MiB`
- indexes: `262.6 MiB`
- total: `1.0 GiB`
- canonical HNSW index size: `65.1 MiB` / `1365.6 B` per row

## Phase 2 Recommendation

This 50K curve repeats the Task 26 shape: requested worker increases improve
local build time up to the `4` worker point, but `8` requested workers regresses
and the evidence does not justify more worker-threshold tuning. The next Task
33 checkpoint should move to Phase 2 design, with the staged/offline bulk build
lane as the default recommendation unless a follow-up profiler proves the
remaining in-Postgres queue/drain boundary is the dominant cost.

## Validation

- `ecaz bench suite audit`: passed, `5` steps
- `ecaz bench suite run`: completed `5` steps with no failures
- `jq empty` on suite config, `suite-manifest.json`, and `results.jsonl`: passed
- `git diff --check`: passed
