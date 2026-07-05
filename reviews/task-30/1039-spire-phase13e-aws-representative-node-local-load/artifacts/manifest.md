# Manifest: AWS Representative Node-Local Load Failure

- Head SHA under test: `1a4fc0df075e0f1df0d5de65db6a17175cc3117b`
- Task bucket: `reviews/task-30`
- Packet: `reviews/task-30/1039-spire-phase13e-aws-representative-node-local-load`
- Timestamp: `2026-05-27 19:02:03-07:00` (`2026-05-28T02:02:03Z`)
- Lane: AWS representative performance, Graviton/aarch64, `us-west-2a`
- Fixture: `qdrant-dbpedia` representative profile, prepared as `ec_real_100k`
- Storage format: `rabitq`
- Rerank mode: production-read representative priority/pooling gates were not reached
- Surface: distributed SPIRE placement intended; run failed during node-local coordinator load before registration/bench
- Isolated one-index-per-table: yes
- Shared-table surfaces: no

## Command

```bash
scripts/spire-aws/run-representative-performance-pass.sh --artifact-dir reviews/task-30/1039-spire-phase13e-aws-representative-node-local-load/artifacts --execute
```

## Key Artifacts

- `run-representative-performance-pass.log`: full AWS pass transcript.
- `aws-topology.json`: provisioned topology.
- `coordinator-load-representative.ssm.json`: failed coordinator SSM load command.
- `coordinator-load-representative-error.log`: extracted error summary.
- `ec2-post-teardown-verify.log`: direct post-teardown EC2 check.
- `aws-pass-watchdog.log`: watchdog/teardown record.

## Key Result Lines

```text
ecaz Phase 13e node-local coordinator load representative ssm command id: d286869a-a78a-4d9e-911e-c343ff9dd580
download failed: s3://ecaz-spire-aws-20260528002751517400000007/representative-load/representative/coordinator/ec_real_100k_corpus.tsv to ../../tmp/ecaz-spire-aws-representative-coordinator/ec_real_100k_corpus.tsv [Errno 28] No space left on device
failed to run commands: exit status 1
[2026-05-28T02:02:03Z] teardown complete and Terraform state is clean
```

`ec2-post-teardown-verify.log` has no instance table rows, which means no pending/running/stopping/stopped instances matched after teardown.
