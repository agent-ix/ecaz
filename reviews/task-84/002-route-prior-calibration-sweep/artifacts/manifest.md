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
- AWS status before run: `cloud-status-before-route-prior.log`
  - Result: `state: paused`
- Resume log: `cloud-resume-route-prior.log`
  - Result: `resume: profile=1m db=10.42.1.131 ready`
- Install log: `cloud-install-route-prior.log`
  - Command: `target/debug/ecaz cloud install --profile 1m --database postgres --git-ref c6e1cd27a --skip-extension-recreate --clean-cargo-target --timeout 3600 --log-file reviews/task-84/002-route-prior-calibration-sweep/artifacts/cloud-install-route-prior.log`
  - Result: `install: profile=1m db=10.42.1.131 ref=c6e1cd27a ok`
- Bench log: `cloud-bench-route-prior.log`
  - Command: `target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-84/002-route-prior-calibration-sweep/suite-aws-1m-route-prior-calibration-q500.json --suite task84-aws-1m-route-prior-calibration-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-84/002-route-prior-calibration-sweep/artifacts/cloud-bench-route-prior.log`
  - Result: synced artifacts from `s3://ecaz-cloud-1m-b62eb804/bench-artifacts/task84-aws-1m-route-prior-calibration-q500/20260606T153929Z/`
- Pause log: `cloud-pause-after-route-prior.log`
- Status logs:
  - `cloud-status-after-route-prior.log`: captured while EC2 was `stopping`.
  - `cloud-status-final-paused-route-prior.log`: final status `state: paused`.

## Synced AWS Artifacts

- `aws-1m-route-prior-calibration-q500/suite-config.json`
- `aws-1m-route-prior-calibration-q500/suite-manifest.json`
- `aws-1m-route-prior-calibration-q500/suite-run.log`
- `aws-1m-route-prior-calibration-q500/results.jsonl`
- `aws-1m-route-prior-calibration-q500/register-enriched-target-block-context.log`
- `aws-1m-route-prior-calibration-q500/pipeline-spire-1m-rabitq-route-prior-002-global1152-q500.log`
- `aws-1m-route-prior-calibration-q500/pipeline-spire-1m-rabitq-route-prior-005-global1152-q500.log`
- `aws-1m-route-prior-calibration-q500/pipeline-spire-1m-rabitq-route-prior-010-global1152-q500.log`
- `aws-1m-route-prior-calibration-q500/pipeline-spire-1m-rabitq-route-prior-020-global1152-q500.log`
- `aws-1m-route-prior-calibration-q500/miss-attribution-spire-1m-route-prior-002-global1152-q500.jsonl`
- `aws-1m-route-prior-calibration-q500/miss-attribution-spire-1m-route-prior-005-global1152-q500.jsonl`
- `aws-1m-route-prior-calibration-q500/miss-attribution-spire-1m-route-prior-010-global1152-q500.jsonl`
- `aws-1m-route-prior-calibration-q500/miss-attribution-spire-1m-route-prior-020-global1152-q500.jsonl`
- `aws-1m-route-prior-calibration-q500/target-block-context-spire-1m-route-prior-002-global1152-q500.jsonl`
- `aws-1m-route-prior-calibration-q500/target-block-context-spire-1m-route-prior-005-global1152-q500.jsonl`
- `aws-1m-route-prior-calibration-q500/target-block-context-spire-1m-route-prior-010-global1152-q500.jsonl`
- `aws-1m-route-prior-calibration-q500/target-block-context-spire-1m-route-prior-020-global1152-q500.jsonl`
- `route-prior-summary.tsv`

## Key Rows

- `route_prior=0.02`: `recall@10=0.9832`, `candidate_sum=9,213,802`,
  `heap_rerank_sum=12,500`, p50 `279.228 ms`, p95 `351.570 ms`, p99
  `364.541 ms`.
- `route_prior=0.05`: `recall@10=0.9832`, `candidate_sum=9,213,740`,
  `heap_rerank_sum=12,500`, p50 `256.523 ms`, p95 `320.070 ms`, p99
  `334.609 ms`.
- `route_prior=0.10`: `recall@10=0.9832`, `candidate_sum=9,213,619`,
  `heap_rerank_sum=12,500`, p50 `255.472 ms`, p95 `317.784 ms`, p99
  `334.127 ms`.
- `route_prior=0.20`: `recall@10=0.9832`, `candidate_sum=9,213,310`,
  `heap_rerank_sum=12,500`, p50 `255.583 ms`, p95 `319.458 ms`, p99
  `334.041 ms`.
- Miss attribution was unchanged at every route-prior point: `4916` hit rows,
  `3` `routing_miss`, `81` `selected_leaf_block_pruning_or_candidate_cap`.
- Selected-leaf miss identity had zero symmetric-diff rows versus packet 001 for
  all four route-prior points.
- Target-block context outside-cap counts increased with route-prior weight:
  `84`, `86`, `91`, `109`.

## Conclusion

Route-prior weighting is rejected as a Task 84 recall-recovery policy. It is
latency-positive in this AWS run, but it does not recover any selected-leaf
misses and higher weights displace more truth target blocks from the fixed cap.
