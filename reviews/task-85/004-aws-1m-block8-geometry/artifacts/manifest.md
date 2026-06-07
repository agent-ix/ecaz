# Task 85 AWS 1M Block8 Geometry Manifest

- Packet: `reviews/task-85/004-aws-1m-block8-geometry/`
- Branch: `task-85-spire-product-scale-pareto`
- Suite config:
  `reviews/task-85/004-aws-1m-block8-geometry/suite-aws-1m-block8-geometry-q500.json`
- Lane / fixture: AWS profile `1m`, q500, prefix
  `task67_1m_hnsw_m7g2xlarge`, database `postgres`
- Storage format: `rabitq`
- Rerank mode: `rerank_width=25`
- Isolated surface: separate block8 SPIRE index
  `aws_spire_1m_rabitq_t85_block8_tg256_idx`; retained block16 index is not
  dropped.

## Rationale

Packet 003 joined retained miss attribution with enriched target-block context
and showed only `7/84` misses are in the immediate `1153..1280` band, while
the contextual miss p50 block rank is `2014` and `40/84` misses are beyond
`2048`. This makes another small global-cap recovery sweep unlikely to improve
latency at retained recall.

The block8 suite tests the remaining candidate-density hypothesis: smaller
leaf blocks may preserve recall at a larger block cap while reducing candidate
rows per selected block enough to beat the retained block16 warm floor.

## Validation Log

- `suite-audit.log`
  - Command: `target/debug/ecaz bench suite audit --config reviews/task-85/004-aws-1m-block8-geometry/suite-aws-1m-block8-geometry-q500.json --log-file reviews/task-85/004-aws-1m-block8-geometry/artifacts/suite-audit.log`
  - Result: `[suite:task85-aws-1m-block8-geometry-q500] audit passed: 8 steps`

## Pending AWS Artifacts

The AWS run should write under:

`reviews/task-85/004-aws-1m-block8-geometry/artifacts/aws-1m-block8-geometry-q500/`

Expected artifacts include suite manifest/results, precheck/build logs,
pipeline logs/funnels for each q500 cap, storage output, cloud resume/bench/pause
logs, and final paused status.
