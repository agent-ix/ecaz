# Task 81 Packet 003: AWS 1M Block-Summary Gate

## Request

Review the AWS 1M follow-up for Task 81 after the accepted local gate in packet 002. This packet intentionally measures the current block-summary mechanism at the candidate-preserving `global1152` point to test the Task 81 AWS gate:

- retained AWS profile `1m`
- retained 990k-row corpus table `task67_1m_hnsw_m7g2xlarge_corpus`
- q500 query lane from `task67_1m_hnsw_m7g2xlarge_queries`
- retained SPIRE index `aws_spire_1m_rabitq_t80_block16_tg256_idx`
- `nprobe=96`
- `rerank_width=25`
- `ec_spire.leaf_block_pruning_max_global_blocks=1152`

## Result

The AWS gate does **not** pass.

| Row | Candidates | p50 | p95 | p99 | recall@10 |
| --- | ---: | ---: | ---: | ---: | ---: |
| old tg96 comparator | 9,213,846 | 268.824 ms | 331.460 ms | 345.762 ms | 0.9832 |
| Task 81 AWS global1152 | 9,213,846 | 265.911 ms | 329.407 ms | 342.454 ms | 0.9832 |

The candidate surface is preserved exactly and p50 improves slightly, but recall is unchanged. Task 81 requires AWS accepted rows to improve recall over the old tg96 row without increasing the candidate surface, so this is a failed gate.

Diagnostic attribution for the measured q500 row:

- `blocks_available=23389983`
- `blocks_selected=576000`
- `blocks_skipped=22813983`
- `summary_score_nanos=23130193665`
- `row_score_nanos=5241865350`
- `summary_bytes=37126170208`
- `row_bytes=304802815448`

The selected block count is exactly `1152 * 500`, so the global block cap is being applied as intended.

## Evidence

- Artifact manifest: `reviews/task-81/003-aws-1m-block-summary-gate/artifacts/manifest.md`
- Suite config: `reviews/task-81/003-aws-1m-block-summary-gate/suite-aws-1m-block-summary-gate.json`
- Suite manifest: `reviews/task-81/003-aws-1m-block-summary-gate/artifacts/aws-1m-block-summary-gate/suite-manifest.json`
- Suite status: `reviews/task-81/003-aws-1m-block-summary-gate/artifacts/aws-1m-block-summary-gate/suite-status.log`
- Suite report: `reviews/task-81/003-aws-1m-block-summary-gate/artifacts/aws-1m-block-summary-gate/suite-report.log`
- Pipeline log: `reviews/task-81/003-aws-1m-block-summary-gate/artifacts/aws-1m-block-summary-gate/pipeline-spire-1m-rabitq-block-summary-global1152.log`
- Diagnostics log: `reviews/task-81/003-aws-1m-block-summary-gate/artifacts/aws-1m-block-summary-gate/diagnostics-spire-1m-rabitq-block-summary-global1152.log`
- Final AWS status: `reviews/task-81/003-aws-1m-block-summary-gate/artifacts/cloud-status-after-pause-final.log`

## Reviewer Focus

1. Confirm this is the correct candidate-preserving AWS gate comparison against the old `9,213,846` q500 shape.
2. Confirm the packet should be interpreted as a failed Task 81 AWS gate, not a closeout.
3. Check the diagnostic aggregate for consistency with the pipeline candidate row and `global1152` block cap.
