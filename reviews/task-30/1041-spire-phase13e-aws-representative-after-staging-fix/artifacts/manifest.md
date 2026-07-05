# Manifest: AWS Representative Rerun After Staging Fix

- Head SHA under test: `095f0d9e5`
- Task bucket: `reviews/task-30`
- Packet: `reviews/task-30/1041-spire-phase13e-aws-representative-after-staging-fix`
- Timestamp: `2026-05-27 20:43:46-07:00` (`2026-05-28T03:43:46Z`)
- Lane: AWS representative performance, Graviton/aarch64, `us-west-2a`
- Fixture: `qdrant-dbpedia`, prepared as `ec_real_100k`
- Storage format: `rabitq`
- Rerank mode: representative priority/pooling gates were not reached
- Surface: distributed SPIRE placement intended; run failed during node-local coordinator reset before registration/bench
- Isolated one-index-per-table: yes
- Shared-table surfaces: no

## Command

```bash
scripts/spire-aws/run-representative-performance-pass.sh --artifact-dir reviews/task-30/1041-spire-phase13e-aws-representative-after-staging-fix/artifacts --execute
```

## Key Artifacts

- `run-representative-performance-pass-rerun.log`: full AWS pass transcript.
- `aws-topology.json`: provisioned topology.
- `coordinator-load-representative.ssm.json`: failed coordinator node-local load command.
- `coordinator-load-representative-error.log`: extracted failure lines.
- `ec2-post-teardown-verify.log`: direct post-teardown EC2 verification.
- `aws-pass-watchdog.log`: watchdog and teardown record.

## Key Result Lines

```text
Filesystem      Size  Used Avail Use% Mounted on
/dev/nvme0n1p1  200G  4.7G  196G   3% /
ecaz Phase 13e node-local coordinator load representative ssm command id: f283106d-7085-4173-8f30-4d9858d67024
reading pgrx home .pgrx
No such file or directory (os error 2)
failed to run commands: exit status 1
[2026-05-28T03:43:46Z] teardown complete and Terraform state is clean
```

`ec2-post-teardown-verify.log` has no instance table rows, proving no pending/running/stopping/stopped instances remained in `us-west-2` after teardown.
