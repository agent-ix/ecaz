# Task 85 AWS 1M Block8 Geometry Manifest

- Packet: `reviews/task-85/004-aws-1m-block8-geometry/`
- Branch: `task-85-spire-product-scale-pareto`
- Head SHA: `f7e34bb9170bcbb1c60293747e8ad8d9586c885e`
- Suite config:
  `reviews/task-85/004-aws-1m-block8-geometry/suite-aws-1m-block8-geometry-q500.json`
- Lane / fixture: AWS profile `1m`, q500, prefix
  `task67_1m_hnsw_m7g2xlarge`, database `postgres`
- Storage format: `rabitq`
- Rerank mode: `rerank_width=25`
- Surface: shared corpus table with a separate block8 SPIRE index
  `aws_spire_1m_rabitq_t85_block8_tg256_idx`; retained block16 SPIRE indexes
  and comparator surfaces were not dropped.

## Rationale

Packet 003 joined retained miss attribution with enriched target-block context
and showed only `7/84` misses are in the immediate `1153..1280` band, while the
contextual miss p50 block rank is `2014` and `40/84` misses are beyond `2048`.
That made another small global-cap recovery sweep unlikely to improve latency at
retained recall.

The block8 suite tests the remaining candidate-density hypothesis: smaller leaf
blocks may preserve recall at a larger block cap while reducing candidate rows
per selected block enough to beat the retained block16 warm floor.

## Commands

Audit:

`target/debug/ecaz bench suite audit --config reviews/task-85/004-aws-1m-block8-geometry/suite-aws-1m-block8-geometry-q500.json --log-file reviews/task-85/004-aws-1m-block8-geometry/artifacts/suite-audit.log`

First AWS attempt:

`target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-85/004-aws-1m-block8-geometry/suite-aws-1m-block8-geometry-q500.json --suite task85-aws-1m-block8-geometry-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-85/004-aws-1m-block8-geometry/artifacts/cloud-bench-block8-geometry.log`

Successful AWS attempt after volume expansion:

`target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-85/004-aws-1m-block8-geometry/suite-aws-1m-block8-geometry-q500.json --suite task85-aws-1m-block8-geometry-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-85/004-aws-1m-block8-geometry/artifacts/cloud-bench-block8-geometry-after-volume-expand.log`

Report generation:

`target/debug/ecaz bench suite report --manifest reviews/task-85/004-aws-1m-block8-geometry/artifacts/aws-1m-block8-geometry-q500/suite-manifest.json --results-output reviews/task-85/004-aws-1m-block8-geometry/artifacts/aws-1m-block8-geometry-q500/results-report.jsonl --log-file reviews/task-85/004-aws-1m-block8-geometry/artifacts/aws-1m-block8-geometry-q500/suite-report.md`

## Failed First Attempt

The first cloud run used SSM command
`f187ed4b-136b-4438-9de9-757f1306ee26` and failed during block8 index build:

`ERROR: could not extend file "base/5/13706659": No space left on device`

Relevant artifacts:

- `cloud-bench-block8-geometry.log`
- `aws-1m-block8-geometry-q500-nospace-failed/build-spire-1m-rabitq-block8-tg256.log`
- `aws-1m-block8-geometry-q500-nospace-failed/precheck-aws-1m-block8-surface.log`
- `aws-1m-block8-geometry-q500-nospace-failed/suite-manifest.json`
- `aws-1m-block8-geometry-q500-nospace-failed/suite-run.log`

Disk inspection showed `/var/lib/pgsql/18` at `100G` used, `857M` available,
`100%` full. The top relations included the 1M HNSW corpus, DiskANN corpus,
M16 rerun corpus, retained HNSW index, and retained SPIRE indexes. Those
surfaces were preserved for Task85 comparator evidence instead of being dropped.

## Volume Expansion

The data EBS volume `vol-0e251e37a779308f8` was expanded from `100` GiB to
`150` GiB and the mounted XFS filesystem was grown:

- `volume-modify-100-to-150.log`
- `xfs-growfs-after-volume-expand.log`

After `xfs_growfs`, `/var/lib/pgsql/18` reported `150G` size, `100G` used,
`51G` available, `67%` full.

## Successful Run

The successful cloud run used SSM command
`67923c8f-4346-4a28-ad38-121f915fd0bb` and synced artifacts from:

`s3://ecaz-cloud-1m-b62eb804/bench-artifacts/task85-aws-1m-block8-geometry-q500/20260607T073042Z/`

Suite result:

- completed: `8`
- failed: `0`
- skipped: `0`
- build duration: `2,811,483 ms`

Key matched-recall comparison against the retained block16 warm floor:

| Row | Recall@10 | p50 | p95 | p99 | Candidate Sum |
| --- | ---: | ---: | ---: | ---: | ---: |
| retained block16 warm floor | 0.9832 | 246.397 ms | 304.476 ms | 321.342 ms | 9,213,846 |
| block8 global1152 first | 0.9832 | 283.839 ms | 357.274 ms | 2729.302 ms | 4,607,442 |

Additional block8 rows:

| Row | Recall@10 | p50 | p95 | p99 | Candidate Sum |
| --- | ---: | ---: | ---: | ---: | ---: |
| block8 global1536 | 0.9876 | 283.900 ms | 340.606 ms | 350.005 ms | 6,143,277 |
| block8 global2048 | 0.9914 | 308.561 ms | 355.499 ms | 361.867 ms | 8,191,063 |
| block8 global2304 | 0.9926 | 320.404 ms | 362.190 ms | 371.869 ms | 9,214,933 |
| block8 global2048 repeat | 0.9914 | 307.925 ms | 355.430 ms | 362.980 ms | 8,191,063 |

Storage result:

- total: `19.3 GiB`
- indexes: `4.0 GiB`
- block8 SPIRE index: `968.8 MiB`, `1026.1 B/row`
- retained Task80 block16 SPIRE index: `872.1 MiB`, `923.7 B/row`
- Task84 k3 block16 SPIRE index: `936.4 MiB`, `991.9 B/row`

## Verdict

Block8 is rejected as a Task85 product-scale latency improvement. It halves
candidate count at the retained recall point, but p50 and p95 regress, and the
first row has a severe p99/max outlier. Higher-recall block8 rows are also
slower. This packet therefore rules out smaller block geometry as the next
default candidate unless a later profile identifies and removes block8-specific
overhead.

## AWS Final State

`target/debug/ecaz cloud status --profile 1m --database postgres`

Final captured status:

`profile=1m state=paused db=10.42.1.131 bucket=ecaz-cloud-1m-b62eb804`
