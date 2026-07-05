# Manifest: SPIRE AWS Auto-Stop Preflight

- head SHA: `e5c26d1b07f4017f4458fed9084b560402095d7a`
- task bucket: `reviews/task-30/`
- packet path: `reviews/task-30/1008-spire-phase13e-autostop-preflight/`
- timestamp: `2026-05-27T15:38:29Z`
- lane: SPIRE Phase 13e AWS operator preflight
- fixture: packet-local fake `aws ec2 describe-images`
- storage format: not applicable
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surface: not applicable
- AWS provisioning: not run
- real AWS API calls: not run for operator-preflight tests

## Artifacts

### `artifacts/fake-bin/aws`

- command: packet-local fake for `aws ec2 describe-images`
- key result: returns `arm64`

### `artifacts/tfvars/future.tfvars`

- command: input fixture with `auto_stop_at = "2026-05-28T06:00:00Z"`
- key result: should pass when now is pinned to `2026-05-27T12:00:00Z`

### `artifacts/tfvars/expired.tfvars`

- command: input fixture with `auto_stop_at = "2026-05-27T06:00:00Z"`
- key result: should fail when now is pinned to `2026-05-27T12:00:00Z`

### `artifacts/bash-n-preflight-operator.log`

- command: `bash -n scripts/spire-aws/preflight-operator.sh`
- timestamp: `2026-05-27 08:37:31-07:00`
- key result: `COMMAND_EXIT_CODE="0"`

### `artifacts/preflight-future.log`

- command: `PATH="$PWD/.../fake-bin:$PATH" SPIRE_AWS_PREFLIGHT_NOW_EPOCH=1779883200 scripts/spire-aws/preflight-operator.sh artifacts/tfvars/future.tfvars`
- timestamp: `2026-05-27 08:37:53-07:00`
- key result: `COMMAND_EXIT_CODE="0"`
- cited line: `SPIRE AWS operator preflight passed: region=us-west-2 az=us-west-2a ami=ami-0123456789abcdef0 coordinator=m7g.large remote=m7g.large remote_count=3`

### `artifacts/preflight-expired.log`

- command: `PATH="$PWD/.../fake-bin:$PATH" SPIRE_AWS_PREFLIGHT_NOW_EPOCH=1779883200 scripts/spire-aws/preflight-operator.sh artifacts/tfvars/expired.tfvars`
- timestamp: `2026-05-27 08:37:53-07:00`
- key result: `COMMAND_EXIT_CODE="2"`
- cited line: `ERROR: auto_stop_at must be in the future before provisioning, got: 2026-05-27T06:00:00Z`

### `artifacts/preflight-local-tfvars.log`

- command: `PATH="$PWD/.../fake-bin:$PATH" scripts/spire-aws/preflight-operator.sh infra/spire-aws/terraform.tfvars`
- timestamp: `2026-05-27 08:37:31-07:00`
- key result: `COMMAND_EXIT_CODE="0"`
- cited line: `SPIRE AWS operator preflight passed: region=us-west-2 az=us-west-2a ami=ami-04e0d7d889f694536 coordinator=m7g.large remote=m7g.large remote_count=3`

### `artifacts/preflight.log`

- command: `make -C infra/spire-aws preflight`
- timestamp: `2026-05-27 08:38:02-07:00`
- key result: `COMMAND_EXIT_CODE="0"`
- cited lines: Terraform configuration valid; shell syntax and suite JSON checks passed
