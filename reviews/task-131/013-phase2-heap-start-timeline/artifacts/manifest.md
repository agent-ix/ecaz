# Task 131 Packet 013 Artifact Manifest

- head SHA: `d4c2e324658f0f4b52196343e1c00d2d282e5b88`
- task bucket: `reviews/task-131/`
- packet path: `reviews/task-131/013-phase2-heap-start-timeline/`
- timestamp: `2026-07-01T08:54:25-07:00`
- lane: local multi-instance PG18, four local PostgreSQL instances
- fixture: Phase 13e static remote placement smoke fixture, 12 coordinator rows, 3 remote nodes, slow node-2 heap lock
- storage format: `rabitq`
- index shape: smoke fixture `nlists=3 / nprobe=3`
- rerank mode: `rerank_width=0`
- isolation: local four-instance coordinator + three remotes, not one-index-per-table

## Commands

### Build Check

```sh
cargo check --lib
```

### Local Multi-Instance Timeline Smoke

```sh
ECAZ_BIN=/home/peter/dev/ecaz/target/debug/ecaz \
  scripts/run_spire_phase13e_static_remote_placement_pg18.sh \
  --artifact-dir reviews/task-131/013-phase2-heap-start-timeline/artifacts \
  --run-id task131-heap-start-013 \
  --fixture-rows 12 \
  --bench-top-k 6 \
  --bench-queries-limit 1 \
  --bench-sweep 3
```

The harness generated and ran:

```sh
/home/peter/dev/ecaz/target/debug/ecaz \
  --database postgres \
  --host /home/peter/dev/ecaz/target/spire-phase13e-sockets-task131-heap-start-013 \
  --port 39440 \
  --user postgres \
  bench spire-pipeline \
  --prefix ec_spire_phase13e_coord \
  --index ec_spire_phase13e_coord_idx \
  --queries-limit 1 \
  --sweep 3 \
  --include-remote \
  --require-remote-placements \
  --remote-selected-pids 2,3,4 \
  --top-k 6 \
  --consistency-mode strict \
  --remote-tuple-transport pg_binary_attr_v1 \
  --include-query-metrics \
  --include-recall \
  --include-production-read-profile \
  --production-read-only \
  --query-metric-k 6 \
  --log-output reviews/task-131/013-phase2-heap-start-timeline/artifacts/bench-suite/spire-pipeline.log
```

## Artifacts

- `artifacts/phase13e-static-remote-placement.log`: local multi-instance harness log.
- `artifacts/production-read-timeline.tsv`: direct SQL timeline rows from the slow-node lock probe.
- `artifacts/bench-suite/phase13e-local-spire-pipeline-suite.json`: generated `ecaz bench suite` config.
- `artifacts/bench-suite/suite-manifest.json`: generated suite manifest; step status is `succeeded`.
- `artifacts/bench-suite/results.jsonl`: structured suite result rows.
- `artifacts/bench-suite/spire-pipeline.log`: rendered CLI report.
- `artifacts/bench-suite/suite-run.log`: suite stdout/stderr.
- `artifacts/cargo-check-lib.log`: build check log.
- `artifacts/*postgres.log`, `node-*-materialize-*.log`, `strict-remote-node2-failure.log`, `slow-remote-node2-lock.log`: small harness support logs.

No corpus TSVs, SSM logs, tunnel state, or raw polling snapshots are included.

## Key Result Lines

From `artifacts/phase13e-static-remote-placement.log`:

- `bench_suite_summary=passed|reviews/task-131/013-phase2-heap-start-timeline/artifacts/bench-suite/phase13e-local-spire-pipeline-suite.json|reviews/task-131/013-phase2-heap-start-timeline/artifacts/bench-suite/suite-manifest.json|reviews/task-131/013-phase2-heap-start-timeline/artifacts/bench-suite/results.jsonl`
- `production_timeline_rows=1|2|candidate_receive|0|13|13|6|ready|none;1|3|candidate_receive|0|13|12|3|ready|none;1|4|candidate_receive|0|13|12|3|ready|none;1|2|heap_receive|13|624|611|6|ready|none;1|3|heap_receive|13|25|12|3|ready|none;1|4|heap_receive|13|25|12|3|ready|none;`
- `production_timeline_summary=3|3|624|25|0`
- `SPIRE Phase 13e static remote placement PG18 fixture passed`

From `artifacts/production-read-timeline.tsv`:

```text
1|2|candidate_receive|0|13|13|6|ready|none
1|3|candidate_receive|0|13|12|3|ready|none
1|4|candidate_receive|0|13|12|3|ready|none
1|2|heap_receive|13|624|611|6|ready|none
1|3|heap_receive|13|25|12|3|ready|none
1|4|heap_receive|13|25|12|3|ready|none
```

Interpretation: after this patch, `heap_receive.started_after_ms` is the actual heap request start in the session-reuse path. In the slow-node fixture, heap requests start at 13 ms, fast-node heap completes at 25 ms, and the slow node completes at 624 ms. That proves the current session-reuse path can begin fast-node heap work before the slow-node heap path completes, and the timeline now reports that accurately.

## Validation

- `cargo check --lib` passed.
- Local four-instance PG18 harness passed.
