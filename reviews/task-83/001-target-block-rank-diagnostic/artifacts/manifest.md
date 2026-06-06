# Task 83 Target-Block Rank Diagnostic Manifest

- Task: `plan/tasks/83-spire-selected-block-containment-recovery.md`
- Packet: `reviews/task-83/001-target-block-rank-diagnostic/`
- Code commit under review: `77cafdacd4361e7fb97f3d2e902f7aaa18c3d809`
- Suite config: `reviews/task-83/001-target-block-rank-diagnostic/suite-aws-1m-target-block-rank-q500.json`

## Code Scope

This checkpoint adds a target-only selected-block containment diagnostic:

- SQL: `ec_spire_index_scan_leaf_target_block_rank_snapshot(index_oid, query, target_local_sequences)`
- CLI: `ecaz bench spire-pipeline --target-block-rank-output <path>`
- Suite config field: `target_block_rank_output`

The diagnostic reuses normal SPIRE routing and block-summary scoring, locates
the truth target row only inside routed leaves, and emits the containing block's
global rank and `selected_by_global_cap` flag. It avoids the Task 82 full
block-rank helper's expensive row scan across every ranked block.

## Validation Artifacts

- `cargo-test-ecaz-cli-spire-pipeline.log`
  - Command: `cargo test -p ecaz-cli spire_pipeline --no-default-features`
  - Result: `20 passed; 0 failed`
- `cargo-build-ecaz-cli.log`
  - Command: `cargo build -p ecaz-cli --no-default-features`
  - Result: passed with the pre-existing `LoadedDistributedPlacementConfig.path` dead-code warning.
- `suite-audit.log`
  - Command: `target/debug/ecaz bench suite audit --config reviews/task-83/001-target-block-rank-diagnostic/suite-aws-1m-target-block-rank-q500.json --log-file reviews/task-83/001-target-block-rank-diagnostic/artifacts/suite-audit.log`
  - Result: `[suite:task83-aws-1m-target-block-rank-q500] audit passed: 2 steps`
- `suite-audit-padded.log`
  - Command: `target/debug/ecaz bench suite audit --config reviews/task-83/001-target-block-rank-diagnostic/suite-aws-1m-target-block-rank-q500.json --log-file reviews/task-83/001-target-block-rank-diagnostic/artifacts/suite-audit-padded.log`
  - Result: padded config audit passed; padding only forced the S3 cloud config upload path.

## AWS Diagnostic Run

Successful run:

- Resume log: `cloud-resume-task83-decode-fix.log`
- Install log: `cloud-install-task83-decode-fix.log`
  - Command: `target/debug/ecaz cloud install --profile 1m --database postgres --git-ref 77cafdacd --skip-extension-recreate --clean-cargo-target --timeout 3600 --log-file reviews/task-83/001-target-block-rank-diagnostic/artifacts/cloud-install-task83-decode-fix.log`
  - Result: `install: profile=1m db=10.42.1.131 ref=77cafdacd ok`
- Bench log: `cloud-bench-task83-target-block-rank-q500-decode-fix.log`
  - Command: `target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-83/001-target-block-rank-diagnostic/suite-aws-1m-target-block-rank-q500.json --suite task83-aws-1m-target-block-rank-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-83/001-target-block-rank-diagnostic/artifacts/cloud-bench-task83-target-block-rank-q500-decode-fix.log`
  - Result: synced artifacts from `s3://ecaz-cloud-1m-b62eb804/bench-artifacts/task83-aws-1m-target-block-rank-q500/20260606T000114Z/`
- Pause log: `cloud-pause-after-task83-target-block-rank-success.log`
- Final status: `cloud-status-after-target-block-rank-success-paused.log`
  - Result: `state: paused`

Synced suite artifacts:

- `aws-1m-target-block-rank-q500/suite-config.json`
- `aws-1m-target-block-rank-q500/suite-manifest.json`
- `aws-1m-target-block-rank-q500/suite-run.log`
- `aws-1m-target-block-rank-q500/results.jsonl`
- `aws-1m-target-block-rank-q500/register-target-block-rank-snapshot.log`
- `aws-1m-target-block-rank-q500/pipeline-spire-1m-rabitq-target-block-rank-global1152-q500.log`
- `aws-1m-target-block-rank-q500/miss-attribution-spire-1m-global1152-q500.jsonl`
- `aws-1m-target-block-rank-q500/target-block-rank-spire-1m-global1152-q500.jsonl`
- `missed-target-block-ranks.tsv`

Key result rows:

- Retained baseline: `recall@10=0.9832`, `candidate_sum=9,213,846`,
  `heap_rerank_sum=12,500`, `route_sum=48,000`, p50 `288.769 ms`,
  p95 `363.138 ms`, p99 `375.732 ms`.
- Miss attribution: `4916` hit rows, `3` `routing_miss`, `81`
  `selected_leaf_block_pruning_or_candidate_cap`.
- Target block rank status: `4916` selected by cap, `81` ranked outside cap,
  `3` not found in routed leaves.
- Selected-leaf miss rank deltas beyond cap `1152`: `7` within `+128`,
  `30` within `+512`, `58` within `+2048`, `23` farther than `+2048`.

## Failed/Diagnostic Attempts

Earlier attempts are retained for provenance:

- `cloud-resume-task83.log`, `cloud-install-task83.log`,
  `cloud-pause-after-stuck-install.log`, `cloud-status-final-paused.log`
- `cloud-resume-task83-retry.log`, `cloud-install-task83-retry.log`,
  `cloud-pause-after-install-retry-stuck.log`,
  `cloud-status-final-paused-after-retry.log`
- `cloud-install-task83-clean.log`, `cloud-bench-task83-target-block-rank-q500-padded.log`,
  `ssm-task83-padded-bench-failure.json`, `cloud-pause-after-padded-bench-fail.log`,
  `cloud-status-after-padded-bench-fail-paused.log`

The padded run failed before `77cafdacd` with `error retrieving column 17:
error deserializing column 17`; that is the flag-width bug fixed by the code
commit under review.
