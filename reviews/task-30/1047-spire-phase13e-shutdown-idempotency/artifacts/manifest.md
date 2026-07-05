# Manifest: Task 30 Packet 1047 Shutdown Idempotency

- head SHA: `eeb47cb1642f1bbbe8d88381c7f1a80cfcee173a`
- task bucket: `reviews/task-30/1047-spire-phase13e-shutdown-idempotency`
- timestamp: 2026-05-28T14:50:43Z
- lane: local-only AWS harness/shutdown hardening; no EC2 provisioning
- fixture: stubbed `AWS_DIR` Makefile and stubbed `aws` CLI
- storage format / rerank mode: not applicable
- isolated one-index-per-table vs shared-table surfaces: not applicable

## Artifacts

### `bash-n.log`

- command: `bash -n scripts/spire-aws/run-pass-with-watchdog.sh scripts/spire-aws/cleanup-residue.sh scripts/spire-aws/check-watchdog-local.sh scripts/spire-aws/check-cleanup-residue-local.sh`
- result: exit 0

### `watchdog-local.log`

- command: `scripts/spire-aws/check-watchdog-local.sh`
- result: exit 0
- key lines:
  - `teardown complete and Terraform state is clean`
  - `SPIRE AWS watchdog local self-check passed`

### `cleanup-residue-local.log`

- command: `scripts/spire-aws/check-cleanup-residue-local.sh`
- result: exit 0
- key lines:
  - `Security group sg-local was already deleted before rule cleanup`
  - `Security group sg-local was already deleted`
  - `SPIRE AWS cleanup residue local self-check passed`

### `aws-running-after-shutdown.log`

- command: `aws ec2 describe-instances --region us-west-2 --filters Name=instance-state-name,Values=pending,running,stopping --query 'Reservations[].Instances[].[InstanceId,State.Name,InstanceType,PrivateIpAddress,Tags[?Key==\`Name\`].Value|[0],Tags[?Key==\`AutoStop\`].Value|[0]]' --output table`
- result: exit 0
- key result: empty output, meaning no pending/running/stopping instances were present in `us-west-2`.
