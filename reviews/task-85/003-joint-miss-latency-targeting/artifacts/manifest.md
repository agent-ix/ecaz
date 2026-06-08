# Task 85 Joint Miss Targeting Manifest

- Packet: `reviews/task-85/003-joint-miss-latency-targeting/`
- Head SHA: `4d7676c32` (`Record Task 85 AWS product baseline`)
- Lane / fixture: AWS 1M/q500 retained SPIRE baseline, q500 truth neighbors
- Storage format: `rabitq`
- Rerank mode: retained `rerank_width=25`
- Isolated surface: retained one-index-per-table SPIRE surface
  `aws_spire_1m_rabitq_t80_block16_tg256_idx`

## Inputs

- `reviews/task-85/001-handoff-product-baseline-suite/artifacts/aws-1m-product-baseline-q500/miss-attribution-retained-global1152-q500-repeat.jsonl`
  - Warm retained Task85 baseline miss attribution.
  - Key baseline row: recall@10 `0.9832`, candidate_sum `9,213,846`, p50
    `246.397 ms`, p95 `304.476 ms`, p99 `321.342 ms`.
- `reviews/task-84/001-enriched-block-context-diagnostic/artifacts/aws-1m-enriched-block-context-q500/target-block-context-spire-1m-global1152-q500.jsonl`
  - Enriched target-block context with `block_rank`, `route_rank`, and
    `block_ip_margin_to_cap`.

## Generated Artifacts

- `joint-miss-records.json`
  - Command:
    `jq -n --slurpfile miss reviews/task-85/001-handoff-product-baseline-suite/artifacts/aws-1m-product-baseline-q500/miss-attribution-retained-global1152-q500-repeat.jsonl --slurpfile ctx reviews/task-84/001-enriched-block-context-diagnostic/artifacts/aws-1m-enriched-block-context-q500/target-block-context-spire-1m-global1152-q500.jsonl '<join and bucket expression>' > reviews/task-85/003-joint-miss-latency-targeting/artifacts/joint-miss-records.json`
  - Result: 84 joined miss records.
- `joint-miss-summary.json`
  - Command:
    `jq '<aggregate expression>' reviews/task-85/003-joint-miss-latency-targeting/artifacts/joint-miss-records.json > reviews/task-85/003-joint-miss-latency-targeting/artifacts/joint-miss-summary.json`
  - Result: miss-stage, block-rank, route-rank, and margin buckets.

## Key Results

- Miss count: `84` across `64` distinct queries.
- Miss stage:
  - routing miss: `3`
  - selected-leaf block pruning or candidate cap: `81`
- Block-rank buckets:
  - `1153..1280`: `7`
  - `1281..1536`: `15`
  - `1537..2048`: `19`
  - `>2048`: `40`
  - missing context: `3`
- Block-rank stats for contextual misses: min `1154`, p50 `2014`, p75 `3501`,
  p90 `5848`, p95 `8789`, max `11559`.
- Route-rank stats for contextual misses: min `1`, p50 `15`, p75 `32`, p90
  `50`, p95 `69`, max `88`.

## Decision Supported

The evidence argues against another small cap/recovery sweep as the next
latency mechanism. Most misses are too far beyond the retained cap for bounded
recovery to retain recall without growing candidates. The next Task85 mechanism
slice should instead test smaller leaf-block geometry (`leaf_block_rows=8`) at
AWS 1M/q500, using the Task85 warm retained row as the floor.
