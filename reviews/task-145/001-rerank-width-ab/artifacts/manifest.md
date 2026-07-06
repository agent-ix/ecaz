# Task 145 Packet 001 Manifest: Rerank Width A/B

- Task bucket: `reviews/task-145/001-rerank-width-ab/`
- Head SHA: `0772ed06488efbf3d298f94008e70a33f53172dd`
- Branch: `task-145-spire-rerank-economy-low-probe`
- Captured: `2026-07-06T11:02:34Z`
- Suite: `task145-rerank-width-ab`
- Runner: `target/release/ecaz bench suite`
- Database: `tqvector_bench_task145`
- Host/socket: `/home/peter/dev/ecaz/target/task145-pg18-socket`
- Port: `28818`
- Isolated one-index-per-table surfaces: yes. Each scale uses a separate
  prefix/index (`t145_10k_n128_rerank_ab`, `t145_50k_n1024_rerank_ab`,
  `t145_100k_n1024_rerank_ab`).

## Backend Provenance

`suite-manifest.json` records the coordinator backend node:

- node: `coordinator`
- database: `tqvector_bench_task145`
- host: `/home/peter/dev/ecaz/target/task145-pg18-socket`
- port: `28818`
- build profile: `release`
- installed backend: `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`
- sha256: `a821e3ee67501cc7489dcc9380e2bfab867b33388f600ef1f8109d19751a5bf8`

The top-level `backend_build_profile` field is null, but per-node
`backend_nodes[]` is populated and every latency/recall result row carries
`backend_build_profile=release` and `backend_node_profiles=coordinator:28818:release`.

## Commands

Release install:

```text
target/release/ecaz dev install ecaz-pg-test --pg 18 ... --log-file reviews/task-145/001-rerank-width-ab/artifacts/install-release-pg18-r2.log
```

Precheck:

```text
target/release/ecaz dev sql --pg 18 --db tqvector_bench_task145 --socket-dir /home/peter/dev/ecaz/target/task145-pg18-socket --raw --sql "LOAD 'ecaz'; SELECT now() AS captured_at, version() AS postgres_version, ecaz_build_profile() AS ecaz_build_profile, current_setting('ec_spire.rerank_width') AS session_rerank_width, current_setting('ec_spire.leaf_score_only_routing') AS leaf_score_only_routing;" --log-output reviews/task-145/001-rerank-width-ab/artifacts/precheck-host.log
```

Suite run:

```text
target/release/ecaz bench suite run \
  --config reviews/task-145/001-rerank-width-ab/artifacts/task145-rerank-width-ab-suite.json \
  --database tqvector_bench_task145 \
  --host /home/peter/dev/ecaz/target/task145-pg18-socket \
  --port 28818 \
  --artifact-dir reviews/task-145/001-rerank-width-ab/artifacts \
  --manifest-output reviews/task-145/001-rerank-width-ab/artifacts/suite-manifest.json \
  --results-output reviews/task-145/001-rerank-width-ab/artifacts/suite-results.jsonl \
  --log-file reviews/task-145/001-rerank-width-ab/artifacts/suite-run.log
```

## Suite Controls

All pipeline cells hold these routing controls constant:

- `ec_spire.leaf_score_only_routing=on`
- `ec_spire.route_overfetch_multiplier=1.0`
- `ec_spire.probe_distance_ratio=0`
- storage format: `rabitq`
- `source_identity=include`
- `boundary_replica_count=0`
- top graph enabled with degree/list-size pass-through from the suite config

The A/B axis is `ec_spire.rerank_width`:

- baseline/full rerank: `rerank_width=0`
- economy candidate: `rerank_width=50`

## Artifacts

- `task145-rerank-width-ab-suite.json`: checked-in `ecaz bench suite` config.
- `suite-manifest.json`: completed suite manifest with per-node release backend provenance.
- `suite-results.jsonl`: structured load/storage/recall/pipeline result rows.
- `suite-run.log`: suite run log.
- `suite-manifest-dry-run.json`: dry-run manifest from suite validation.
- `install-release-pg18-r2.log`: release install log.
- `precheck-after-reinstall.log`, `precheck-host.log`,
  `precheck-task-socket.log`: release/profile and GUC prechecks.
- `load-*.log`, `storage-*.log`, `truth-cache-*.log`,
  `pipeline-*.log`: packet-local command logs cited by the suite manifest.

Truth-cache JSON files under this packet are intentionally ignored and are not
committed. They are regenerable cache data; the committed truth-cache logs and
`suite-results.jsonl` carry the cited recall facts.

## Storage

| Cell | SPIRE index size | Index bytes/row | Total bytes/row |
| --- | ---: | ---: | ---: |
| 10k n128/b0 | 10.1 MiB | 1058.4 B | 17752.9 B |
| 50k n1024/b0 | 54.4 MiB | 1139.8 B | 17826.9 B |
| 100k n1024/b0 | 97.8 MiB | 1025.1 B | 17711.3 B |

## Recall Truth-Cache Controls

| Cell | nprobe | distinct_recall@k | mean q-time | backend |
| --- | ---: | ---: | ---: | --- |
| 10k n128/b0 | 96 | 1.0000 | 265.83 ms | release |
| 50k n1024/b0 | 96 | 0.9590 | 190.43 ms | release |
| 100k n1024/b0 | 96 | 0.9300 | 391.04 ms | release |

## Pipeline A/B Results

### 10k n128/b0

| rerank_width | nprobe | p50 | p95 | distinct_recall@k |
| ---: | ---: | ---: | ---: | ---: |
| 0 | 8 | 24.530 ms | 29.981 ms | 0.9935 |
| 0 | 16 | 46.721 ms | 53.682 ms | 0.9970 |
| 0 | 32 | 91.495 ms | 99.278 ms | 1.0000 |
| 0 | 64 | 176.207 ms | 188.029 ms | 1.0000 |
| 0 | 96 | 267.021 ms | 285.123 ms | 1.0000 |
| 50 | 8 | 3.849 ms | 4.165 ms | 0.9935 |
| 50 | 16 | 4.397 ms | 4.778 ms | 0.9970 |
| 50 | 32 | 5.400 ms | 5.904 ms | 1.0000 |
| 50 | 64 | 7.396 ms | 8.037 ms | 1.0000 |
| 50 | 96 | 9.419 ms | 10.325 ms | 1.0000 |

At nprobe96, width 50 preserves recall and reduces p95 from 285.123 ms to
10.325 ms (27.6x faster).

### 50k n1024/b0

| rerank_width | nprobe | p50 | p95 | distinct_recall@k |
| ---: | ---: | ---: | ---: | ---: |
| 0 | 8 | 22.487 ms | 27.907 ms | 0.7590 |
| 0 | 16 | 37.161 ms | 45.097 ms | 0.8490 |
| 0 | 32 | 66.663 ms | 76.806 ms | 0.9105 |
| 0 | 64 | 126.842 ms | 142.654 ms | 0.9475 |
| 0 | 96 | 187.600 ms | 206.638 ms | 0.9595 |
| 50 | 8 | 10.097 ms | 11.128 ms | 0.7590 |
| 50 | 16 | 10.464 ms | 11.540 ms | 0.8490 |
| 50 | 32 | 11.078 ms | 12.143 ms | 0.9105 |
| 50 | 64 | 12.458 ms | 13.860 ms | 0.9475 |
| 50 | 96 | 13.748 ms | 15.059 ms | 0.9595 |

At nprobe96, width 50 preserves recall and reduces p95 from 206.638 ms to
15.059 ms (13.7x faster).

### 100k n1024/b0

| rerank_width | nprobe | p50 | p95 | distinct_recall@k |
| ---: | ---: | ---: | ---: | ---: |
| 0 | 8 | 32.225 ms | 42.912 ms | 0.7155 |
| 0 | 16 | 61.462 ms | 75.011 ms | 0.8270 |
| 0 | 32 | 122.283 ms | 139.827 ms | 0.8895 |
| 0 | 64 | 241.991 ms | 272.853 ms | 0.9375 |
| 0 | 96 | 367.310 ms | 403.823 ms | 0.9570 |
| 50 | 8 | 10.751 ms | 11.783 ms | 0.7155 |
| 50 | 16 | 11.249 ms | 12.287 ms | 0.8270 |
| 50 | 32 | 12.643 ms | 14.659 ms | 0.8895 |
| 50 | 64 | 15.415 ms | 17.266 ms | 0.9375 |
| 50 | 96 | 18.232 ms | 20.764 ms | 0.9570 |

At nprobe96, width 50 preserves recall and reduces p95 from 403.823 ms to
20.764 ms (19.4x faster).

## Decision

Promote `rerank_width=50` as the Task 145 economy candidate for the next slice.
In this release-build matrix it preserved distinct recall exactly at every
measured scale/nprobe while removing the full-heap rerank latency ramp.
