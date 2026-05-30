# Task 33 HNSW M5 Reference Refresh 100K

Reviewer: please review the locally feasible larger-corpus Task 33 M5 refresh.

## Scope

This packet extends packet 002 from real50K to real100K using the staged
`data/task31_m5_dbpedia_staged/ec_hnsw_real_100k_*` fixture. It does not change
runtime code.

The suite executed:

- fixture load/setup for `task33_m5_hnsw_real100k`
- raw SQL build sweep with requested workers `1/2/4/8`
- recall@10 sweep on the selected workers=4 index
- latency sweep on the selected workers=4 index
- storage report

## Key Results

Requested worker build wall time:

| requested workers | build wall seconds | index bytes |
| --- | ---: | ---: |
| 1 | 98.046380 | 136544256 |
| 2 | 24.837646 | 136544256 |
| 4 | 18.469675 | 136544256 |
| 8 | 21.854627 | 136544256 |

Recall on the workers=4 index:

| ef_search | recall@10 | mean q-time |
| --- | ---: | ---: |
| 64 | 0.8350 | 1.71 ms |
| 128 | 0.9205 | 2.14 ms |
| 200 | 0.9575 | 2.88 ms |
| 400 | 0.9775 | 4.81 ms |

Latency on the workers=4 index:

| ef_search | p50 | p95 | p99 |
| --- | ---: | ---: | ---: |
| 64 | 1.32 ms | 2.46 ms | 3.18 ms |
| 128 | 2.04 ms | 3.40 ms | 4.95 ms |
| 200 | 2.81 ms | 4.19 ms | 5.98 ms |
| 400 | 4.68 ms | 6.63 ms | 8.78 ms |

Storage: canonical workers=4 HNSW index is `130.2 MiB`, `1365.4 B` per row.

## Caveats

The release-installed extension does not expose the pgrx-test-only
`tests.ec_hnsw_debug_last_build_timing()` or
`tests.ec_hnsw_debug_parallel_graph_build_workers_launched()` helpers from the
scaffold. This packet uses wall-clock timing and
`pg_stat_get_db_parallel_workers_launched` deltas instead. The PG counter stayed
`0` for every build, so launched graph-worker count is not proved.

Build-time memory HWM is recorded as `not_measured`; the raw SQL suite step
does not have backend RSS sampling. Latency memory sampling also produced `0`
samples in this run.

Recall, latency, and storage are measured against the workers=4 build only.
The 1/2/8 builds contribute build-time and index-size evidence.

## Recommendation

This larger run confirms the packet 002 recommendation: stop worker-threshold
tuning and move Task 33 to Phase 2. I recommend the offline/staged bulk build
ADR lane as the next checkpoint, with direct DSM ingestion kept conditional on
a focused profile proving the queue/drain boundary is still the dominant cost.

## Validation

- `ecaz bench suite audit --config reviews/task-33/003-30214-task33-hnsw-m5-reference-refresh-100k/task33-hnsw-m5-reference-refresh-100k.packet.json`
- `ecaz bench suite run --config reviews/task-33/003-30214-task33-hnsw-m5-reference-refresh-100k/task33-hnsw-m5-reference-refresh-100k.packet.json --manifest-output reviews/task-33/003-30214-task33-hnsw-m5-reference-refresh-100k/artifacts/suite-manifest.json`

Follow-up local validation is recorded in the manifest.

