# Artifact Manifest

- Head SHA: `2a9435ccb`
- Task bucket: `reviews/task-30/1030-spire-phase13e-autostop-lead-gate`
- Timestamp: `2026-05-27T11:08:19-07:00`
- Lane: Phase 13e AWS operator preflight safety
- Fixture / storage / rerank mode: not applicable; no benchmark fixture
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `bash-n-preflight-operator.log`

- Command: `bash -n scripts/spire-aws/preflight-operator.sh`
- Result: passed.

### `preflight-good-lead.log`

- Command: `env PATH=reviews/task-30/1030-spire-phase13e-autostop-lead-gate/artifacts/fake-bin:/usr/bin:/bin SPIRE_AWS_PREFLIGHT_NOW_EPOCH=1779904800 bash scripts/spire-aws/preflight-operator.sh reviews/task-30/1030-spire-phase13e-autostop-lead-gate/artifacts/tfvars/good-lead.tfvars`
- Result: passed.
- Key line:
  - `SPIRE AWS operator preflight passed: region=us-west-2 az=us-west-2a ami=ami-04e0d7d889f694536 coordinator=m7g.large remote=m7g.large remote_count=3`

### `preflight-short-lead.log`

- Command: `env PATH=reviews/task-30/1030-spire-phase13e-autostop-lead-gate/artifacts/fake-bin:/usr/bin:/bin SPIRE_AWS_PREFLIGHT_NOW_EPOCH=1779904800 bash scripts/spire-aws/preflight-operator.sh reviews/task-30/1030-spire-phase13e-autostop-lead-gate/artifacts/tfvars/short-lead.tfvars`
- Result: expected rejection, exit 2.
- Key line:
  - `ERROR: auto_stop_at must be at least 18000s after preflight time for the representative pass watchdog budget, got: 2026-05-27T20:00:00Z`

### `preflight-current-tfvars.log`

- Command: `bash scripts/spire-aws/preflight-operator.sh infra/spire-aws/terraform.tfvars`
- Result: passed.
- Key line:
  - `SPIRE AWS operator preflight passed: region=us-west-2 az=us-west-2a ami=ami-04e0d7d889f694536 coordinator=m7g.large remote=m7g.large remote_count=3`

### `git-show-stat.log`

- Command: `git show --stat --oneline --decorate --no-renames 2a9435ccb`
- Key lines:
  - `2a9435ccb (HEAD -> diskann-aws-optimization) Gate SPIRE AWS auto-stop lead time`
  - `plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md | 4 ++++`
  - `scripts/spire-aws/preflight-operator.sh                        | 5 +++++`

### `git-diff-check.log`

- Command: `git diff --check HEAD^ HEAD -- scripts/spire-aws/preflight-operator.sh plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`
- Result: passed with no output.

### Fixtures

- `fake-bin/aws`: fake `aws ec2 describe-images` responder that prints `arm64`.
- `tfvars/good-lead.tfvars`: same Graviton lane with enough auto-stop lead.
- `tfvars/short-lead.tfvars`: same Graviton lane with insufficient auto-stop lead.

## Notes

No AWS provisioning command was run. The only real AWS call was the read-only AMI
architecture lookup performed by `preflight-current-tfvars.log`.
