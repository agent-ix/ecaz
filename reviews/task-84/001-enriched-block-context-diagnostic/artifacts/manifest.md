# Task 84 Enriched Block Context Diagnostic Manifest

- Task: `plan/tasks/84-spire-1m-recall-recovery-without-candidate-inflation.md`
- Packet: `reviews/task-84/001-enriched-block-context-diagnostic/`
- Code commit under review: `1784524bd`
- Branch: `task-84-spire-recall-recovery`
- Suite config: `reviews/task-84/001-enriched-block-context-diagnostic/suite-aws-1m-enriched-block-context-q500.json`

## Code Scope

Diagnostic-only enrichment for SPIRE block-rank snapshots:

- Adds `cap_block_ip`, the block summary score at the active global block cap.
- Adds `block_ip_margin_to_cap`, computed as `block_ip - cap_block_ip`.
- Adds `route_rank`, the 1-based selected route order for the target leaf.
- Adds `route_score`, the selected route score for the target leaf.
- Wires the fields through:
  - scan row: `SpireLeafBlockRankSnapshotRow`
  - coordinator row: `SpireIndexScanLeafBlockRankSnapshotRow`
  - SQL functions:
    `ec_spire_index_scan_leaf_block_rank_snapshot(...)` and
    `ec_spire_index_scan_leaf_target_block_rank_snapshot(...)`
  - CLI JSONL output from `ecaz bench spire-pipeline`

No production candidate selection, cap policy, or scoring behavior changes in
this checkpoint.

## Validation Artifacts

- `cargo-test-ecaz-cli-spire-pipeline.log`
  - Command: `cargo test -p ecaz-cli spire_pipeline --no-default-features`
  - Result: `20 passed; 0 failed`
- `suite-audit.log`
  - Command: `target/debug/ecaz bench suite audit --config reviews/task-84/001-enriched-block-context-diagnostic/suite-aws-1m-enriched-block-context-q500.json --log-file reviews/task-84/001-enriched-block-context-diagnostic/artifacts/suite-audit.log`
  - Result: audit passed before padding.
- `suite-audit-padded.log`
  - Command: `target/debug/ecaz bench suite audit --config reviews/task-84/001-enriched-block-context-diagnostic/suite-aws-1m-enriched-block-context-q500.json --log-file reviews/task-84/001-enriched-block-context-diagnostic/artifacts/suite-audit-padded.log`
  - Result: `[suite:task84-aws-1m-enriched-block-context-q500] audit passed: 2 steps`

## Suite Intent

The suite is prepared for the first Task 84 AWS baseline run. It preserves the
Task 79/81 retained surface:

- AWS profile: `1m`
- fixture/index: `task67_1m_hnsw_m7g2xlarge`,
  `aws_spire_1m_rabitq_t80_block16_tg256_idx`
- cap/settings: `global1152`, no sampled global probing, no route-prior
  weighting, radius weight `0.25`
- q500 truth cache:
  `benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/truth-aws-real-1m-q500-k10.json`

The config is padded above 7 KB so `ecaz cloud bench` should use the S3 config
upload path rather than inline SSM config transfer.

## Pending Evidence

The AWS 1M/q500 enriched baseline has not been run in this checkpoint. The next
step is to install this branch on AWS `1m`, run the suite, pause AWS, and join
the enriched output against Task 83's missed selected-leaf rows.
