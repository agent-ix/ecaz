# Artifact Manifest

- Head SHA: `cc4f2c2d0`
- Task bucket: `reviews/task-30/1029-spire-phase13e-current-aws-preflight`
- Timestamp: `2026-05-27T11:03:24-07:00`
- Lane: Phase 13e current AWS preflight readiness
- Fixture / storage / rerank mode: not applicable; no benchmark fixture
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `preflight-operator-current.log`

- Command: `bash scripts/spire-aws/preflight-operator.sh infra/spire-aws/terraform.tfvars`
- Result: passed.
- Key line:
  - `SPIRE AWS operator preflight passed: region=us-west-2 az=us-west-2a ami=ami-04e0d7d889f694536 coordinator=m7g.large remote=m7g.large remote_count=3`

### `preflight-state-current.log`

- Command: `bash scripts/spire-aws/preflight-state.sh infra/spire-aws/terraform.tfstate`
- Result: passed.
- Key line:
  - `SPIRE AWS state preflight passed: local Terraform state has no managed resources`

### `preflight-permissions-current.log`

- Command: `bash scripts/spire-aws/preflight-permissions.sh`
- Result: expected current-state failure, exit 2.
- Key lines:
  - `AWS identity: arn:aws:iam::932658697181:user/ecaz-operator`
  - six `ecaz-spire-aws-*` buckets matched the cleanup-sensitive prefix.
  - each bucket failed `s3:ListBucketVersions` with `AccessDenied`.
  - `Secrets Manager list permission ok for prefix ecaz-spire-aws`

### `preflight-permissions-current-override.log`

- Command: `SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1 bash scripts/spire-aws/preflight-permissions.sh`
- Result: passed with warnings for the documented old buckets.
- Key line:
  - `SPIRE AWS permission preflight passed`

### `make-current-aws-preflight.log`

- Command: `SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1 make -C infra/spire-aws preflight-operator preflight-state preflight-permissions preflight-representative-performance`
- Result: passed.
- Key lines:
  - `SPIRE AWS operator preflight passed: region=us-west-2 az=us-west-2a ami=ami-04e0d7d889f694536 coordinator=m7g.large remote=m7g.large remote_count=3`
  - `SPIRE AWS state preflight passed: local Terraform state has no managed resources`
  - `SPIRE AWS permission preflight passed`
  - `SPIRE representative performance preflight passed: priority=/home/peter/dev/ecaz/scripts/spire-aws/suite-representative-priority.json pooling=/home/peter/dev/ecaz/scripts/spire-aws/suite-representative-pooling.json`

### `git-show-stat.log`

- Command: `git show --stat --oneline --decorate --no-renames cc4f2c2d0`
- Key lines:
  - `cc4f2c2d0 (HEAD -> diskann-aws-optimization) Record SPIRE current AWS preflight status`
  - `plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md | 11 +++++++++++`

### `git-diff-check.log`

- Command: `git diff --check cc4f2c2d0^ cc4f2c2d0 -- plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`
- Result: passed with no output.

## Notes

No provisioning, teardown, EC2 start, EC2 stop, Terraform apply, or Terraform
destroy command was run. Existing untracked SPIRE artifact directories were left
untouched and were not staged.
