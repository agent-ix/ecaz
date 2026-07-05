# Task 30 Packet 1065: AWS Representative Performance Complete

Request review of the completed Phase 13e representative AWS performance packet.

This packet reused the still-loaded packet 1062 Graviton cluster and did not
provision, reinstall, reload, or teardown before the benchmark. The run
started the four stopped instances, opened SSM tunnels, ran the established
`smoke-representative`, `bench-representative-priority`, and
`bench-representative-pooling` targets against prefix `ec_spire_aws_repr_1m`,
then stopped all us-west-2 instances after the verifier pass/fix cycle. The
packet-local `artifacts/aws-stop-verify.log` is empty because AWS returned no
pending/running/stopping EC2 instances after shutdown.

Key evidence:

- `artifacts/smoke-customscan-read.log` shows
  `Custom Scan (EcSpireDistributedScan)`.
- `artifacts/production-read-profile-smoke.log` and
  `artifacts/bench-spire-pipeline-smoke.log` show the production remote
  `remote_heap_candidates` path before the representative suite.
- `artifacts/representative-latency-recall-summary.tsv` captures p50/p95/p99
  latency and recall. Representative k=10 recall reaches `0.9573` at
  nprobe `64`; production read profile k=10 reports coordinator
  p50/p95/p99 `99.298/107.870/117.529 ms` with recall `0.9573`.
- `artifacts/representative-production-profile-summary.tsv` captures the
  q=1000 production read profile. k=10 at nprobe `64` used
  `remote_heap_candidates`, `dispatch_sum=3000`, `socket_open_sum=0`,
  `total_p50=46.000 ms`, `total_p95=49.000 ms`, and zero timeout/cancel/
  degraded skips. k=100 reports recall `0.9334`, `total_p50=48.000 ms`,
  and `total_p95=50.000 ms`.
- `artifacts/representative-pooling-delta-summary.tsv` captures the full
  q=1000 pooled-vs-unpooled comparison: socket opens drop from `3000` to `0`,
  connect p95 drops from `19 ms` to `0 ms`, production total p95 improves from
  `59 ms` to `49 ms`, coordinator latency p95 improves from `120.175 ms` to
  `107.893 ms`, and recall delta is `0`.
- `artifacts/suite-results-representative-priority.jsonl`,
  `artifacts/suite-results-representative-pooling.jsonl`,
  `artifacts/suite-manifest-representative-priority.json`, and
  `artifacts/suite-manifest-representative-pooling.json` are the raw suite
  outputs from `ecaz bench suite`.

Validation:

```text
make -C infra/spire-aws ARTIFACT_DIR=/home/peter/dev/ecaz/reviews/task-30/1065-spire-phase13e-aws-representative-performance-complete/artifacts verify-representative-performance-summary
representative performance summary verified: /home/peter/dev/ecaz/reviews/task-30/1065-spire-phase13e-aws-representative-performance-complete/artifacts nprobes=[latency:8 16 24 32 recall:8 16 24 32 64 production:64 pooling:64]
```

One local verifier fix is included with this packet: the pooling comparison TSV
emits query-latency and production-profile metrics as separate rows per mode,
while the delta TSV merges them. The verifier now accepts that emitted shape
and still requires disabled/enabled latency rows, disabled/enabled production
profile rows, positive socket/latency improvements, and zero recall regression.
