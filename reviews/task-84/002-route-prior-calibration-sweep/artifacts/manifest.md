# Task 84 Route Prior Calibration Sweep Manifest

- Task: `plan/tasks/84-spire-1m-recall-recovery-without-candidate-inflation.md`
- Packet: `reviews/task-84/002-route-prior-calibration-sweep/`
- Branch: `task-84-spire-recall-recovery`
- Baseline evidence: `reviews/task-84/001-enriched-block-context-diagnostic/`
- Suite config:
  `reviews/task-84/002-route-prior-calibration-sweep/suite-aws-1m-route-prior-calibration-q500.json`

## Hypothesis

The packet tests whether route-aware block scoring can reorder the fixed
`global1152` block cap toward truth-containing target blocks without increasing
the global block budget. This follows packet 001, where `52/81`
selected-leaf misses were in the top 24 routed leaves and `71/81` were in the
top 48, while only `26/81` were within `0.01` of the score cap.

## Planned Evidence

- `suite-audit.log`
  - Command: `target/debug/ecaz bench suite audit --config reviews/task-84/002-route-prior-calibration-sweep/suite-aws-1m-route-prior-calibration-q500.json --log-file reviews/task-84/002-route-prior-calibration-sweep/artifacts/suite-audit.log`
  - Result: `[suite:task84-aws-1m-route-prior-calibration-q500] audit passed: 5 steps`
- Run the suite on AWS `1m` with q500 truth.
- Record `recall@10`, `candidate_sum`, `heap_rerank_sum`, p50, p95, p99, and
  miss attribution for each route-prior point.
- Pause AWS `1m` after the run and capture final status.
