# Task 84 AWS 1M k=3 Summary Build Manifest

- Task: `plan/tasks/84-spire-1m-recall-recovery-without-candidate-inflation.md`
- Packet: `reviews/task-84/004-aws-1m-k3-summary-build/`
- Branch: `task-84-spire-recall-recovery`
- Code baseline: `3fb62d82d`
- Build/code commits used during this packet:
  - `2719e2bee` packet setup and original suite.
  - `7297e3306` cloud bench uploaded-config refresh.
  - `96554b077` inline medium cloud bench suite configs.
  - `c5b980e82` inline Task 84 suite config.
  - `50b8b2033` query-only suite.
  - `aca92e28a` / `ba206bbf1` suite child-output diagnostic tooling.
- Suite config:
  `reviews/task-84/004-aws-1m-k3-summary-build/suite-aws-1m-k3-summary-build-q500.json`

## Intent

This packet tests the first real multi-representative recovery path enabled by
packet 003. It builds an AWS 1M block16 RaBitQ SPIRE index with three summary
representatives per block and measures q500 recall/candidate/latency at
candidate-preserving and nearby caps.

## Evidence

- `suite-audit.log`
  - Command: `target/debug/ecaz bench suite audit --config reviews/task-84/004-aws-1m-k3-summary-build/suite-aws-1m-k3-summary-build-q500.json --log-file reviews/task-84/004-aws-1m-k3-summary-build/artifacts/suite-audit.log`
  - Result: `[suite:task84-aws-1m-k3-summary-build-q500] audit passed: 7 steps`
- `suite-audit-query-only.log`
  - Command: `target/debug/ecaz bench suite audit --config reviews/task-84/004-aws-1m-k3-summary-build/suite-aws-1m-k3-summary-query-only-q500.json --log-file reviews/task-84/004-aws-1m-k3-summary-build/artifacts/suite-audit-query-only.log`
  - Result: `[suite:task84-aws-1m-k3-summary-query-only-q500] audit passed: 4 steps`
- `s3-20260606T165349Z-build-spire-1m-rabitq-k3-block16-tg256.log`
  - Source: `s3://ecaz-cloud-1m-b62eb804/bench-artifacts/task84-aws-1m-k3-summary-build-q500/20260606T165349Z/build-spire-1m-rabitq-k3-block16-tg256.log`
  - Result: `CREATE INDEX`; index size `936 MB`.
  - Build timing: `total_ms=1713717`, `heap_scan_ms=601404`,
    `draft_ms=1061544`, `top_graph_ms=22674`.
- `s3-20260606T165349Z-suite-manifest.json`
  - Source: original build suite manifest.
  - Statuses: precheck succeeded, enriched target-block registration
    succeeded, k=3 build succeeded, first `global1024` pipeline failed.
- `s3-20260606T181133Z-query-only-suite-manifest.json`
  - Source: query-only rerun manifest.
  - Status: first `global1024` pipeline failed before recall/candidate rows.
- `ssm-61026420-query-only-errorlog.json`
  - Source: SSM command invocation for diagnostic query-only run.
  - Root error:
    `ERROR: ec_spire_distributed: relation context could not be loaded`
    with ADR-069 hint while executing the SPIRE query-metrics KNN query.
- Cloud lifecycle logs:
  - `cloud-status-before-k3-summary-build.log`: initial state `paused`.
  - `cloud-status-final-paused-k3-query-only-errorlog.log`: final state
    `paused`, cost `~$0.00/hr running`, retained storage `~$8.00/mo`.

## Result

This packet proves the k=3 summary index can be built on AWS 1M, but it does
not produce valid recall/candidate comparisons. The retained k=3 index became
unusable for the SPIRE query pipeline after later install/restart activity
because distributed relation context could not be loaded.
