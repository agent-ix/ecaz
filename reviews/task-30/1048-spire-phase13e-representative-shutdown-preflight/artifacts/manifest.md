# Manifest

- head SHA: `3acd2c2fb02e539355d2cff7393907b4add810fd`
- task bucket: `reviews/task-30`
- packet: `reviews/task-30/1048-spire-phase13e-representative-shutdown-preflight`
- timestamp: `2026-05-28T15:09:54Z`
- lane: Phase 13e representative AWS performance preflight
- fixture: established Graviton/aarch64 lane (`us-west-2`, `us-west-2a`, `m7g.large`, 1 coordinator + 3 remotes)
- storage format: SPIRE distributed remote placements
- rerank mode: production read profile suites; dry-run only, no provisioning
- isolated one-index-per-table vs shared-table surface: one-index-per-table representative SPIRE surface

## Artifacts

| Artifact | Command | Key result |
| --- | --- | --- |
| `bash-n-post-commit.log` | `bash -n scripts/spire-aws/preflight-representative-performance.sh scripts/spire-aws/run-representative-performance-pass.sh scripts/spire-aws/refresh-auto-stop-at.sh scripts/spire-aws/check-watchdog-local.sh scripts/spire-aws/check-cleanup-residue-local.sh` | shell syntax passed |
| `preflight-post-commit.log` | `scripts/spire-aws/preflight-representative-performance.sh` | `SPIRE representative performance preflight passed` |
| `refresh-auto-stop-at.log` | `scripts/spire-aws/refresh-auto-stop-at.sh reviews/task-30/1048-spire-phase13e-representative-shutdown-preflight/artifacts/tfvars-refresh-fixture.tfvars 6` | refreshed fixture `auto_stop_at=2026-05-28T21:08:52Z` |
| `representative-pass-dry-run.log` | `scripts/spire-aws/run-representative-performance-pass.sh --artifact-dir reviews/task-30/1048-spire-phase13e-representative-shutdown-preflight/artifacts` before the auto-stop refresh wiring | expected local failure: stale `auto_stop_at=2026-05-28T14:00:00Z` |
| `representative-pass-dry-run-post-commit.log` | same representative pass command after the code commit | refreshed ignored local `terraform.tfvars` to `2026-05-28T23:09:25Z`; operator, state, permissions, and representative preflight passed; dry-run only |
| `aws-running-before.log` | `aws ec2 describe-instances --region us-west-2 --filters Name=instance-state-name,Values=pending,running,stopping ...` | no rows |
| `aws-running-after-post-commit.log` | same EC2 state check after post-commit dry-run | no rows |
| `tfvars-refresh-fixture.tfvars` | copied from `infra/spire-aws/terraform.tfvars.example`, then updated by `refresh-auto-stop-at.sh` | packet-local fixture showing the rewritten auto-stop deadline |

## Cited Lines

- `representative-pass-dry-run.log`: `ERROR: auto_stop_at must be in the future before provisioning, got: 2026-05-28T14:00:00Z`
- `representative-pass-dry-run-post-commit.log`: `Updated /home/peter/dev/ecaz/infra/spire-aws/terraform.tfvars auto_stop_at=2026-05-28T23:09:25Z`
- `representative-pass-dry-run-post-commit.log`: `SPIRE AWS operator preflight passed: region=us-west-2 az=us-west-2a ami=ami-04e0d7d889f694536 coordinator=m7g.large remote=m7g.large remote_count=3`
- `representative-pass-dry-run-post-commit.log`: `SPIRE AWS state preflight passed: local Terraform state has no managed resources`
- `representative-pass-dry-run-post-commit.log`: `SPIRE AWS permission preflight passed`
- `representative-pass-dry-run-post-commit.log`: `SPIRE representative performance preflight passed`
- `representative-pass-dry-run-post-commit.log`: `Dry run only. Re-run with --execute after explicit AWS approval.`
