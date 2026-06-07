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

This confirms Task 85 needs a cloud-wrapper observability/timeout checkpoint
before spending more AWS time on the product baseline.
