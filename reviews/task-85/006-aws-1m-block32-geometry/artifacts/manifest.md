# Task 85 Packet 006 Artifact Manifest

- Task bucket: `reviews/task-85/`
- Packet: `reviews/task-85/006-aws-1m-block32-geometry/`
- Head SHA at preparation: `8baede8c6`
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

Suite completed successfully:

- S3 source:
  `s3://ecaz-cloud-1m-b62eb804/bench-artifacts/task85-aws-1m-block32-geometry-q500/20260607T102609Z/`
- Local suite manifest:
  `reviews/task-85/006-aws-1m-block32-geometry/artifacts/aws-1m-block32-geometry-q500/suite-manifest.json`
- Local results:
  `reviews/task-85/006-aws-1m-block32-geometry/artifacts/aws-1m-block32-geometry-q500/results.jsonl`
- Local report:
  `reviews/task-85/006-aws-1m-block32-geometry/artifacts/aws-1m-block32-geometry-q500/suite-report.md`

Key rows:

| row | recall@10 | p50 ms | p95 ms | p99 ms | candidates |
|---|---:|---:|---:|---:|---:|
| block16 global1152 first | 0.9876 | 257.664 | 331.715 | 2353.880 | 9,213,846 |
| block32 global384 | 0.9636 | 149.910 | 199.878 | 218.125 | 6,137,953 |
| block32 global576 | 0.9730 | 178.624 | 235.037 | 250.139 | 9,206,722 |
| block32 global768 | 0.9800 | 199.480 | 259.216 | 275.563 | 12,275,644 |
| block32 global1152 | 0.9876 | 235.691 | 295.157 | 308.841 | 18,413,851 |
| block16 global1152 repeat | 0.9876 | 237.482 | 297.192 | 310.792 | 9,213,846 |

Funnel medians:

| row | candidate sum | p50 object read ms | p50 score ms |
|---|---:|---:|---:|
| block16 global1152 repeat | 9,213,846 | 183.712 | 56.872 |
| block32 global384 | 6,137,953 | 98.280 | 30.196 |
| block32 global576 | 9,206,722 | 123.267 | 33.717 |
| block32 global768 | 12,275,644 | 139.581 | 37.230 |
| block32 global1152 | 18,413,851 | 167.198 | 44.204 |

Storage:

- block16 retained index: 872.1 MiB, 923.7 B/row.
- block32 index: 823.8 MiB, 872.6 B/row.

AWS final state after pause: `paused`, `~$0.00/hr running`,
`~$8.00/mo retained storage`.
