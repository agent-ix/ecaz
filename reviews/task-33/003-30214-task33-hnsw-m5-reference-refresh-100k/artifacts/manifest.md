# Artifact Manifest: Task 33 HNSW M5 Reference Refresh 100K

- head SHA: `a9363795adeb07affa6ee3dc26734397a21f7da7`
- task bucket: `reviews/task-33`
- packet path: `reviews/task-33/003-30214-task33-hnsw-m5-reference-refresh-100k`
- lane: HNSW M5 real100K larger-corpus worker refresh
- timestamp: `2026-05-30T17:11:16Z`
- hardware: Apple Silicon local development host; see `hardware-fingerprint.log`
- PostgreSQL: PG18 on socket `/Users/peter/.pgrx`, port `28818`
- fixture: `data/task31_m5_dbpedia_staged/ec_hnsw_real_100k_*`
- isolation/shared-table surface: one Task 33 corpus prefix with one index per
  requested worker count during the raw SQL sweep
- storage format: `ec_hnsw`, `m = 16`, `ef_construction = 128`,
  `build_source_column = source`
- query surface: recall, latency, and storage were measured against the
  `workers=4` build, renamed from `task33_m5_hnsw_real100k_w4_idx` to
  `task33_m5_hnsw_real100k_m16_idx`

## Artifacts

| Artifact | Purpose |
| --- | --- |
| `../task33-hnsw-m5-reference-refresh-100k.packet.json` | suite config |
| `task33_hnsw_m5_worker_sweep_100k.sql` | packet-local SQL for requested worker-count build sweep |
| `audit.log` | suite audit output |
| `suite-manifest.json` | executed suite manifest and per-step status |
| `results.jsonl` | parsed suite result rows plus worker-sweep summary rows |
| `load-hnsw-real100k-setup.log` | fixture load and setup index log |
| `task33_hnsw_m5_worker_sweep.log` | requested worker sweep log |
| `recall-hnsw-real100k-best-worker-index.log` | recall sweep on workers=4 index |
| `latency-hnsw-real100k-best-worker-index.log` | latency sweep on workers=4 index |
| `storage-hnsw-real100k-best-worker-index.log` | storage report after sweep |
| `truth-real100k-k10.json` | recall ground-truth cache |
| `hardware-fingerprint.log` | local host and OS fingerprint |

## Commands

```sh
/Users/peter/.cargo/bin/ecaz --log-file reviews/task-33/003-30214-task33-hnsw-m5-reference-refresh-100k/artifacts/audit.log bench suite audit --config reviews/task-33/003-30214-task33-hnsw-m5-reference-refresh-100k/task33-hnsw-m5-reference-refresh-100k.packet.json
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-33/003-30214-task33-hnsw-m5-reference-refresh-100k/task33-hnsw-m5-reference-refresh-100k.packet.json --manifest-output reviews/task-33/003-30214-task33-hnsw-m5-reference-refresh-100k/artifacts/suite-manifest.json
```

## Results

The suite completed all `5` steps with no failed steps in
`suite-manifest.json`.

Requested worker sweep on real100K:

| requested workers | build wall seconds | PG parallel-worker counter delta | index bytes |
| --- | ---: | ---: | ---: |
| 1 | 98.046380 | 0 | 136544256 |
| 2 | 24.837646 | 0 | 136544256 |
| 4 | 18.469675 | 0 | 136544256 |
| 8 | 21.854627 | 0 | 136544256 |

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
| 64 | 0.8350 | 1.71 ms |
| 128 | 0.9205 | 2.14 ms |
| 200 | 0.9575 | 2.88 ms |
| 400 | 0.9775 | 4.81 ms |

Latency on the workers=4 index:

| ef_search | p50 | p95 | p99 | mean | memory HWM |
| --- | ---: | ---: | ---: | ---: | --- |
| 64 | 1.32 ms | 2.46 ms | 3.18 ms | 1.44 ms | not sampled (`0` samples) |
| 128 | 2.04 ms | 3.40 ms | 4.95 ms | 2.23 ms | not sampled (`0` samples) |
| 200 | 2.81 ms | 4.19 ms | 5.98 ms | 2.94 ms | not sampled (`0` samples) |
| 400 | 4.68 ms | 6.63 ms | 8.78 ms | 4.87 ms | not sampled (`0` samples) |

Storage after retaining the workers=4 canonical index and the other worker
indexes:

- rows: `100000`
- table: `1.6 GiB`
- indexes: `525.2 MiB`
- total: `2.1 GiB`
- canonical HNSW index size: `130.2 MiB` / `1365.4 B` per row

## Phase 2 Recommendation

The 100K curve makes the stop condition clearer than the 50K run: requested
workers improve build time up to `4`, but `8` regresses. The only remaining
worker-threshold conclusion is "use the 4-worker surface for reference rows";
it is not a design direction. Task 33 should move to Phase 2 and prefer the
offline/staged bulk build ADR lane unless a focused profile proves direct DSM
ingestion is the dominant remaining in-Postgres cost.

## Validation

- `ecaz bench suite audit`: passed, `5` steps
- `ecaz bench suite run`: completed `5` steps with no failures
- `jq empty` on suite config, `suite-manifest.json`, and `results.jsonl`: passed
- `git diff --check`: passed
