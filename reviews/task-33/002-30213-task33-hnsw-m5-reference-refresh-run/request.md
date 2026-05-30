# Task 33 HNSW M5 Reference Refresh Run

Reviewer: please review the executed Task 33 M5 reference refresh packet.

## Scope

This packet runs the scaffold approved in packet 001 against the real50K HNSW
fixture. It does not change runtime code.

The suite executed:

- fixture load/setup for `task33_m5_hnsw_real50k`
- raw SQL build sweep with requested workers `1/2/4/8`
- recall@10 sweep on the selected workers=4 index
- latency sweep on the selected workers=4 index
- storage report

## Key Results

Requested worker build wall time:

| requested workers | build wall seconds | index bytes |
| --- | ---: | ---: |
| 1 | 11.402388 | 68280320 |
| 2 | 8.363440 | 68280320 |
| 4 | 6.055808 | 68280320 |
| 8 | 7.728653 | 68280320 |

Recall on the workers=4 index:

| ef_search | recall@10 | mean q-time |
| --- | ---: | ---: |
| 64 | 0.9400 | 1.22 ms |
| 128 | 0.9560 | 1.38 ms |
| 200 | 0.9710 | 1.88 ms |
| 400 | 0.9800 | 3.28 ms |

Latency on the workers=4 index:

| ef_search | p50 | p95 | p99 |
| --- | ---: | ---: | ---: |
| 64 | 0.92 ms | 1.17 ms | 1.48 ms |
| 128 | 1.40 ms | 1.76 ms | 2.15 ms |
| 200 | 1.92 ms | 2.43 ms | 2.87 ms |
| 400 | 3.29 ms | 4.09 ms | 4.47 ms |

Storage: canonical workers=4 HNSW index is `65.1 MiB`, `1365.6 B` per row.

## Caveats

The release-installed extension does not expose the pgrx-test-only
`tests.ec_hnsw_debug_last_build_timing()` or
`tests.ec_hnsw_debug_parallel_graph_build_workers_launched()` helpers from the
scaffold. I replaced those calls with per-build wall-clock timing and
`pg_stat_get_db_parallel_workers_launched` deltas. The PG counter stayed `0`
for every build, so this packet does not prove graph-worker launched counts.

Build-time memory HWM is recorded as `not_measured`; the raw SQL suite step
does not have backend RSS sampling. Latency memory sampling also produced `0`
samples in this run.

Recall, latency, and storage are measured against the workers=4 build only.
The 1/2/8 builds contribute build-time and index-size evidence.

## Recommendation

Do not continue worker-threshold tuning from this packet. The M5 curve improves
through requested workers=4 and regresses at requested workers=8, which matches
the Task 26 conclusion closely enough that Phase 2 design is the useful next
step. I recommend the offline/staged bulk build lane unless profiling shows the
remaining in-Postgres queue/drain boundary is still dominant enough to justify
direct DSM ingestion first.

## Validation

- `ecaz bench suite audit --config reviews/task-33/002-30213-task33-hnsw-m5-reference-refresh-run/task33-hnsw-m5-reference-refresh-run.packet.json`
- `ecaz bench suite run --config reviews/task-33/002-30213-task33-hnsw-m5-reference-refresh-run/task33-hnsw-m5-reference-refresh-run.packet.json --manifest-output reviews/task-33/002-30213-task33-hnsw-m5-reference-refresh-run/artifacts/suite-manifest.json`

Follow-up local validation is recorded in the manifest.

