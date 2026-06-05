# Task 83 Target-Block Rank Diagnostic Manifest

- Task: `plan/tasks/83-spire-selected-block-containment-recovery.md`
- Packet: `reviews/task-83/001-target-block-rank-diagnostic/`
- Code commit under review: `ee9ee8615`
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

## AWS Attempt

AWS `1m` was resumed twice to install and run the q500 target-block-rank suite,
but `target/debug/ecaz cloud install` remained silent before writing its
packet-local log on both attempts:

- First attempt:
  - Resume log: `cloud-resume-task83.log`
  - Install log: `cloud-install-task83.log` (empty)
  - Pause log: `cloud-pause-after-stuck-install.log`
  - Status after pause: `cloud-status-final-paused.log`
- Retry attempt:
  - Resume log: `cloud-resume-task83-retry.log`
  - Install log: `cloud-install-task83-retry.log` (empty)
  - Pause log: `cloud-pause-after-install-retry-stuck.log`
  - Final status: `cloud-status-final-paused-after-retry.log`

The retry install was allowed to run for about six minutes and was then stopped
locally because the AWS profile was running at hourly cost while no install log
or terminal output was produced. Final AWS state is `paused`.

## Pending Evidence

The q500 AWS target-block-rank distribution has not run yet. The next step is
to debug why `cloud install` blocks before log output, then rerun:

```text
target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-83/001-target-block-rank-diagnostic/suite-aws-1m-target-block-rank-q500.json --suite task83-aws-1m-target-block-rank-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-83/001-target-block-rank-diagnostic/artifacts/cloud-bench-task83-target-block-rank-q500.log
```
