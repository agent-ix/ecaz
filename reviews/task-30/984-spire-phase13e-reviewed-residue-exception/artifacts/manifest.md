# Artifact Manifest: SPIRE Phase 13e Reviewed Residue Exception

Head SHA: `5e361d33a7d3e0ab88d470ccf80c008000ff6c66`
Task bucket: `reviews/task-30/984-spire-phase13e-reviewed-residue-exception`
Timestamp: `2026-05-25T22:00:48Z`
Lane: local/AWS preflight exception evidence
AWS resources provisioned: no
AWS resources modified/deleted: no
Storage format / rerank mode: not applicable
Surface isolation: not applicable; no benchmark tables were created

## Artifacts

| Artifact | Command | Result |
| --- | --- | --- |
| `bash-syntax.log` | `bash -n scripts/spire-aws/preflight-permissions.sh scripts/spire-aws/archive-local-state.sh scripts/spire-aws/preflight-state.sh` | exit 0 |
| `preflight-permissions-override.log` | `SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1 make -C infra/spire-aws preflight-permissions` | exit 0; old buckets warned, Secrets Manager list OK |
| `archive-state-synthetic.log` | `archive-local-state.sh` against synthetic state | exit 0 |
| `pre-archive-state-resources.txt` | `jq ... infra/spire-aws/terraform.tfstate` before archive | records `aws_s3_bucket.artifacts` |
| `pre-archive-state-sha256.txt` | `sha256sum infra/spire-aws/terraform.tfstate` before archive | records source state hash |
| `archive-state-actual.log` | actual local state archive summary | exit 0; state moved aside |
| `preflight-state-after-archive.log` | `make -C infra/spire-aws preflight-state` | exit 0; no local state file |
| `git-diff-check.log` | `git diff --check` | exit 0 |

## Key Result Lines

- Old buckets: `ecaz-spire-aws-20260523165108075000000003`, `ecaz-spire-aws-20260525201045387900000003`
- Override warnings: `missing s3:ListBucketVersions for pre-existing residue bucket`
- Permission preflight result: `SPIRE AWS permission preflight passed`
- Pre-archive resource: `aws_s3_bucket.artifacts`
- State preflight after archive: `SPIRE AWS state preflight passed: no local Terraform state file`
