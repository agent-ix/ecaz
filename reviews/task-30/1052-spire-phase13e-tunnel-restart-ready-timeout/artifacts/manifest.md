# Task 30 Packet 1052 Artifact Manifest

- head SHA: `b9fe580957c80948c475b7fd759ece51c294111e`
- task bucket: `reviews/task-30/1052-spire-phase13e-tunnel-restart-ready-timeout`
- timestamp: `2026-05-28T16:47:07Z`
- lane: Phase 13e AWS representative performance harness
- fixture: Graviton AWS lane failure evidence plus local tunnel restart regression gate
- storage format: `rabitq`
- rerank mode: benchmark not reached in failed AWS pass
- table surface: representative pass uses isolated prefix `ec_spire_aws_repr_1m`; AWS failure stopped before remote shard load/benchmark summaries

## Artifacts

- `artifacts/aws-failure/run-representative-performance-pass.log`
  - command: `scripts/spire-aws/run-representative-performance-pass.sh --artifact-dir reviews/task-30/1051-spire-phase13e-aws-representative-performance-rerun/artifacts --execute`
  - key lines: `profile=ec_real_100k corpus_rows=100000 query_rows=1000`; `load-representative Error 1`; `teardown complete and Terraform state is clean`

- `artifacts/aws-failure/coordinator-load-representative.log`
  - command: node-local coordinator SSM load emitted by `scripts/spire-aws/load.sh representative ...`
  - key lines: `corpus: 100000 rows`; `built ec_spire_aws_repr_1m_idx in 134.58s`; `completed prefix ec_spire_aws_repr_1m in 201.53s`

- `artifacts/aws-failure/tunnel-restart-after-node-local-load.log`
  - command: coordinator tunnel restart hook after node-local coordinator load
  - key lines: attempts 1-4 timed out waiting for the ready log; attempts 5 and 7 saw `bind: address already in use`; final status timed out waiting for coordinator restart on `127.0.0.1:15432`

- `artifacts/aws-failure/tunnel-coordinator-restart-attempt-5.log`
  - command: SSM coordinator restart attempt 5
  - key line: `Cannot perform start session: listen tcp 127.0.0.1:15432: bind: address already in use`

- `artifacts/aws-failure/tunnel-coordinator-restart-attempt-7.log`
  - command: SSM coordinator restart attempt 7
  - key line: `Cannot perform start session: listen tcp 127.0.0.1:15432: bind: address already in use`

- `artifacts/aws-failure/aws-running-after-failure.log`
  - command: `aws ec2 describe-instances --region us-west-2 --filters Name=instance-state-name,Values=pending,running,stopping ...`
  - key result: no pending/running/stopping EC2 instances printed

- `artifacts/load-tunnel-restart-local.log`
  - command: `scripts/spire-aws/check-load-tunnel-restart-local.sh`
  - key result: `SPIRE AWS load tunnel restart local self-check passed`

- `artifacts/representative-pass-dry-run.log`
  - command: `scripts/spire-aws/run-representative-performance-pass.sh --artifact-dir reviews/task-30/1052-spire-phase13e-tunnel-restart-ready-timeout/artifacts`
  - key result: Graviton operator preflight passed (`m7g.large`, `us-west-2a`, remote count 3), state and permissions preflight passed, representative performance preflight passed, no provisioning

- `artifacts/aws-running-after-local-gate.log`
  - command: `aws ec2 describe-instances --region us-west-2 --filters Name=instance-state-name,Values=pending,running,stopping ...`
  - key result: no pending/running/stopping EC2 instances printed
