# Manifest: AWS Retained Row Segment Funnel After Signature Update

Task bucket: `reviews/task-85/019-aws-retained-row-segment-funnel-post-signature/`
Head SHA: `e07b4be5ee28ae74d85a7b4a601340307f0bb413`
Timestamp: 2026-06-07

## Surface

- Lane: AWS `1m` retained SPIRE q500
- Fixture prefix: `task67_1m_hnsw_m7g2xlarge`
- Corpus rows: `990000`
- Query rows: `10000`
- Index: `aws_spire_1m_rabitq_t80_block16_tg256_idx`
- Index size: `872 MB`
- Storage format: `rabitq`
- Isolated one-index-per-table surface: yes
- Shared-table surface: no
- Rerank width: `25`
- nprobe: `96`
- Retained block policy:
  - `ec_spire.leaf_block_pruning_max_blocks_per_leaf=0`
  - `ec_spire.leaf_block_pruning_max_global_blocks=1152`
  - `ec_spire.leaf_block_pruning_global_probe_blocks=0`
  - `ec_spire.leaf_block_pruning_sample_rows_per_block=0`
  - `ec_spire.leaf_block_pruning_sample_summary_prior_weight=0.8`
  - `ec_spire.leaf_block_pruning_summary_radius_weight=0.25`
  - `ec_spire.leaf_block_pruning_route_prior_weight=0.0`

## Commands

- Audit:
  - `target/debug/ecaz bench suite audit --config reviews/task-85/019-aws-retained-row-segment-funnel-post-signature/suite-aws-1m-retained-row-segment-funnel-post-signature-q500.json`

- AWS run:
  - `target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-85/019-aws-retained-row-segment-funnel-post-signature/suite-aws-1m-retained-row-segment-funnel-post-signature-q500.json --suite task85-aws-1m-retained-row-segment-funnel-post-signature-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-85/019-aws-retained-row-segment-funnel-post-signature/artifacts/cloud-bench-retained-row-segment-post-signature.log`

- Pause:
  - `target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-85/019-aws-retained-row-segment-funnel-post-signature/artifacts/cloud-pause-after-retained-row-segment-post-signature.log`

- Final status:
  - `target/debug/ecaz cloud status --profile 1m --log-file reviews/task-85/019-aws-retained-row-segment-funnel-post-signature/artifacts/cloud-status-final-after-retained-row-segment-post-signature.log`

## Artifacts

- `suite-aws-1m-retained-row-segment-funnel-post-signature-q500.json`
  - Suite config for retained q500 first and warm repeat runs.

- `artifacts/suite-audit.log`
  - Result: audit passed, 3 steps.

- `artifacts/aws-1m-retained-row-segment-funnel-post-signature-q500/precheck-aws-1m-retained-row-segment-surface-post-signature.log`
  - Key lines:
    - PostgreSQL 18.3 on aarch64 Amazon Linux
    - corpus rows `990000`
    - query rows `10000`
    - index size `872 MB`
    - appended row-segment columns selectable from
      `ec_spire_index_scan_leaf_candidate_snapshot`

- `artifacts/aws-1m-retained-row-segment-funnel-post-signature-q500/results.jsonl`
  - Warm repeat result:
    - recall@10 `0.9876`
    - candidate_sum `9,213,846`
    - heap_rerank_sum `12,500`
    - latency p50 `227.388 ms`
    - latency p95 `284.166 ms`
    - latency p99 `297.164 ms`
    - latency max `301.404 ms`

- `artifacts/aws-1m-retained-row-segment-funnel-post-signature-q500/funnel-retained-global1152-q500-repeat-post-signature.jsonl`
  - Warm repeat row-segment aggregate:
    - queries `500`
    - candidate_sum `9,213,846`
    - leaf_object_bytes_sum `341,934,265,656`
    - leaf_summary_object_bytes_sum `37,126,170,208`
    - leaf_row_object_bytes_sum `304,802,815,448`
    - leaf_row_segment_read_count_sum `1,180,606`
    - leaf_row_segment_read_bytes_sum `9,622,405,352`
    - leaf_object_read_nanos_sum `94,059,241,491`
    - leaf_summary_score_nanos_sum `22,719,628,957`
    - candidate_score_nanos_sum `27,865,902,960`
    - selected_blocks_sum `576,000`

- `artifacts/cloud-pause-after-retained-row-segment-post-signature.log`
  - Result: DB and loader instances moved from running to stopping.

- `artifacts/cloud-status-final-after-retained-row-segment-post-signature.log`
  - Result: profile `1m` paused, running cost `$0.00/hr`, retained storage
    `~$8.00/mo`.
