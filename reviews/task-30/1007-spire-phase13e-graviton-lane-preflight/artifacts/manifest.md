# Manifest: SPIRE AWS Graviton Lane Preflight

- head SHA: `8497313f588bca7809d901d118e282e6a0df737e`
- task bucket: `reviews/task-30/`
- packet path: `reviews/task-30/1007-spire-phase13e-graviton-lane-preflight/`
- timestamp: `2026-05-27T15:34:45Z`
- lane: SPIRE Phase 13e AWS Graviton/aarch64 guard
- fixture: packet-local fake `aws ec2 describe-images`
- storage format: not applicable
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surface: not applicable
- AWS provisioning: not run
- real AWS API calls: not run for operator-preflight lane tests

## Artifacts

### `artifacts/fake-bin/aws`

- command: packet-local fake for `aws ec2 describe-images`
- key result: returns `arm64`

### `artifacts/tfvars/good.tfvars`

- command: input fixture for the established lane
- key result: `us-west-2`, `us-west-2a`, `m7g.large`, `remote_count=3`

### `artifacts/tfvars/bad-region.tfvars`

- command: input fixture for a non-standard region
- key result: `us-east-1`, expected rejection

### `artifacts/tfvars/bad-instance.tfvars`

- command: input fixture for a non-Graviton/x86-like instance family
- key result: `r6i.large`, expected rejection

### `artifacts/bash-n-preflight-operator.log`

- command: `bash -n scripts/spire-aws/preflight-operator.sh`
- timestamp: `2026-05-27 08:34:10-07:00`
- key result: `COMMAND_EXIT_CODE="0"`

### `artifacts/preflight-good.log`

- command: `PATH="$PWD/.../fake-bin:$PATH" scripts/spire-aws/preflight-operator.sh artifacts/tfvars/good.tfvars`
- timestamp: `2026-05-27 08:34:10-07:00`
- key result: `COMMAND_EXIT_CODE="0"`
- cited line: `SPIRE AWS operator preflight passed: region=us-west-2 az=us-west-2a ami=ami-0123456789abcdef0 coordinator=m7g.large remote=m7g.large remote_count=3`

### `artifacts/preflight-bad-region.log`

- command: `PATH="$PWD/.../fake-bin:$PATH" scripts/spire-aws/preflight-operator.sh artifacts/tfvars/bad-region.tfvars`
- timestamp: `2026-05-27 08:34:10-07:00`
- key result: `COMMAND_EXIT_CODE="2"`
- cited line: `ERROR: region must match the established Phase 13e Graviton lane (us-west-2); got: us-east-1. Amend the task/runbook before changing AWS lane.`

### `artifacts/preflight-bad-instance.log`

- command: `PATH="$PWD/.../fake-bin:$PATH" scripts/spire-aws/preflight-operator.sh artifacts/tfvars/bad-instance.tfvars`
- timestamp: `2026-05-27 08:34:10-07:00`
- key result: `COMMAND_EXIT_CODE="2"`
- cited line: `ERROR: coordinator_instance_type must use the established Graviton/aarch64 lane, got: r6i.large`

### `artifacts/preflight.log`

- command: `make -C infra/spire-aws preflight`
- timestamp: `2026-05-27 08:34:20-07:00`
- key result: `COMMAND_EXIT_CODE="0"`
- cited lines: Terraform configuration valid; shell syntax and suite JSON checks passed
