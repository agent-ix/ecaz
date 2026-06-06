# Task 84 AWS 1M k=3 Summary Build Manifest

- Task: `plan/tasks/84-spire-1m-recall-recovery-without-candidate-inflation.md`
- Packet: `reviews/task-84/004-aws-1m-k3-summary-build/`
- Branch: `task-84-spire-recall-recovery`
- Code baseline: `3fb62d82d`
- Suite config:
  `reviews/task-84/004-aws-1m-k3-summary-build/suite-aws-1m-k3-summary-build-q500.json`

## Intent

This packet tests the first real multi-representative recovery path enabled by
packet 003. It builds an AWS 1M block16 RaBitQ SPIRE index with three summary
representatives per block and measures q500 recall/candidate/latency at
candidate-preserving and nearby caps.

## Planned Evidence

- `suite-audit.log`
  - Command: `target/debug/ecaz bench suite audit --config reviews/task-84/004-aws-1m-k3-summary-build/suite-aws-1m-k3-summary-build-q500.json --log-file reviews/task-84/004-aws-1m-k3-summary-build/artifacts/suite-audit.log`
  - Result: `[suite:task84-aws-1m-k3-summary-build-q500] audit passed: 7 steps`
- AWS status before run.
- AWS resume/install/build/bench logs.
- q500 pipeline outputs for `global1024`, `global1152`, and `global1280`.
- Miss attribution and enriched target-block context JSONL for each cap.
- Storage evidence for the k=3 index.
- Final AWS paused status.
