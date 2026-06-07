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
