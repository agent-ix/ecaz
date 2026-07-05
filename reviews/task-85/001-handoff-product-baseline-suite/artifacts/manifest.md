# Task 85 Handoff and Product Baseline Suite Manifest

- Packet: `reviews/task-85/001-handoff-product-baseline-suite/`
- Branch: `task-85-spire-product-scale-pareto`
- Suite config:
  `reviews/task-85/001-handoff-product-baseline-suite/suite-aws-1m-product-baseline-q500.json`

## Handoff Evidence

- `reviews/task-84/006-closeout-no-bounded-recovery/`: no accepted Task 84
  recall-recovery policy.
- `reviews/task-84/007-latency-retention-aws-k2-k3-control/`: no Task 84
  latency mechanism beats warmed retained k2 at the retained recall/candidate
  surface.
- `reviews/task-84/007-latency-retention-aws-k2-k3-control/feedback/2026-06-06-01-reviewer.md`:
  reviewer accepted the paired warmup-controlled interpretation.

## Baseline Suite

The suite is prepared to run on AWS profile `1m` and database `postgres`.
It preserves the Task 79/81 retained surface:

- prefix: `task67_1m_hnsw_m7g2xlarge`
- index: `aws_spire_1m_rabitq_t80_block16_tg256_idx`
- q500 truth cache:
  `benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/truth-aws-real-1m-q500-k10.json`
- retained settings: `nprobe=96`, `rerank_width=25`,
  `global_blocks=1152`, route prior `0.0`

It also includes Task 83 blanket-cap controls at `global1280` and
`global1536`, plus a storage step.

## Validation Log

- `suite-audit.log`
  - Command: `target/debug/ecaz bench suite audit --config reviews/task-85/001-handoff-product-baseline-suite/suite-aws-1m-product-baseline-q500.json --log-file reviews/task-85/001-handoff-product-baseline-suite/artifacts/suite-audit.log`
  - Result: `[suite:task85-aws-1m-product-baseline-q500] audit passed: 6 steps`

## AWS Lifecycle Attempt

- `cloud-status-before-baseline.log`
  - Command: `target/debug/ecaz cloud status --profile 1m --database postgres --log-file reviews/task-85/001-handoff-product-baseline-suite/artifacts/cloud-status-before-baseline.log`
  - Result: profile `1m` paused.
- `cloud-resume-baseline.log`
  - Command: `target/debug/ecaz cloud resume --profile 1m --database postgres --log-file reviews/task-85/001-handoff-product-baseline-suite/artifacts/cloud-resume-baseline.log`
  - Result: resume completed; database `10.42.1.131` ready.
- `cloud-install-baseline.log`
  - Command: `target/debug/ecaz cloud install --profile 1m --database postgres --git-ref task-85-spire-product-scale-pareto --skip-extension-recreate --log-file reviews/task-85/001-handoff-product-baseline-suite/artifacts/cloud-install-baseline.log`
  - Result: no verifiable completion from the local command during polling; no
    benchmark was run from this state.
- `cloud-status-during-install.log`
  - Result: profile `1m` was running during the stalled install wait.
- `cloud-pause-after-install-stall.log`
  - Command: `target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-85/001-handoff-product-baseline-suite/artifacts/cloud-pause-after-install-stall.log`
  - Result: pause requested for db and loader instances.
- `cloud-status-after-install-stall-pause.log`
  - Command: `target/debug/ecaz cloud status --profile 1m --database postgres --log-file reviews/task-85/001-handoff-product-baseline-suite/artifacts/cloud-status-after-install-stall-pause.log`
  - Result: profile `1m` paused.
- `cloud-status-final-paused-after-stall.log`
  - Command: `target/debug/ecaz cloud status --profile 1m --database postgres --log-file reviews/task-85/001-handoff-product-baseline-suite/artifacts/cloud-status-final-paused-after-stall.log`
  - Result: profile `1m` paused.

No AWS benchmark result is claimed yet. The next Task 85 checkpoint should run
the audited suite after a verifiable branch install, or skip install only with a
documented reason that the remote binary already matches this branch.

## No-Reinstall Bench Attempt

Because Task 85 had no executable code changes beyond packet files, a second
attempt skipped install and tried to run the audited suite against the existing
remote `/usr/local/bin/ecaz`:

- `cloud-resume-for-baseline-suite.log`
  - Command: `target/debug/ecaz cloud resume --profile 1m --database postgres --log-file reviews/task-85/001-handoff-product-baseline-suite/artifacts/cloud-resume-for-baseline-suite.log`
  - Result: resume completed; database `10.42.1.131` ready.
- `cloud-bench-product-baseline.log`
  - Command: `target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-85/001-handoff-product-baseline-suite/suite-aws-1m-product-baseline-q500.json --suite task85-aws-1m-product-baseline-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-85/001-handoff-product-baseline-suite/artifacts/cloud-bench-product-baseline.log`
  - Result: no verifiable completion from the local command during polling; no
    fresh current SSM invocation was visible in the db-instance history.
- `cloud-pause-after-bench-stall.log`
  - Command: `target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-85/001-handoff-product-baseline-suite/artifacts/cloud-pause-after-bench-stall.log`
  - Result: pause requested for db and loader instances.
- `cloud-status-after-bench-stall-pause.log`
  - Result: first post-pause status observed `stopping`.
- `cloud-status-final-paused-after-bench-stall.log`
  - Command: `target/debug/ecaz cloud status --profile 1m --database postgres --log-file reviews/task-85/001-handoff-product-baseline-suite/artifacts/cloud-status-final-paused-after-bench-stall.log`
  - Result: profile `1m` paused.

This confirmed Task 85 needed a cloud-wrapper observability/timeout checkpoint
before spending more AWS time on the product baseline.

## Diagnosed SSM Inline Config Stall

After packet 002 added SSM command IDs and polling timeouts, another run showed
the command was reaching the instance but stalling while writing the 9301-byte
suite config through an inline heredoc:

- `cloud-resume-after-ssm-timeout-fix.log`
  - Command: `target/debug/ecaz cloud resume --profile 1m --database postgres --log-file reviews/task-85/001-handoff-product-baseline-suite/artifacts/cloud-resume-after-ssm-timeout-fix.log`
  - Result: resume completed; database `10.42.1.131` ready.
- `cloud-bench-product-baseline-after-ssm-timeout-fix.log`
  - Command: `target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-85/001-handoff-product-baseline-suite/suite-aws-1m-product-baseline-q500.json --suite task85-aws-1m-product-baseline-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-85/001-handoff-product-baseline-suite/artifacts/cloud-bench-product-baseline-after-ssm-timeout-fix.log`
  - Result: SSM command `76ef8d52-4caa-462b-9d97-3ee836a14557` was cancelled
    after no progress; stderr showed it stopped at `cat` for the inline suite
    config.
- `cloud-pause-after-cancelled-ssm-bench.log`
  - Command: `target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-85/001-handoff-product-baseline-suite/artifacts/cloud-pause-after-cancelled-ssm-bench.log`
  - Result: pause requested for db and loader instances.
- `cloud-status-final-paused-after-cancelled-ssm-bench.log`
  - Command: `target/debug/ecaz cloud status --profile 1m --database postgres --log-file reviews/task-85/001-handoff-product-baseline-suite/artifacts/cloud-status-final-paused-after-cancelled-ssm-bench.log`
  - Result: profile `1m` paused.

Packet 002 then changed cloud bench to upload suite configs larger than 6000
bytes through S3 instead of inline SSM script text.

## Successful Product Baseline Run

- Head SHA: `22310bc92` (`Upload larger cloud bench suite configs`)
- Task bucket: `reviews/task-85/001-handoff-product-baseline-suite/`
- Lane / fixture: AWS profile `1m`, q500, prefix
  `task67_1m_hnsw_m7g2xlarge`, database `postgres`
- Storage format: `rabitq`
- Rerank mode: `rerank_width=25`, heap rerank ready sum `12,500` per 500-query
  pipeline row
- Isolated surface: one retained SPIRE index per table surface
  (`aws_spire_1m_rabitq_t80_block16_tg256_idx`) plus the Task84 k3 control
  index for storage comparison; not a shared-table multi-index run.

Artifacts:

- `cloud-resume-after-config-upload-fix.log`
  - Command: `target/debug/ecaz cloud resume --profile 1m --database postgres --log-file reviews/task-85/001-handoff-product-baseline-suite/artifacts/cloud-resume-after-config-upload-fix.log`
  - Timestamp: `2026-06-07T06:00:21Z` suite run key
  - Result: resume completed; database `10.42.1.131` ready.
- `cloud-bench-product-baseline-after-config-upload-fix.log`
  - Command: `target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-85/001-handoff-product-baseline-suite/suite-aws-1m-product-baseline-q500.json --suite task85-aws-1m-product-baseline-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-85/001-handoff-product-baseline-suite/artifacts/cloud-bench-product-baseline-after-config-upload-fix.log`
  - Result: synced artifacts from
    `s3://ecaz-cloud-1m-b62eb804/bench-artifacts/task85-aws-1m-product-baseline-q500/20260607T060021Z/`.
- `aws-1m-product-baseline-q500/suite-manifest.json`
  - Command: generated by `ecaz bench suite run` on the AWS database host.
  - Result: 6 steps succeeded, 0 failed.
- `aws-1m-product-baseline-q500/results.jsonl`
  - Command: generated by `ecaz bench suite run`.
  - Key rows:
    - retained global1152 first: recall@10 `0.9832`, p50 `264.946 ms`,
      p95 `328.183 ms`, p99 `338.347 ms`, candidate_sum `9,213,846`,
      heap_rerank_sum `12,500`.
    - retained global1152 warm repeat: recall@10 `0.9832`, p50 `246.397 ms`,
      p95 `304.476 ms`, p99 `321.342 ms`, candidate_sum `9,213,846`,
      heap_rerank_sum `12,500`.
    - Task83 global1280 control: recall@10 `0.9846`, p50 `255.151 ms`,
      p95 `309.259 ms`, p99 `325.029 ms`, candidate_sum `10,237,554`,
      heap_rerank_sum `12,500`.
    - Task83 global1536 control: recall@10 `0.9876`, p50 `272.482 ms`,
      p95 `327.933 ms`, p99 `337.447 ms`, candidate_sum `12,284,852`,
      heap_rerank_sum `12,500`.
- `aws-1m-product-baseline-q500/suite-report.md`
  - Command: `target/debug/ecaz bench suite report --manifest reviews/task-85/001-handoff-product-baseline-suite/artifacts/aws-1m-product-baseline-q500/suite-manifest.json --results-output reviews/task-85/001-handoff-product-baseline-suite/artifacts/aws-1m-product-baseline-q500/results-report.jsonl --log-file reviews/task-85/001-handoff-product-baseline-suite/artifacts/aws-1m-product-baseline-q500/suite-report.md`
  - Result: 6 completed, 0 failed, 0 skipped, 0 stale.
- `aws-1m-product-baseline-q500/storage-retained-spire-1m-rabitq.log`
  - Command: generated by suite storage step.
  - Key rows: total `18.4 GiB`; retained k2 SPIRE index `872.1 MiB`
    (`923.7 B` per row); Task84 k3 SPIRE index `936.4 MiB` (`991.9 B` per
    row).
- `cloud-pause-after-product-baseline.log`
  - Command: `target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-85/001-handoff-product-baseline-suite/artifacts/cloud-pause-after-product-baseline.log`
  - Result: pause requested and wrapper reported `pause: profile=1m stopped
    (db + loader)`.
- `cloud-status-final-stopped-after-product-baseline.log`
  - Command: `target/debug/ecaz cloud status --profile 1m --database postgres --log-file reviews/task-85/001-handoff-product-baseline-suite/artifacts/cloud-status-final-stopped-after-product-baseline.log`
  - Result: profile `1m` state `paused`, cost `$0.00/hr running`.
