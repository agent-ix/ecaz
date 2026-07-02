# Task 123 Packet 011 Artifact Manifest

- Head SHA: `641c853e792ee7c713049467c1e43b46a42481e1`
- Task bucket: `reviews/task-123/011-multi-instance-100k-timeline-rerun`
- Timestamp: `2026-06-28T04:22:20-07:00`
- Host lane: local four-instance PG18, Unix sockets, one coordinator plus three local remote PostgreSQL instances
- Corpus: `ec_real_100k`, staged under `/home/peter/dev/ecaz/data/staged-current`
- Storage format: `rabitq`
- Isolated surfaces: one coordinator table/index plus one remote table/index per local remote instance
- Runner: `ecaz bench suite`
- TSV policy: generated corpus and assignment TSV files were deleted after load/materialization; no TSV files remain in the packet.

## Measurement Infra Change

Commit `641c853e792ee7c713049467c1e43b46a42481e1` adds:

- `--bench-query-metric-projection-columns` to `ecaz dev spire-multicluster local-multinode-pg18`
- `bench_query_metric_projection_columns` to `spire-local-multinode` suite steps

Validation:

- `cargo test -p ecaz-cli spire_local_multinode_step_expands_local_four_instance_lane -- --nocapture`
  - Result: passed, 1 test, 414 filtered
- `cargo build -p ecaz-cli --bin ecaz`
  - Result: succeeded
  - Note: emitted the pre-existing `LoadedDistributedPlacementConfig.path` dead-code warning.

## Commands

### n128 b4/tr50/f8 200q Realistic Projection Attempt

```bash
target/debug/ecaz bench suite run \
  --config reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/task123-mi-100k-n128-200q-source-suite.json \
  --database postgres --host /tmp --port 28818 \
  --manifest-output reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n128-200q-suite-manifest.json \
  --results-output reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n128-200q-results.jsonl \
  --log-file reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n128-200q-suite-run.log
```

Outcome: build completed through indexes; initial run was interrupted by premature cleanup of generated assignment TSVs. The cluster was recovered by regenerating assignments, refreshing descriptor generation, and rerunning the production-read suite with id-only projection.

### n128 b4/tr50/f8 200q Id-Only Recovery

```bash
target/debug/ecaz --database postgres \
  --host /tmp/ecaz-task123/target/spire-local-multinode-sockets-task123-p11-mi-n128-b4-200q-source \
  --port 40320 --user ecaz_coord \
  bench suite run \
  --config reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n128-b4-200q-source/bench-suite/local-real-production-read-idonly-suite.json \
  --manifest-output reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n128-b4-200q-source/bench-suite/suite-manifest-idonly.json \
  --results-output reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n128-b4-200q-source/bench-suite/results-idonly.jsonl \
  --log-file reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n128-b4-200q-source/bench-suite/suite-run-idonly.log
```

### n1024 b2/tr50/f8 200q Realistic Projection Attempt

```bash
target/debug/ecaz bench suite run \
  --config reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/task123-mi-100k-n1024-200q-source-suite.json \
  --database postgres --host /tmp --port 28818 \
  --manifest-output reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n1024-200q-suite-manifest.json \
  --results-output reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n1024-200q-results.jsonl \
  --log-file reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n1024-200q-suite-run.log
```

Outcome: clean build, materialization, and storage step completed. The nested production-read step with `query_metric_projection_columns=["id","source"]` failed with `remote_heap_resolution_failed`; see `n1024-b2-200q-source/bench-suite/suite-run.log` and `n1024-b2-200q-source/coord-postgres.log`.

### n1024 b2/tr50/f8 200q Id-Only Recovery

```bash
target/debug/ecaz --database postgres \
  --host /tmp/ecaz-task123/target/spire-local-multinode-sockets-task123-p11-mi-n1024-b2-200q-source \
  --port 40330 --user ecaz_coord \
  bench suite run \
  --config reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n1024-b2-200q-source/bench-suite/local-real-production-read-idonly-suite.json \
  --manifest-output reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n1024-b2-200q-source/bench-suite/suite-manifest-idonly.json \
  --results-output reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n1024-b2-200q-source/bench-suite/results-idonly.jsonl \
  --log-file reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n1024-b2-200q-source/bench-suite/suite-run-idonly.log
```

## Key Results

### n128 b4/tr50/f8, id-only, 200 queries

Artifacts:

- Config: `artifacts/n128-b4-200q-source/bench-suite/local-real-production-read-idonly-suite.json`
- Results JSONL: `artifacts/n128-b4-200q-source/bench-suite/results-idonly.jsonl`
- Default log: `artifacts/n128-b4-200q-source/bench-suite/production-read-k10-default-idonly.log`
- Rowcap log: `artifacts/n128-b4-200q-source/bench-suite/production-read-k10-rowcap25k-idonly.log`
- Storage log: `artifacts/n128-b4-200q-source/bench-suite/storage.log`

Storage:

- Table total: `1.6 GiB`
- Indexes: `394.5 MiB`
- Total: `1.9 GiB`
- Coordinator index: `392.2 MiB`, `4112.6 B/row`

Coordinator query metrics:

| mode | nprobe | queries | p50 | p95 | recall@10 |
| --- | ---: | ---: | ---: | ---: | ---: |
| default | 8 | 200 | 662.821 ms | 923.969 ms | 0.9900 |
| default | 96 | 200 | 5408.521 ms | 5815.967 ms | 1.0000 |
| rowcap25k | 8 | 200 | 660.048 ms | 928.136 ms | 0.9900 |
| rowcap25k | 96 | 200 | 5409.689 ms | 5767.709 ms | 1.0000 |

Per-node id payload bytes:

- nprobe 8: node 2 `15,680`, node 3 `15,760`, node 4 `15,920`
- nprobe 96: node 2 `16,000`, node 3 `16,000`, node 4 `16,000`

### n1024 b2/tr50/f8, id-only, 200 queries

Artifacts:

- Config: `artifacts/n1024-b2-200q-source/bench-suite/local-real-production-read-idonly-suite.json`
- Results JSONL: `artifacts/n1024-b2-200q-source/bench-suite/results-idonly.jsonl`
- Default log: `artifacts/n1024-b2-200q-source/bench-suite/production-read-k10-default-idonly.log`
- Rowcap log: `artifacts/n1024-b2-200q-source/bench-suite/production-read-k10-rowcap25k-idonly.log`
- Storage log: `artifacts/n1024-b2-200q-source/bench-suite/storage.log`

Storage:

- Table total: `1.6 GiB`
- Indexes: `248.4 MiB`
- Total: `1.8 GiB`
- Coordinator index: `246.1 MiB`, `2580.9 B/row`

Coordinator query metrics:

| mode | nprobe | queries | p50 | p95 | recall@10 |
| --- | ---: | ---: | ---: | ---: | ---: |
| default | 8 | 200 | 555.397 ms | 581.701 ms | 0.9290 |
| default | 64 | 200 | 770.595 ms | 860.296 ms | 1.0000 |
| rowcap25k | 8 | 200 | 557.193 ms | 582.105 ms | 0.9290 |
| rowcap25k | 64 | 200 | 766.879 ms | 845.695 ms | 1.0000 |

Per-node id payload bytes:

- nprobe 8: node 2 `15,920`, node 3 `14,880`, node 4 `15,600`
- nprobe 64: node 2 `16,000`, node 3 `16,000`, node 4 `16,000`

## Realistic Projection Status

The clean n1024 run with `query_metric_projection_columns=["id","source"]` did not produce valid timing rows. It failed during the nested production-read step with:

```text
ERROR: EcSpireDistributedScan production executor blocked: status remote_heap_resolution_failed, next_blocker remote_heap_resolution, recommendation inspect production remote heap failure category before final row delivery
```

This is recorded in:

- `artifacts/n1024-b2-200q-source/bench-suite/suite-run.log`
- `artifacts/n1024-b2-200q-source/coord-postgres.log`

The completed decision-grade timings in this packet therefore use id-only projection to isolate the core routing/recall behavior from the current realistic-payload projection failure.
