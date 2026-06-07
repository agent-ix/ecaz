# Task 85 Packet 006 Artifact Manifest

- Task bucket: `reviews/task-85/`
- Packet: `reviews/task-85/006-aws-1m-block32-geometry/`
- Head SHA at preparation: pending commit
- Suite config:
  `reviews/task-85/006-aws-1m-block32-geometry/suite-aws-1m-block32-geometry-q500.json`
- Lane: AWS 1M, PG18, `ec_spire`, q500.
- Surface: isolated one-index-per-table SPIRE index surfaces on
  `task67_1m_hnsw_m7g2xlarge_corpus`.

## Purpose

Test block32 as a latency mechanism under the corrected Task 85 goal:
lower latency while retaining the current recall level. This keeps global block
allocation enabled and varies block geometry, because packet 005 showed per-leaf
allocation breaks both recall and object-read cost.

## Planned Commands

Audit:

`target/debug/ecaz bench suite audit --config reviews/task-85/006-aws-1m-block32-geometry/suite-aws-1m-block32-geometry-q500.json --log-file reviews/task-85/006-aws-1m-block32-geometry/artifacts/suite-audit.log`

Result: passed, 9 steps.

AWS run:

`target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-85/006-aws-1m-block32-geometry/suite-aws-1m-block32-geometry-q500.json --suite task85-aws-1m-block32-geometry-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-85/006-aws-1m-block32-geometry/artifacts/cloud-bench-block32-geometry.log`

Report:

`target/debug/ecaz bench suite report --manifest reviews/task-85/006-aws-1m-block32-geometry/artifacts/aws-1m-block32-geometry-q500/suite-manifest.json --results-output reviews/task-85/006-aws-1m-block32-geometry/artifacts/aws-1m-block32-geometry-q500/results-report.jsonl --log-file reviews/task-85/006-aws-1m-block32-geometry/artifacts/aws-1m-block32-geometry-q500/suite-report.md`

## Results

Pending.
