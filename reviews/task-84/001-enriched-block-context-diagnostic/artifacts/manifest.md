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

## AWS Enriched Baseline

- Resume log: `cloud-resume-task84-enriched-baseline.log`
  - Command: `target/debug/ecaz cloud resume --profile 1m --database postgres --log-file reviews/task-84/001-enriched-block-context-diagnostic/artifacts/cloud-resume-task84-enriched-baseline.log`
  - Result: `resume: profile=1m db=10.42.1.131 ready`
- Install log: `cloud-install-task84-enriched-baseline.log`
  - Command: `target/debug/ecaz cloud install --profile 1m --database postgres --git-ref 3b8513a4d --skip-extension-recreate --clean-cargo-target --timeout 3600 --log-file reviews/task-84/001-enriched-block-context-diagnostic/artifacts/cloud-install-task84-enriched-baseline.log`
  - Result: `install: profile=1m db=10.42.1.131 ref=3b8513a4d ok`
- Bench log: `cloud-bench-task84-enriched-baseline.log`
  - Command: `target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-84/001-enriched-block-context-diagnostic/suite-aws-1m-enriched-block-context-q500.json --suite task84-aws-1m-enriched-block-context-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-84/001-enriched-block-context-diagnostic/artifacts/cloud-bench-task84-enriched-baseline.log`
  - Result: synced artifacts from `s3://ecaz-cloud-1m-b62eb804/bench-artifacts/task84-aws-1m-enriched-block-context-q500/20260606T150136Z/`
- Pause log: `cloud-pause-after-task84-enriched-baseline.log`
- Status logs:
  - `cloud-status-after-task84-enriched-baseline.log`: captured while EC2 was `stopping`.
  - `cloud-status-final-paused.log`: final status `state: paused`.

Synced artifacts:

- `aws-1m-enriched-block-context-q500/suite-config.json`
- `aws-1m-enriched-block-context-q500/suite-manifest.json`
- `aws-1m-enriched-block-context-q500/suite-run.log`
- `aws-1m-enriched-block-context-q500/results.jsonl`
- `aws-1m-enriched-block-context-q500/register-enriched-target-block-context.log`
- `aws-1m-enriched-block-context-q500/pipeline-spire-1m-rabitq-enriched-block-context-global1152-q500.log`
- `aws-1m-enriched-block-context-q500/miss-attribution-spire-1m-global1152-q500.jsonl`
- `aws-1m-enriched-block-context-q500/target-block-context-spire-1m-global1152-q500.jsonl`
- `selected-leaf-miss-enriched-context.tsv`

Key rows:

- Retained baseline: `recall@10=0.9832`, `candidate_sum=9,213,846`,
  p50 `282.869 ms`, p95 `354.557 ms`, p99 `368.232 ms`.
- Miss attribution: `4916` hit rows, `3` `routing_miss`, `81`
  `selected_leaf_block_pruning_or_candidate_cap`.
- Target context status: `4916` selected by cap, `81` ranked outside cap,
  `3` not found in routed leaves.
- Selected-leaf miss rank deltas beyond cap `1152`: `7` within `+128`,
  `23` in `+129..+512`, `28` in `+513..+2048`, `23` farther than `+2048`.
- Selected-leaf miss score margins (`block_ip - cap_block_ip`): min
  `-0.07606012`, p50 `-0.017314821`, p90 `-0.0036982894`, max
  `-0.00009295344`.
- Selected-leaf miss exclusive margin buckets: `3` within `0.001`, `8` in
  `0.001..0.005`, `15` in `0.005..0.01`, `55` worse than `-0.01`;
  cumulative counts are `11` within `0.005` and `26` within `0.01`.
- Selected-leaf miss route ranks: min `1`, p50 `15`, p90 `50`, max `88`.

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

## Next Evidence

The next Task 84 checkpoint should use this enriched baseline to test block-score
calibration changes against the same retained candidate surface. The route-rank
distribution suggests most selected-leaf misses are already in plausible routed
leaves; the score-margin distribution suggests a very narrow near-cap rescue is
insufficient by itself.
