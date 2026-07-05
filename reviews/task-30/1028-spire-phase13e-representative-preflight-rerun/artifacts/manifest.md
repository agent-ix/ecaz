# Artifact Manifest

- Head SHA: `1810830a6`
- Task bucket: `reviews/task-30/1028-spire-phase13e-representative-preflight-rerun`
- Timestamp: `2026-05-27T10:59:32-07:00`
- Lane: Phase 13e representative performance readiness
- Fixture / storage / rerank mode: local preflight only; no benchmark fixture
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `preflight-representative-performance.log`

- Command: `bash scripts/spire-aws/preflight-representative-performance.sh`
- Result: passed.
- Key line:
  - `SPIRE representative performance preflight passed: priority=/home/peter/dev/ecaz/scripts/spire-aws/suite-representative-priority.json pooling=/home/peter/dev/ecaz/scripts/spire-aws/suite-representative-pooling.json`

### `aws-us-west-2-nonterminated-count.log`

- Command: `aws ec2 describe-instances --region us-west-2 --filters Name=instance-state-name,Values=pending,running,stopping,stopped --query 'length(Reservations[].Instances[])' --output text`
- Result: passed.
- Key line:
  - `0`

### `aws-us-west-2-nonterminated-instances.log`

- Command: `aws ec2 describe-instances --region us-west-2 --filters Name=instance-state-name,Values=pending,running,stopping,stopped --query 'Reservations[].Instances[].{InstanceId:InstanceId,State:State.Name,Type:InstanceType,Az:Placement.AvailabilityZone,Name:Tags[?Key==`Name`]|[0].Value}' --output table`
- Result: passed.
- Key result: no rows returned for pending/running/stopping/stopped instances.

## Notes

No AWS provisioning command was run. The existing untracked SPIRE artifact
directory under `scripts/spire-aws/artifacts/` was left untouched and was not
staged.
