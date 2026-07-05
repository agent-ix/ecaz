# Manifest: Task 30 / 1043 SPIRE Phase 13e AWS Representative After psql Fix

- head SHA at run start: `d81acceb0e6ea7fd76702ebfd02528e2491f01ec`
- task bucket: `reviews/task-30`
- packet path: `reviews/task-30/1043-spire-phase13e-aws-representative-after-psql-fix`
- timestamp: `2026-05-28T03:48:23Z`
- lane: AWS representative performance pass, Graviton `m7g.large`, `us-west-2a`, 1 coordinator + 3 remotes
- fixture: representative AWS SPIRE topology
- storage format: SPIRE AWS harness package/install stage only
- rerank mode: not reached
- isolated one-index-per-table or shared-table surface: not reached

## Artifacts

### `artifacts/run-representative-performance-pass.log`

- command: `script -q -e -c "scripts/spire-aws/run-representative-performance-pass.sh --artifact-dir reviews/task-30/1043-spire-phase13e-aws-representative-after-psql-fix/artifacts --execute" reviews/task-30/1043-spire-phase13e-aws-representative-after-psql-fix/artifacts/run-representative-performance-pass.log`
- result: operator-interrupted before load/benchmark stage
- key result lines:
  - `Apply complete! Resources: 33 added, 0 changed, 0 destroyed.`
  - `ssm_online instance_count=4 status=ready`
  - `i-03312737f946a8485 ssm command id: 3330ff1d-1b2a-4159-b169-53f064907f38`
  - `i-022965f4aa10f3b89 ssm command id: f983384d-d1b3-49ab-84a4-272f39b19bc5`
  - `i-07463f9a766f8692e ssm command id: c8668e6a-932f-49ec-894a-e3e09d3430aa`
  - `i-0756c05b5f48ca4d7 ssm command id: 158d1db9-4344-480d-9287-1aa55008c06a`

### `artifacts/install.log`

- command: produced by `scripts/spire-aws/install.sh`
- result: SSM became ready and all four install commands were submitted
- key result: `ssm_online instance_count=4 status=ready`

### `artifacts/install-i-*.log`

- command: per-node `aws ssm get-command-invocation` install transcripts
- result: coordinator, remote-1, and remote-2 install logs were captured before shutdown
- key result: each captured node log ended with successful release build and PostgreSQL service setup.

### `artifacts/aws-pass-watchdog.log`

- command: produced by `scripts/spire-aws/run-pass-with-watchdog.sh`
- result: teardown completed after operator shutdown
- key result lines:
  - `Destroy complete! Resources: 33 destroyed.`
  - `SPIRE AWS state preflight passed: local Terraform state has no managed resources`
  - `[2026-05-28T04:56:26Z] teardown complete and Terraform state is clean`

### `artifacts/ec2-running-after-shutdown.log`

- command: `script -q -e -c "aws ec2 describe-instances --region us-west-2 --filters Name=instance-state-name,Values=pending,running,stopping --query 'Reservations[].Instances[].[InstanceId,State.Name,InstanceType,PrivateIpAddress,Tags[?Key==\`Name\`].Value|[0]]' --output text" reviews/task-30/1043-spire-phase13e-aws-representative-after-psql-fix/artifacts/ec2-running-after-shutdown.log`
- result: passed
- key result: no pending/running/stopping instances were returned.

## Omitted Local-Only Payloads

The packet directory still contains local generated source/package trees and tarballs from the interrupted install run. They are intentionally not part of the durable review evidence because the relevant proof is in the packet-local logs above.
