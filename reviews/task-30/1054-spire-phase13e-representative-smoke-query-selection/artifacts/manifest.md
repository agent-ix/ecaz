# Task 30 Packet 1054 Artifact Manifest

- head SHA: `2c63beb9cff923e17671c613943d44d016fc17f2`
- task bucket: `reviews/task-30/1054-spire-phase13e-representative-smoke-query-selection`
- timestamp: `2026-05-28T17:55:50Z`
- lane: Phase 13e AWS representative performance harness
- fixture: Graviton representative AWS failure classification plus local preflight after smoke query fix
- storage format: `rabitq`
- rerank mode: benchmark not reached in failed AWS pass
- table surface: representative prefix `ec_spire_aws_repr_1m`

## Artifacts

- `artifacts/aws-failure/run-representative-performance-pass.log`
  - command: `scripts/spire-aws/run-representative-performance-pass.sh --artifact-dir reviews/task-30/1053-spire-phase13e-aws-representative-performance-after-tunnel-readiness/artifacts --execute`
  - key lines: `profile=ec_real_100k corpus_rows=100000 query_rows=1000`; `published_static_remote_placements`; `smoke-customscan-read.sql:21: error: no rows returned for \gset`; `teardown complete and Terraform state is clean`

- `artifacts/aws-failure/smoke-customscan-read.log`
  - command: `scripts/spire-aws/smoke.sh ...`
  - key result: `no rows returned for \gset` at the query row selection step

- `artifacts/aws-failure/coordinator-load-representative.log`
  - command: node-local coordinator load emitted by `scripts/spire-aws/load.sh representative ...`
  - key result: coordinator loaded 100,000 rows and built `ec_spire_aws_repr_1m_idx`

- `artifacts/aws-failure/placement-remotes.json`
  - command: registration/publish step output
  - key result: remote node IDs `2`, `3`, and `4` were present before smoke

- `artifacts/aws-failure/node-2-remote-materialize.log`
  - command: remote leaf materialization for node 2
  - key result: materialized 34,589 rows

- `artifacts/aws-failure/node-3-remote-materialize.log`
  - command: remote leaf materialization for node 3
  - key result: materialized 32,930 rows

- `artifacts/aws-failure/node-4-remote-materialize.log`
  - command: remote leaf materialization for node 4
  - key result: materialized the node 4 shard before descriptor publication

- `artifacts/aws-failure/coordinator-placement-snapshot-after-remote-publish.log`
  - command: placement snapshot after remote publish
  - key result: one local placement plus remote placements for node IDs 2, 3, and 4

- `artifacts/aws-failure/aws-running-after-failure.log`
  - command: `aws ec2 describe-instances --region us-west-2 --filters Name=instance-state-name,Values=pending,running,stopping ...`
  - key result: no pending/running/stopping EC2 instances printed

- `artifacts/preflight-representative-performance.log`
  - command: `scripts/spire-aws/preflight-representative-performance.sh`
  - key result: representative performance preflight passed after the smoke query selection fix

- `artifacts/aws-running-after-local-gate.log`
  - command: `aws ec2 describe-instances --region us-west-2 --filters Name=instance-state-name,Values=pending,running,stopping ...`
  - key result: no pending/running/stopping EC2 instances printed
